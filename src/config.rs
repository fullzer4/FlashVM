use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

/// Main configuration for running Python code in a microVM
#[derive(Debug, Clone)]
pub struct VMConfig {
    /// Image reference (None = use embedded image; Some = docker://, containers-storage:, simple name, oci:, dir:, oci-archive:)
    pub image: Option<String>,
    /// CPUs
    pub cpus: u32,
    /// Memory (MB)
    pub memory_mb: u32,
    /// Environment variables
    pub env: HashMap<String, String>,
    /// Working directory in the guest (top-level, e.g., /work)
    pub workdir: String,
    /// Overall timeout
    pub timeout: Duration,
    /// Enable network
    pub network: bool,
    /// Ports (host:guest) when network=true
    pub ports: Vec<(u16, u16)>,
    /// Extra Python args (e.g., -u)
    pub python_args: Vec<String>,
    /// Max size in bytes to inline artifacts
    pub max_bytes_inline: u64,
}

impl Default for VMConfig {
    fn default() -> Self {
        Self {
            image: None,
            cpus: 1,
            memory_mb: 512,
            env: HashMap::new(),
            workdir: "/work".to_string(),
            timeout: Duration::from_secs(30),
            network: false,
            ports: vec![],
            python_args: vec!["-u".to_string()],
            max_bytes_inline: 1024 * 1024, // 1MB
        }
    }
}

/// Input files
#[derive(Debug, Clone)]
pub struct FileInput {
    pub host_path: PathBuf,
    pub guest_path: String, // relative to /work/in/
}

/// Output patterns (glob under /work/out)
#[derive(Debug, Clone)]
pub struct FileOutput {
    pub pattern: String, // e.g.: "out/*.parquet"
}

/// Execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub execution_time: Duration,
    pub artifacts: Vec<Artifact>,
    pub image_used: String,
}

/// Collected artifact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    pub guest_path: String,
    pub host_path: PathBuf,
    pub size_bytes: u64,
    pub content: Option<Vec<u8>>,
}

/// Capture mode (future use)
#[derive(Debug, Clone)]
pub enum CaptureMode {
    Stdio,
    JsonLines,
    Both,
}

/// Local cache/config
#[derive(Debug, Clone)]
pub struct CacheConfig {
    pub cache_dir: String,
    pub cache_ttl_seconds: u64,
    pub max_cache_size_mb: u64,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            cache_dir: format!(
                "{}/.cache/flashvm",
                std::env::var("HOME").unwrap_or("/tmp".to_string())
            ),
            cache_ttl_seconds: 24 * 3600,
            max_cache_size_mb: 1024,
        }
    }
}
