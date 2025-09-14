use serde::{Deserialize, Serialize};
use pyo3::prelude::*;
use pyo3::types::PyDict;


#[pyclass]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ImageHandle {
    #[pyo3(get)] pub kernel: String,
    #[pyo3(get)] pub rootfs_img: String,
    #[pyo3(get)] pub initrd: Option<String>,
    #[pyo3(get)] pub cache_key: Option<String>,
}


impl ImageHandle {
    pub fn from_pydict<'py>(d: &pyo3::Bound<'py, PyDict>) -> PyResult<Self> {
        // TODO: validação robusta + normalização de paths
        Ok(Self {
            kernel: d
                .get_item("kernel")?
                .ok_or_else(|| pyo3::exceptions::PyValueError::new_err("missing kernel"))?
                .extract()?,
            rootfs_img: d
                .get_item("rootfs_img")?
                .ok_or_else(|| pyo3::exceptions::PyValueError::new_err("missing rootfs_img"))?
                .extract()?,
            initrd: d.get_item("initrd")?.map(|v| v.extract()).transpose()?,
            cache_key: d.get_item("cache_key")?.map(|v| v.extract()).transpose()?,
        })
    }
}


#[pyclass]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RunOptions {
    #[pyo3(get)] pub cpus: u8,
    #[pyo3(get)] pub mem_mb: u32,
    #[pyo3(get)] pub timeout_ms: Option<u64>,
    #[pyo3(get)] pub output_mode: String, // "diff" | "all" | "none" | "paths"
}


impl RunOptions {
    pub fn from_py<'py>(d: Option<&pyo3::Bound<'py, PyDict>>) -> PyResult<Self> {
        let cpus: u8 = match d {
            Some(x) => x.get_item("cpus")?.map(|v| v.extract().unwrap_or(1)).unwrap_or(1),
            None => 1,
        };
        let mem_mb: u32 = match d {
            Some(x) => x.get_item("mem_mb")?.map(|v| v.extract().unwrap_or(512)).unwrap_or(512),
            None => 512,
        };
        let timeout_ms: Option<u64> = match d {
            Some(x) => x
                .get_item("timeout_ms")?
                .map(|v| v.extract().unwrap_or(0))
                .filter(|&t| t > 0),
            None => None,
        };
        let output_mode: String = match d {
            Some(x) => x
                .get_item("output")?
                .map(|v| v.extract().unwrap_or("diff".to_string()))
                .unwrap_or("diff".into()),
            None => "diff".into(),
        };
        Ok(Self { cpus, mem_mb, timeout_ms, output_mode })
    }
}


#[pyclass]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RunResult {
    #[pyo3(get)] pub stdout: String,
    #[pyo3(get)] pub stderr: String,
    #[pyo3(get)] pub exit_status: i32,
    #[pyo3(get)] pub outputs_dir: Option<String>,
}