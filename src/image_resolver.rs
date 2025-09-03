use crate::config::CacheConfig;
use crate::error::VMError;
use crate::wheel_resources::WheelResources;
use anyhow::Result;
use log::{debug, info, warn};
use pyo3::Python;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct ImageResolver {
    cache_config: CacheConfig,
}

const CANONICAL_IMAGE: &str = "localhost/flashvm:python-basic";
const EMBEDDED_TAG: &str = "python-basic";

impl ImageResolver {
    pub fn new() -> Self { Self { cache_config: CacheConfig::default() } }

    /// Resolve image reference:
    /// - None => import (once) the embedded OCI layout → containers-storage: and return canonical name
    /// - Some => validate/normalize (accepts docker://, containers-storage:, simple name, oci:/dir..., dir:, oci-archive:)
    pub fn resolve_image_ref(&self, image_ref: Option<&str>) -> Result<String, VMError> {
        match image_ref {
            None => {
                self.ensure_embedded_image_imported()?;
                Ok(CANONICAL_IMAGE.to_string())
            }
            Some(s) => self.validate_image_ref(s),
        }
    }

    /// Import the embedded OCI layout into containers-storage (idempotent).
    fn ensure_embedded_image_imported(&self) -> Result<(), VMError> {
        if self.image_exists_in_storage(CANONICAL_IMAGE)? {
            debug!("Image already present in containers-storage: {}", CANONICAL_IMAGE);
            return Ok(());
        }

        let oci_path: PathBuf = Python::with_gil(|py| {
            WheelResources::find_embedded_data_path(py).map_err(|e| {
                VMError::ImageResolution(format!("Failed to locate embedded data: {}", e))
            })
        })?
        .ok_or_else(|| {
            VMError::ImageResolution(
                "Embedded OCI image not found (flashvm/data/oci)".to_string(),
            )
        })?;

        self.validate_oci_layout_dir(&oci_path)?;

        let source_oci = format!("oci:{}:{}", oci_path.to_string_lossy(), EMBEDDED_TAG);

        if self.command_exists("skopeo") {
            info!(
                "Importing embedded image with skopeo: {} -> containers-storage:{}",
                source_oci, CANONICAL_IMAGE
            );
            let copy_cmd = format!(
                "skopeo copy --insecure-policy '{}' 'containers-storage:{}'",
                source_oci, CANONICAL_IMAGE
            );
            if self.run_in_buildah_unshare(&copy_cmd)? {
                self.mark_import_sentinel(&oci_path)?;
                return Ok(());
            } else {
                warn!("skopeo copy failed; trying fallback with buildah");
            }
        }

        info!("Importing via buildah (fallback) from {}", source_oci);
        let from_out = self.run_in_buildah_unshare_capture(&format!("buildah from '{}'", source_oci))?;
        if !from_out.success {
            return Err(VMError::ImageResolution(format!(
                "buildah from failed: {}",
                from_out.stderr
            )));
        }
        let container_name = from_out.stdout.trim();
        if container_name.is_empty() {
            return Err(VMError::ImageResolution(
                "buildah from did not return a container name".to_string(),
            ));
        }
        let ok_commit =
            self.run_in_buildah_unshare(&format!("buildah commit '{}' '{}'", container_name, CANONICAL_IMAGE))?;
        let _ = self.run_in_buildah_unshare(&format!("buildah rm '{}'", container_name));
        if !ok_commit {
            return Err(VMError::ImageResolution(
                "buildah commit failed in fallback".to_string(),
            ));
        }
        self.mark_import_sentinel(&oci_path)?;
        Ok(())
    }

    fn mark_import_sentinel(&self, oci_path: &Path) -> Result<(), VMError> {
        let cache_dir = PathBuf::from(&self.cache_config.cache_dir).join("state");
        fs::create_dir_all(&cache_dir)?;
        let sentinel_path = cache_dir.join("embedded_import.json");
        let content = format!(
            "{{\n  \"image\": \"{}\",\n  \"oci_path\": \"{}\",\n  \"version\": \"{}\"\n}}\n",
            CANONICAL_IMAGE,
            oci_path.to_string_lossy(),
            env!("CARGO_PKG_VERSION")
        );
        let mut f = fs::File::create(&sentinel_path)?;
        f.write_all(content.as_bytes())?;
        Ok(())
    }

