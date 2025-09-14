use anyhow::Result;
use vm_memory::{GuestMemoryMmap, GuestAddress};


pub struct GuestMem { pub mem: GuestMemoryMmap }


impl GuestMem {
    pub fn create(mem_mb: u32) -> Result<Self> {
    let size: usize = (mem_mb as usize)
        .checked_mul(1024 * 1024)
        .ok_or_else(|| anyhow::anyhow!("mem_mb too large"))?;
    let gm = GuestMemoryMmap::from_ranges(&[(GuestAddress(0), size)])?;
    Ok(Self { mem: gm })
}}