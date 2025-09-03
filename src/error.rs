use pyo3::exceptions::PyRuntimeError;
use std::fmt;

#[derive(Debug)]
pub enum VMError {
    ImageResolution(String),
    VMConfiguration(String),
    Execution(String),
    IO(std::io::Error),
    Timeout(String),
    MissingDependency(String),
    Cache(String),
}

impl fmt::Display for VMError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VMError::ImageResolution(msg) => write!(f, "Image resolution error: {}", msg),
            VMError::VMConfiguration(msg) => write!(f, "VM configuration error: {}", msg),
            VMError::Execution(msg) => write!(f, "Execution error: {}", msg),
            VMError::IO(err) => write!(f, "I/O error: {}", err),
            VMError::Timeout(msg) => write!(f, "Timeout: {}", msg),
            VMError::MissingDependency(dep) => write!(f, "Missing dependency: {}", dep),
            VMError::Cache(msg) => write!(f, "Cache error: {}", msg),
        }
    }
}

impl std::error::Error for VMError {}

impl From<std::io::Error> for VMError {
    fn from(err: std::io::Error) -> Self { VMError::IO(err) }
}

impl From<anyhow::Error> for VMError {
    fn from(err: anyhow::Error) -> Self { VMError::Execution(err.to_string()) }
}

pub type PyVMError = PyRuntimeError;
