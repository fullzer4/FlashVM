use crate::config::{Artifact, ExecutionResult, FileInput, FileOutput, VMConfig};
use crate::error::VMError;
use crate::image_resolver::ImageResolver;
use anyhow::Result;
use glob::glob;
use log::{debug, info};
use serde_json;
use std::fs;
use std::io::Write;
use std::os::unix::process::ExitStatusExt;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use tempfile::{NamedTempFile, TempDir};
use uuid::Uuid;

fn shell_escape(s: &str) -> String {
    if s.chars().all(|c| c.is_ascii_alphanumeric() || "/-_.:@".contains(c)) {
        s.to_string()
    } else {
        format!("'{}'", s.replace('\'', "'\\''"))
    }
}

struct WorkDirectories {
    _temp_base: TempDir,
    input_dir: std::path::PathBuf,
    output_dir: std::path::PathBuf,
    _tmp_dir: std::path::PathBuf,
    scripts_dir: std::path::PathBuf,
}

pub struct VMRunner {
    image_resolver: ImageResolver,
}

impl VMRunner {
    pub fn new() -> Self {
        Self {
            image_resolver: ImageResolver::new(),
        }
    }

    pub fn pre_pull_image(&self, image_ref: &str) -> Result<(), VMError> {
        let resolved = self.image_resolver.resolve_image_ref(Some(image_ref))?;
        let normalized = self.normalize_image_for_krunvm(&resolved)?;
        let vm_name = format!("prepull-{}", &Uuid::new_v4().to_string()[..8]);
        let script = format!(
            "set -e\n\
             krunvm create --cpus 1 --mem 256 --workdir /work --name {} {} >/dev/null\n\
             krunvm delete -f {} >/dev/null 2>&1 || krunvm delete {} >/dev/null 2>&1 || true\n",
            shell_escape(&vm_name),
            normalized,
            shell_escape(&vm_name),
            shell_escape(&vm_name),
        );
        let out = self.run_in_buildah_unshare_capture(&script)?;
        if !out.success {
            return Err(VMError::VMConfiguration(format!("pre-pull failed: {}", out.stderr)));
        }
        Ok(())
    }

    pub fn execute_python_code(
        &self,
        code: &str,
        config: &VMConfig,
        files_in: Vec<FileInput>,
        expect: Vec<FileOutput>,
    ) -> Result<ExecutionResult, VMError> {
        let start_time = Instant::now();

        info!("Starting execution with config: {:?}", config);

        self.check_dependencies()?;

        // Resolve image → nome aceitável pelo krunvm
        let resolved_image = self.image_resolver.resolve_image_ref(config.image.as_deref())?;
        let image_ref = self.normalize_image_for_krunvm(&resolved_image)?;
        info!("Using image: {}", image_ref);

        let temp_dirs = self.setup_work_directories()?;
        self.prepare_input_files(&files_in, &temp_dirs.input_dir)?;
        let script_file = self.create_python_script(code)?;

        let vm_result = self.run_vm_with_krunvm(&image_ref, &script_file, config, &temp_dirs)?;
        let artifacts = self.collect_artifacts(&expect, &temp_dirs.output_dir, config.max_bytes_inline)?;
        let execution_time = start_time.elapsed();

        Ok(ExecutionResult {
            stdout: vm_result.stdout,
            stderr: vm_result.stderr,
            exit_code: vm_result.exit_code,
            execution_time,
            artifacts,
            image_used: image_ref,
        })
    }

    fn normalize_image_for_krunvm(&self, image: &str) -> Result<String, VMError> {
        if let Some(name) = image.strip_prefix("containers-storage:") {
            return Ok(name.to_string());
        }
        if image.starts_with("oci:") {
            let tmp_name = format!("localhost/flashvm:imported-{}", &Uuid::new_v4().to_string()[..8]);
            self.import_oci_to_storage(image, &tmp_name)?;
            return Ok(tmp_name);
        }
        Ok(image.to_string())
    }

