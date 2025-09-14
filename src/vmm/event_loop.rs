use anyhow::Result;


pub struct VmLoop;


impl VmLoop {
    pub fn new() -> Result<Self> { Ok(Self) }
    pub fn run_until_exit(&mut self, _timeout_ms: Option<u64>) -> Result<i32> {
        // TODO: dirigir o loop com event-manager
        Ok(0)
    }
}