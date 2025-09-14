use pyo3::exceptions::PyRuntimeError;
use pyo3::PyErr;


pub fn pyerr<E: std::fmt::Display>(e: E) -> PyErr { PyRuntimeError::new_err(e.to_string()) }