    fn import_oci_to_storage(&self, oci_ref: &str, dest_name: &str) -> Result<(), VMError> {
        if self.command_exists("skopeo") {
            let cmd = format!(
                "skopeo copy --insecure-policy '{}' 'containers-storage:{}'",
                oci_ref, dest_name
            );
            if self.run_in_buildah_unshare(&cmd)? {
                return Ok(());
            }
        }
        let from_out = self.run_in_buildah_unshare_capture(&format!("buildah from '{}'", oci_ref))?;
        if !from_out.success {
            return Err(VMError::Execution(format!("buildah from failed: {}", from_out.stderr)));
        }
        let container = from_out.stdout.trim();
        if container.is_empty() {
            return Err(VMError::Execution("buildah from did not return a name".to_string()));
        }
        let ok_commit =
            self.run_in_buildah_unshare(&format!("buildah commit '{}' '{}'", container, dest_name))?;
        let _ = self.run_in_buildah_unshare(&format!("buildah rm '{}'", container));
        if !ok_commit {
            return Err(VMError::Execution("buildah commit failed".to_string()));
        }
        Ok(())
    }

    fn check_dependencies(&self) -> Result<(), VMError> {
        if !self.command_exists("krunvm") {
            return Err(VMError::MissingDependency(
                "krunvm not found. Please install krunvm to continue.".to_string(),
            ));
        }
        if !self.command_exists("buildah") {
            return Err(VMError::MissingDependency(
                "buildah not found. Required for rootless.".to_string(),
            ));
        }
        if !Path::new("/dev/kvm").exists() {
            return Err(VMError::MissingDependency(
                "KVM not available. Ensure virtualization is enabled.".to_string(),
            ));
        }
        Ok(())
    }

    fn command_exists(&self, cmd: &str) -> bool {
        Command::new("which")
            .arg(cmd)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    fn run_in_buildah_unshare(&self, sh_cmd: &str) -> Result<bool, VMError> {
        debug!("Executing: buildah unshare sh -c '{}'", sh_cmd);
        let status = Command::new("buildah")
            .args(["unshare", "sh", "-c", sh_cmd])
            .status()
            .map_err(|e| VMError::Execution(format!("Failed to execute command: {}", e)))?;
        Ok(status.success())
    }

    fn run_in_buildah_unshare_capture(&self, sh_cmd: &str) -> Result<Captured, VMError> {
        debug!("Executing: buildah unshare sh -c '{}'", sh_cmd);
        let output = Command::new("buildah")
            .args(["unshare", "sh", "-c", sh_cmd])
            .output()
            .map_err(|e| VMError::Execution(format!("Failed to execute command: {}", e)))?;
        Ok(Captured {
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            success: output.status.success(),
            exit_code: output.status.code(),
            timed_out: false,
        })
    }

    fn run_in_buildah_unshare_capture_timeout(
        &self,
        sh_cmd: &str,
        timeout: Duration,
    ) -> Result<Captured, VMError> {
        debug!("Executing (timeout={:?}): buildah unshare sh -c '{}'", timeout, sh_cmd);
        let mut child = Command::new("buildah")
            .args(["unshare", "sh", "-c", sh_cmd])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| VMError::Execution(format!("Failed to spawn command: {}", e)))?;

        let mut stdout = child
            .stdout
            .take()
            .ok_or_else(|| VMError::Execution("Failed to capture stdout".to_string()))?;
        let mut stderr = child
            .stderr
            .take()
            .ok_or_else(|| VMError::Execution("Failed to capture stderr".to_string()))?;

        let stdout_handle = std::thread::spawn(move || {
            let mut buf = Vec::new();
            let _ = std::io::copy(&mut stdout, &mut buf);
            buf
        });
        let stderr_handle = std::thread::spawn(move || {
            let mut buf = Vec::new();
            let _ = std::io::copy(&mut stderr, &mut buf);
            buf
        });

        let start = Instant::now();
        let mut timed_out = false;
        let status = loop {
            match child.try_wait() {
                Ok(Some(status)) => break status,
                Ok(None) => {
                    if start.elapsed() >= timeout {
                        timed_out = true;
                        let _ = child.kill();
                        // Single wait; if it fails, synthesize a 124 exit status (like coreutils timeout)
                        break child
                            .wait()
                            .unwrap_or_else(|_| std::process::ExitStatus::from_raw(124 << 8));
                    }
                    std::thread::sleep(Duration::from_millis(25));
                }
                Err(e) => {
                    return Err(VMError::Execution(format!("Failed to wait for process: {}", e)));
                }
            }
        };

        let out_v = stdout_handle.join().unwrap_or_default();
        let err_v = stderr_handle.join().unwrap_or_default();

        let mut exit_code = status.code();
        if timed_out {
            // padroniza como 124 (semelhante a coreutils timeout)
            exit_code = Some(124);
        }

        Ok(Captured {
            stdout: String::from_utf8_lossy(&out_v).to_string(),
            stderr: String::from_utf8_lossy(&err_v).to_string(),
            success: status.success(),
            exit_code,
            timed_out,
        })
    }

