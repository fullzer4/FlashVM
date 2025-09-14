use pyo3::prelude::*;
use pyo3::types::PyDict;


mod vmm;
mod types;
mod error;


use crate::types::{ImageHandle, RunOptions, RunResult};


#[pyfunction]
fn prepare_image<'py>(py: Python<'py>, meta: Bound<'py, PyDict>) -> PyResult<ImageHandle> {
    // Aviso se base="embedded" + packages (DEV only)
    // TODO(meta): o resolvedor de assets "embedded" deve viver no Python, e retornar paths prontos
    if let (Some(base_obj), Some(pkgs_obj)) = (meta.get_item("base")?, meta.get_item("packages")?) {
        let base: String = base_obj.extract()?;
        let pkgs: Vec<String> = pkgs_obj.extract().unwrap_or_default();
        if base == "embedded" && !pkgs.is_empty() {
            let warnings = pyo3::types::PyModule::import_bound(py, "warnings")?;
            let warn_type = py.get_type_bound::<pyo3::exceptions::PyRuntimeWarning>();
            warnings.call_method1(
                "warn",
                (
                    "DEV-only base. For production, prefer `flashvm build -p ...` to create a pinned OCI image.",
                    warn_type,
                ),
            )?;
        }
    }

    // Validar/transformar meta em ImageHandle
    ImageHandle::from_pydict(&meta)
}


#[pyfunction]
#[pyo3(signature = (image, code, opts=None))]
fn run<'py>(py: Python<'py>, image: &ImageHandle, code: &str, opts: Option<Bound<'py, PyDict>>) -> PyResult<RunResult> {
    let opts = RunOptions::from_py(opts.as_ref())?;
    let res = py
        .allow_threads(|| vmm::run::run_vm(image, code, &opts))
        .map_err(error::pyerr)?;
    Ok(res)
}


#[pymodule]
fn flashvm_native<'py>(_py: Python<'py>, m: Bound<'py, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(prepare_image, &m)?)?;
    m.add_function(wrap_pyfunction!(run, &m)?)?;
    m.add_class::<ImageHandle>()?;
    m.add_class::<RunOptions>()?;
    m.add_class::<RunResult>()?;
    Ok(())
}