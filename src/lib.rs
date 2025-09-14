use pyo3::prelude::*;
use pyo3::types::PyDict;


mod vmm;
mod types;
mod error;


use crate::types::{ImageHandle, RunOptions, RunResult};


#[pyfunction]
fn prepare_image<'py>(_py: Python<'py>, meta: Bound<'py, PyDict>) -> PyResult<ImageHandle> {
    // TODO: validar dict (kernel, rootfs_img, initrd opcional, cache_key)
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
    m.add_function(wrap_pyfunction_bound!(prepare_image, &m)?)?;
    m.add_function(wrap_pyfunction_bound!(run, &m)?)?;
    m.add_class::<ImageHandle>()?;
    m.add_class::<RunOptions>()?;
    m.add_class::<RunResult>()?;
    Ok(())
}