    fn setup_work_directories(&self) -> Result<WorkDirectories, VMError> {
        let temp_base = TempDir::new().map_err(VMError::IO)?;
        let input_dir = temp_base.path().join("in");
        let output_dir = temp_base.path().join("out");
        let tmp_dir = temp_base.path().join("tmp");
        let scripts_dir = temp_base.path().join("scripts");
        fs::create_dir_all(&input_dir)?;
        fs::create_dir_all(&output_dir)?;
        fs::create_dir_all(&tmp_dir)?;
        fs::create_dir_all(&scripts_dir)?;
        Ok(WorkDirectories {
            input_dir,
            output_dir,
            _tmp_dir: tmp_dir,
            scripts_dir,
            _temp_base: temp_base,
        })
    }

    fn prepare_input_files(&self, files_in: &[FileInput], input_dir: &Path) -> Result<(), VMError> {
        for file_input in files_in {
            let target_path = input_dir.join(&file_input.guest_path);
            if let Some(parent) = target_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(&file_input.host_path, &target_path).map_err(VMError::IO)?;
            debug!("File copied: {:?} -> {:?}", file_input.host_path, target_path);
        }
        Ok(())
    }

    fn create_python_script(&self, code: &str) -> Result<NamedTempFile, VMError> {
        let mut script_file = NamedTempFile::new().map_err(VMError::IO)?;
        script_file.write_all(code.as_bytes()).map_err(VMError::IO)?;
        script_file.flush().map_err(VMError::IO)?;
        Ok(script_file)
    }

    fn create_guest_runner(
        &self,
        config: &VMConfig,
        scripts_dir: &Path,
        main_script: &str,
    ) -> Result<String, VMError> {
        let runner_path = scripts_dir.join("run.py");
        let env_json = serde_json::to_string(&config.env).map_err(|e| VMError::Execution(e.to_string()))?;
        let args_json =
            serde_json::to_string(&config.python_args).map_err(|e| VMError::Execution(e.to_string()))?;
        let runner_code = format!(
            "#!/usr/bin/env python3\n\
             import os, sys, json, subprocess\n\
             ENV=json.loads(r'''{}''')\n\
             PY_ARGS=json.loads(r'''{}''')\n\
             os.environ.update({{k:str(v) for k,v in ENV.items()}})\n\
             cmd=['/usr/bin/env','python3']+PY_ARGS+['/work/scripts/{}']\n\
             res=subprocess.run(cmd)\n\
             sys.exit(res.returncode)\n",
            env_json, args_json, main_script
        );
        fs::write(&runner_path, runner_code.as_bytes())?;
        Ok("/work/scripts/run.py".to_string())
    }

    fn run_vm_with_krunvm(
        &self,
        image_ref: &str,
        script_file: &NamedTempFile,
        config: &VMConfig,
        work_dirs: &WorkDirectories,
    ) -> Result<VMExecutionResult, VMError> {
        // Copia o script principal para /work/scripts/main.py
        let script_filename = "main.py";
        let script_target = work_dirs.scripts_dir.join(script_filename);
        fs::copy(script_file.path(), &script_target)?;
        if !script_target.exists() {
            return Err(VMError::IO(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Script was not copied correctly to {:?}", script_target),
            )));
        }

