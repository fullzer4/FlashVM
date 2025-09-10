use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::collections::HashMap;
use std::time::Duration;

mod vm_runner;
mod image_resolver;
mod config;
mod error;
mod wheel_resources;

use vm_runner::VMRunner;
use image_resolver::ImageResolver;
use config::{FileInput, FileOutput, VMConfig};
use error::PyVMError;
use crate::error::VMError as InternalVMError;
use wheel_resources::find_embedded_data_path;

#[pyfunction]
#[pyo3(signature = (
    code,
    image = None,
    cpus = None,
    memory_mb = None,
    env = None,
    timeout_seconds = None,
    workdir = None,
    python_args = None,
    network = None,
    ports = None,
    files_in = None,
    expect = None,
    max_bytes_inline = None,
))]
fn run(
    py: Python,
    code: String,
    image: Option<String>,
    cpus: Option<u32>,
    memory_mb: Option<u32>,
    env: Option<HashMap<String, String>>,
    timeout_seconds: Option<u64>,
    workdir: Option<String>,
    python_args: Option<Vec<String>>,
    network: Option<bool>,
    ports: Option<Vec<(u16, u16)>>,
    files_in: Option<Vec<(String, String)>>,
    expect: Option<Vec<String>>,
    max_bytes_inline: Option<u64>,
) -> PyResult<PyObject> {
    let config = VMConfig {
        image,
        cpus: cpus.unwrap_or(1),
        memory_mb: memory_mb.unwrap_or(512),
        env: env.unwrap_or_default(),
        workdir: workdir.unwrap_or_else(|| "/work".to_string()),
        timeout: Duration::from_secs(timeout_seconds.unwrap_or(30)),
        network: network.unwrap_or(false),
        ports: ports.unwrap_or_default(),
        python_args: python_args.unwrap_or_else(|| vec!["-u".to_string()]),
        max_bytes_inline: max_bytes_inline.unwrap_or(1024 * 1024),
    };

    if !config.workdir.starts_with('/') || config.workdir.matches('/').count() > 1 {
        return Err(PyVMError::new_err(
            "workdir must be a top-level directory (e.g., /work)".to_string(),
        ));
    }

    let files_in_vec: Vec<FileInput> = files_in
        .unwrap_or_default()
        .into_iter()
        .map(|(host, guest)| FileInput {
            host_path: std::path::PathBuf::from(host),
            guest_path: guest,
        })
        .collect();

    let expect_vec: Vec<FileOutput> = expect
        .unwrap_or_default()
        .into_iter()
        .map(|pat| FileOutput { pattern: pat })
        .collect();

    let result = py.allow_threads(|| {
        let runner = VMRunner::new();
        runner.execute_python_code(&code, &config, files_in_vec, expect_vec)
    });

    match result {
        Ok(execution_result) => {
            let mut stdout = execution_result.stdout;
            let mut stderr = execution_result.stderr;
            let exit_code = execution_result.exit_code;

            // Se houve erro e STDERR vier vazio (krunvm pode colapsar streams), duplica STDOUT.
            if exit_code != 0 && stderr.trim().is_empty() && !stdout.trim().is_empty() {
                stderr = stdout.clone();
            }

            let dict = PyDict::new_bound(py);
            dict.set_item("stdout", stdout)?;
            dict.set_item("stderr", stderr)?;
            dict.set_item("exit_code", exit_code)?;
            let exec_ms = std::cmp::max(1, execution_result.execution_time.as_millis() as u64);
            dict.set_item("execution_time_ms", exec_ms)?;
            dict.set_item("image_used", execution_result.image_used)?;

            let artifacts_py = pyo3::types::PyList::empty_bound(py);
            for a in execution_result.artifacts {
                let a_dict = PyDict::new_bound(py);
                a_dict.set_item("guest_path", a.guest_path)?;
                a_dict.set_item("host_path", a.host_path.to_string_lossy().to_string())?;
                a_dict.set_item("size_bytes", a.size_bytes)?;
                if let Some(content) = a.content {
                    a_dict.set_item("content", pyo3::types::PyBytes::new_bound(py, &content))?;
                }
                artifacts_py.append(a_dict)?;
            }
            dict.set_item("artifacts", artifacts_py)?;
            Ok(dict.into())
        }
        Err(e) => Err(PyVMError::new_err(format!("Execution error: {}", e))),
    }
}

