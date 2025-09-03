use pyo3::prelude::*;
use std::path::PathBuf;

#[pyfunction]
pub fn find_embedded_data_path() -> PyResult<String> {
    Python::with_gil(|py| {
        let importlib_resources = py.import_bound("importlib.resources")?;
        let pathlib = py.import_bound("pathlib")?;

        let package_files = importlib_resources
            .getattr("files")?
            .call1(("flashvm",))?;

        let data_path = package_files.call_method1("joinpath", ("data",))?;
        let oci_path = data_path.call_method1("joinpath", ("oci",))?;

        let as_file = importlib_resources.getattr("as_file")?;
        let context_manager = as_file.call1((oci_path,))?;
        let enter_method = context_manager.getattr("__enter__")?;
        let oci_real_path = enter_method.call0()?;

        let path_obj = pathlib.getattr("Path")?.call1((oci_real_path,))?;
        let path_str: String = path_obj.call_method0("__str__")?.extract()?;

        let oci_layout_file = path_obj.call_method1("joinpath", ("oci-layout",))?;
        let index_file = path_obj.call_method1("joinpath", ("index.json",))?;
        let blobs_dir = path_obj.call_method1("joinpath", ("blobs",))?;
        let blobs_sha256 = blobs_dir.call_method1("joinpath", ("sha256",))?;

        let oci_layout_exists: bool = oci_layout_file.call_method0("exists")?.extract()?;
        let index_exists: bool = index_file.call_method0("exists")?.extract()?;
        let blobs_exists: bool = blobs_dir.call_method0("exists")?.extract()?;
        let blobs_sha256_exists: bool = blobs_sha256.call_method0("exists")?.extract()?;

        let exit_method = context_manager.getattr("__exit__")?;
        exit_method.call1((py.None(), py.None(), py.None()))?;

        if oci_layout_exists && index_exists && blobs_exists && blobs_sha256_exists {
            Ok(format!("oci:{}:python-basic", path_str))
        } else {
            Err(pyo3::exceptions::PyRuntimeError::new_err(format!(
                "Invalid OCI structure at {}: oci-layout={}, index.json={}, blobs={}, blobs/sha256={}",
                path_str, oci_layout_exists, index_exists, blobs_exists, blobs_sha256_exists
            )))
        }
    })
}

pub struct WheelResources;

impl WheelResources {
    pub fn find_embedded_data_path(_py: Python) -> PyResult<Option<PathBuf>> {
        match find_embedded_data_path() {
            Ok(oci_ref) => {
                if let Some((path_part, _tag)) = oci_ref
                    .strip_prefix("oci:")
                    .and_then(|s| s.rsplit_once(':'))
                {
                    Ok(Some(PathBuf::from(path_part)))
                } else {
                    Ok(None)
                }
            }
            Err(_) => Ok(None),
        }
    }

    pub fn check_embedded_image_available(_py: Python, _tag: &str) -> PyResult<bool> {
        match find_embedded_data_path() {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}