        let runner_path_guest = self.create_guest_runner(config, &work_dirs.scripts_dir, script_filename)?;

        // Comando dentro da VM: rodar diretamente python sem shell
        let start_cmd = format!(
            "krunvm start {vm} /usr/bin/env python3 {runner}",
            vm = "{vm}", // placeholder trocado no format abaixo
            runner = shell_escape(&runner_path_guest),
        );

        // Script de controle (create → start com retries → delete)
        let vm_name = format!("flashvm-{}", &Uuid::new_v4().to_string()[..8]);
        let work_base = work_dirs._temp_base.path().to_string_lossy().to_string();
        let mut ports_args = String::new();
        if config.network {
            for (host, guest) in &config.ports {
                ports_args.push_str(&format!(" --port {}:{}", host, guest));
            }
        }

        let shell_script = format!(
            "set -e\n\
             krunvm create --cpus {cpus} --mem {mem} --workdir {workdir} --name {vm} --volume '{work}:/work'{ports} {image} >/dev/null\n\
             set +e\n\
             tries=0\n\
             ec=1\n\
             while [ $tries -lt 3 ]; do\n\
               {start}\n\
               ec=$?\n\
               [ $ec -eq 0 ] && break\n\
               tries=$((tries+1))\n\
               sleep 0.15\n\
             done\n\
             set -e\n\
             krunvm delete -f {vm} >/dev/null 2>&1 || krunvm delete {vm} >/dev/null 2>&1 || true\n\
             exit $ec\n",
            cpus = config.cpus,
            mem = config.memory_mb,
            workdir = shell_escape(&config.workdir),
            vm = shell_escape(&vm_name),
            work = shell_escape(&work_base),
            ports = ports_args,
            image = image_ref,
            start = start_cmd.replace("{vm}", &shell_escape(&vm_name)),
        );

        // Timeout total = timeout de usuário + pequena folga p/ create/delete
        let hard_timeout = config.timeout + Duration::from_secs(2);
        let out = self.run_in_buildah_unshare_capture_timeout(&shell_script, hard_timeout)?;

        // Cleanup extra se houve timeout ou falha antes do delete interno
        if out.timed_out || !out.success {
            let _ = self.run_in_buildah_unshare(&format!(
                "krunvm delete -f {vm} >/dev/null 2>&1 || krunvm delete {vm} >/dev/null 2>&1 || true",
                vm = shell_escape(&vm_name)
            ));
        }

        Ok(VMExecutionResult {
            stdout: out.stdout,
            stderr: out.stderr,
            exit_code: out.exit_code.unwrap_or(-1),
            success: out.success,
        })
    }

    fn collect_artifacts(
        &self,
        expect: &[FileOutput],
        output_dir: &Path,
        max_inline: u64,
    ) -> Result<Vec<Artifact>, VMError> {
        let mut artifacts = Vec::new();
        for file_output in expect {
            let pattern = output_dir.join(&file_output.pattern);
            let pattern_str = pattern.to_string_lossy().to_string();
            for entry in glob(&pattern_str).map_err(|e| VMError::Execution(e.msg.to_string()))? {
                if let Ok(path) = entry {
                    if path.is_file() {
                        let metadata = fs::metadata(&path)?;
                        let size_bytes = metadata.len();
                        let content = if size_bytes <= max_inline {
                            Some(fs::read(&path)?)
                        } else {
                            None
                        };
                        let guest_rel = path.strip_prefix(output_dir).unwrap_or(&path);
                        let guest_path = format!("out/{}", guest_rel.to_string_lossy());
                        artifacts.push(Artifact {
                            guest_path,
                            host_path: path.clone(),
                            size_bytes,
                            content,
                        });
                    }
                }
            }
        }
        Ok(artifacts)
    }
}

#[derive(Debug)]
struct VMExecutionResult {
    stdout: String,
    stderr: String,
    exit_code: i32,
    success: bool,
}

#[derive(Debug)]
struct Captured {
    stdout: String,
    stderr: String,
    success: bool,
    exit_code: Option<i32>,
    timed_out: bool,
}