    fn image_exists_in_storage(&self, name: &str) -> Result<bool, VMError> {
        let out = self.run_in_buildah_unshare_capture("buildah images --format '{{.Name}}:{{.Tag}}'")?;
        if !out.success {
            return Err(VMError::Execution(format!(
                "Failed to list images in containers-storage: {}",
                out.stderr
            )));
        }
        Ok(out.stdout.lines().any(|l| l.trim() == name))
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
        })
    }

    fn validate_oci_layout_dir(&self, oci_dir: &Path) -> Result<(), VMError> {
        let layout_file = oci_dir.join("oci-layout");
        let index_file = oci_dir.join("index.json");
        let blobs_sha256_dir = oci_dir.join("blobs").join("sha256");
        if !layout_file.exists() || !index_file.exists() || !blobs_sha256_dir.exists() {
            return Err(VMError::ImageResolution(format!(
                "Invalid OCI layout in {}: requires oci-layout, index.json and blobs/sha256",
                oci_dir.to_string_lossy()
            )));
        }
        Ok(())
    }

    fn validate_image_ref(&self, image_ref: &str) -> Result<String, VMError> {
        info!("Validating image reference: {}", image_ref);

        if image_ref.starts_with("docker://") {
            self.validate_docker_ref(image_ref)
        } else if image_ref.starts_with("containers-storage:") {
            debug!("containers-storage accepted: {}", image_ref);
            Ok(image_ref.to_string())
        } else if image_ref.starts_with("oci:")
            || image_ref.starts_with("dir:")
            || image_ref.starts_with("oci-archive:")
        {
            self.get_image_path_ref(image_ref)
        } else {
            debug!("Assuming simple name (docker): {}", image_ref);
            Ok(image_ref.to_string())
        }
    }

    fn get_image_path_ref(&self, image_ref: &str) -> Result<String, VMError> {
        if image_ref.starts_with("oci:") {
            return self.validate_oci_ref(image_ref);
        }
        if image_ref.starts_with("docker://") {
            return self.validate_docker_ref(image_ref);
        }

        let path_ref = if image_ref.starts_with("dir:") {
            &image_ref[4..]
        } else if image_ref.starts_with("oci-archive:") {
            &image_ref[12..]
        } else {
            image_ref
        };

        let path = Path::new(path_ref);
        if path.exists() {
            return Ok(image_ref.to_string());
        }

        Err(VMError::ImageResolution(format!(
            "Could not resolve image reference: {}",
            image_ref
        )))
    }

    fn validate_oci_ref(&self, oci_ref: &str) -> Result<String, VMError> {
        // oci:<path>[:tag]
        let parts: Vec<&str> = oci_ref.splitn(2, ':').collect();
        if parts.len() < 2 {
            return Err(VMError::ImageResolution(
                "Invalid OCI reference. Expected: oci:<path>[:tag]".to_string(),
            ));
        }
        let path_part = parts[1];
        let (path, _tag) = if let Some(colon_pos) = path_part.rfind(':') {
            (&path_part[..colon_pos], &path_part[colon_pos + 1..])
        } else {
            (path_part, "latest")
        };

        let oci_path = Path::new(path);
        if !oci_path.exists() {
            return Err(VMError::ImageResolution(format!(
                "OCI path does not exist: {}",
                path
            )));
        }
        let layout_file = oci_path.join("oci-layout");
        let index_file = oci_path.join("index.json");
        let blobs_sha256 = oci_path.join("blobs").join("sha256");
        if !layout_file.exists() || !index_file.exists() || !blobs_sha256.exists() {
            return Err(VMError::ImageResolution(format!(
                "Invalid OCI layout in {}: requires oci-layout, index.json and blobs/sha256",
                path
            )));
        }
        debug!("OCI reference validated: {}", oci_ref);
        Ok(oci_ref.to_string())
    }

    fn validate_docker_ref(&self, docker_ref: &str) -> Result<String, VMError> {
        if !docker_ref.starts_with("docker://") {
            return Err(VMError::ImageResolution(
                "Docker reference must start with docker://".to_string(),
            ));
        }
        let image_name = &docker_ref[9..];
        if image_name.is_empty() {
            return Err(VMError::ImageResolution(
                "Docker image name cannot be empty".to_string(),
            ));
        }
        debug!("Docker reference validated ⇒ {}", image_name);
        Ok(image_name.to_string())
    }

    pub fn list_cached_images(&self) -> Result<Vec<String>, VMError> {
        Ok(vec![])
    }
    pub fn clear_cache(&self) -> Result<(), VMError> { Ok(()) }

    fn command_exists(&self, command: &str) -> bool {
        Command::new("which")
            .arg(command)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    pub fn embedded_is_imported(&self) -> Result<bool, VMError> {
        self.image_exists_in_storage(CANONICAL_IMAGE)
    }
    pub fn import_embedded_now(&self) -> Result<(), VMError> {
        self.ensure_embedded_image_imported()
    }
}

impl Default for ImageResolver { fn default() -> Self { Self::new() } }

#[derive(Debug)]
struct Captured {
    stdout: String,
    stderr: String,
    success: bool,
}