#[pyfunction]
fn run_with_config(py: Python, code: String, config: &Bound<PyDict>) -> PyResult<PyObject> {
    let image = config.get_item("image")?.and_then(|v| v.extract::<String>().ok());
    let cpus = config.get_item("cpus")?.and_then(|v| v.extract::<u32>().ok()).unwrap_or(1);
    let memory_mb = config.get_item("memory_mb")?.and_then(|v| v.extract::<u32>().ok()).unwrap_or(512);
    let env = config.get_item("env")?.and_then(|v| v.extract::<HashMap<String, String>>().ok()).unwrap_or_default();
    let timeout_seconds = config.get_item("timeout_seconds")?.and_then(|v| v.extract::<u64>().ok()).unwrap_or(30);
    let workdir = config.get_item("workdir")?.and_then(|v| v.extract::<String>().ok()).unwrap_or_else(|| "/work".to_string());
    let python_args = config.get_item("python_args")?.and_then(|v| v.extract::<Vec<String>>().ok()).unwrap_or_else(|| vec!["-u".to_string()]);
    let network = config.get_item("network")?.and_then(|v| v.extract::<bool>().ok()).unwrap_or(false);
    let ports = config.get_item("ports")?.and_then(|v| v.extract::<Vec<(u16,u16)>>().ok()).unwrap_or_default();
    let files_in = config.get_item("files_in")?.and_then(|v| v.extract::<Vec<(String,String)>>().ok()).unwrap_or_default();
    let expect = config.get_item("expect")?.and_then(|v| v.extract::<Vec<String>>().ok()).unwrap_or_default();
    let max_bytes_inline = config.get_item("max_bytes_inline")?.and_then(|v| v.extract::<u64>().ok()).unwrap_or(1024*1024);

    let vm_config = VMConfig {
        image,
        cpus,
        memory_mb,
        env,
        workdir,
        timeout: Duration::from_secs(timeout_seconds),
        network,
        ports,
        python_args,
        max_bytes_inline,
    };

    if !vm_config.workdir.starts_with('/') || vm_config.workdir.matches('/').count() > 1 {
        return Err(PyVMError::new_err("workdir must be top-level (e.g., /work)".to_string()));
    }

    let files_in_vec: Vec<FileInput> = files_in
        .into_iter()
        .map(|(host, guest)| FileInput {
            host_path: std::path::PathBuf::from(host),
            guest_path: guest,
        })
        .collect();

    let expect_vec: Vec<FileOutput> = expect.into_iter().map(|pat| FileOutput { pattern: pat }).collect();

    let result = py.allow_threads(|| {
        let runner = VMRunner::new();
        runner.execute_python_code(&code, &vm_config, files_in_vec, expect_vec)
    });

    match result {
        Ok(execution_result) => {
            let mut stdout = execution_result.stdout;
            let mut stderr = execution_result.stderr;
            let exit_code = execution_result.exit_code;

            if exit_code != 0 && stderr.trim().is_empty() && !stdout.trim().is_empty() {
                stderr = stdout.clone();
            }

            let dict = PyDict::new_bound(py);
            dict.set_item("stdout", stdout)?;
            dict.set_item("stderr", stderr)?;
            dict.set_item("exit_code", exit_code)?;
            let exec_ms = std::cmp::max(1, execution_result.execution_time.as_millis() as u64);
            dict.set_item("execution_time_ms", exec_ms)?;
            dict.set_item("image_used", execution_result.image_used)?;
            let artifacts_py = pyo3::types::PyList::empty_bound(py);
            for a in execution_result.artifacts {
                let a_dict = PyDict::new_bound(py);
                a_dict.set_item("guest_path", a.guest_path)?;
                a_dict.set_item("host_path", a.host_path.to_string_lossy().to_string())?;
                a_dict.set_item("size_bytes", a.size_bytes)?;
                if let Some(content) = a.content { a_dict.set_item("content", pyo3::types::PyBytes::new_bound(py, &content))?; }
                artifacts_py.append(a_dict)?;
            }
            dict.set_item("artifacts", artifacts_py)?;
            Ok(dict.into())
        }
        Err(e) => Err(PyVMError::new_err(format!("Execution error: {}", e))),
    }
}

#[pyfunction]
#[pyo3(signature = (image=None, packages=None, tag=None, index_url=None, extra_index_url=None))]
fn prepare_image(
    py: Python,
    image: Option<String>,
    packages: Option<Vec<String>>, 
    tag: Option<String>,
    index_url: Option<String>,
    extra_index_url: Option<String>,
) -> PyResult<bool> {
    let result: Result<bool, InternalVMError> = py.allow_threads(|| {
        let resolver = ImageResolver::new();
        match (image, packages) {
            (None, None) => {
                // Import embedded image (idempotent)
                resolver.import_embedded_now()?;
                Ok(true)
            }
            (Some(img), None) => {
                // Validate and pre-pull docker-like refs (keeps behavior)
                let validated = resolver.resolve_image_ref(Some(&img))?;
                let is_docker_like = img.starts_with("docker://") || !img.starts_with("oci:");
                if is_docker_like {
                    let runner = VMRunner::new();
                    runner.pre_pull_image(&validated)?;
                }
                Ok(true)
            }
            // With packages: layer pip installs on top of base (embedded or provided)
            (img_opt, Some(pkgs)) => {
                let base = img_opt.as_deref();
                // Default tag: overwrite canonical so image=None uses the baked image next runs
                let target_tag = tag.as_deref().unwrap_or("python-basic");
                let _new_img = resolver.pip_install_into_image(
                    base,
                    &pkgs,
                    Some(target_tag),
                    index_url.as_deref(),
                    extra_index_url.as_deref(),
                )?;
                Ok(true)
            }
        }
    });

    match result {
        Ok(v) => Ok(v),
        Err(e) => Err(PyVMError::new_err(format!("Error preparing image: {}", e))),
    }
}

#[pyfunction]
#[pyo3(signature = (packages, base_image=None, tag=None, index_url=None, extra_index_url=None))]
fn pip_prepare_image(
    packages: Vec<String>,
    base_image: Option<String>,
    tag: Option<String>,
    index_url: Option<String>,
    extra_index_url: Option<String>,
) -> PyResult<String> {
    let resolver = ImageResolver::new();
    let image = resolver
        .pip_install_into_image(
            base_image.as_deref(),
            &packages,
            tag.as_deref(),
            index_url.as_deref(),
            extra_index_url.as_deref(),
        )
        .map_err(|e| PyVMError::new_err(format!("pip_prepare_image error: {}", e)))?;
    Ok(image)
}

#[pyfunction]
fn list_cached_images(py: Python) -> PyResult<Vec<String>> {
    let result = py.allow_threads(|| {
        let resolver = ImageResolver::new();
        resolver.list_cached_images()
    });
    match result {
        Ok(images) => Ok(images),
        Err(e) => Err(PyVMError::new_err(format!("Error listing images: {}", e))),
    }
}

#[pyfunction]
fn clear_cache(py: Python) -> PyResult<bool> {
    let result = py.allow_threads(|| {
        let resolver = ImageResolver::new();
        resolver.clear_cache()
    });
    match result {
        Ok(_) => Ok(true),
        Err(e) => Err(PyVMError::new_err(format!("Error clearing cache: {}", e))),
    }
}

#[pyfunction]
fn doctor(py: Python) -> PyResult<PyObject> {
    let dict: Bound<'_, PyDict> = PyDict::new_bound(py);

    let krunvm_available = std::process::Command::new("krunvm").arg("--version").output().is_ok();
    let buildah_available = std::process::Command::new("buildah").arg("--version").output().is_ok();
    let skopeo_available = std::process::Command::new("skopeo").arg("--version").output().is_ok();
    let kvm_available = std::path::Path::new("/dev/kvm").exists();

    let offline_available = wheel_resources::WheelResources::check_embedded_image_available(py, "python-basic")
        .unwrap_or(false);

    let mut embedded_imported = false;
    if offline_available {
        if let Ok(resolver) = std::panic::catch_unwind(|| ImageResolver::new()) {
            if let Ok(imported) = resolver.embedded_is_imported() {
                embedded_imported = imported;
            }
        }
    }

    let mut offline_message: Option<String> = None;
    if !offline_available {
        if let Ok(Some(oci_path)) = wheel_resources::WheelResources::find_embedded_data_path(py) {
            let layout = oci_path.join("oci-layout");
            let index = oci_path.join("index.json");
            let blobs = oci_path.join("blobs").join("sha256");
            let mut missing = vec![];
            if !layout.exists() { missing.push("oci-layout"); }
            if !index.exists() { missing.push("index.json"); }
            if !blobs.exists() { missing.push("blobs/sha256"); }
            if !missing.is_empty() {
                offline_message = Some(format!(
                    "Embedded OCI layout incomplete at {}: missing {}",
                    oci_path.to_string_lossy(),
                    missing.join(", ")
                ));
            } else {
                offline_message = Some("Failed to validate embedded OCI layout".to_string());
            }
        } else {
            offline_message = Some("Embedded OCI image not found (flashvm/data/oci)".to_string());
        }
    }

    dict.set_item("krunvm", krunvm_available)?;
    dict.set_item("buildah", buildah_available)?;
    dict.set_item("skopeo", skopeo_available)?;
    dict.set_item("kvm", kvm_available)?;
    dict.set_item("offline_mode", offline_available)?;
    dict.set_item("embedded_imported", embedded_imported)?;
    if !skopeo_available {
        dict.set_item("note", "skopeo not found; import will use buildah fallback and may be slower")?;
    }
    if let Some(msg) = offline_message { dict.set_item("offline_message", msg)?; }
    dict.set_item("ready", krunvm_available && buildah_available && kvm_available)?;

    Ok(dict.into())
}

#[pymodule]
#[pyo3(name = "_core")]
fn flashvm(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(run, m)?)?;
    m.add_function(wrap_pyfunction!(run_with_config, m)?)?;
    m.add_function(wrap_pyfunction!(prepare_image, m)?)?;
    m.add_function(wrap_pyfunction!(pip_prepare_image, m)?)?;
    m.add_function(wrap_pyfunction!(list_cached_images, m)?)?;
    m.add_function(wrap_pyfunction!(clear_cache, m)?)?;
    m.add_function(wrap_pyfunction!(doctor, m)?)?;
    m.add_function(wrap_pyfunction!(find_embedded_data_path, m)?)?;
    Ok(())
}
