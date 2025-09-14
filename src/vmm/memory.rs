use anyhow::Result;
use vm_memory::{GuestMemoryMmap, GuestAddress};


pub struct GuestMem { pub mem: GuestMemoryMmap }


impl GuestMem {
    pub fn create(mem_mb: u32) -> Result<Self> {
    let size_u64 = (mem_mb as u64) * 1024 * 1024;
    let size: usize = size_u64.try_into().map_err(|_| anyhow::anyhow!("mem size too big"))?;
    let gm = GuestMemoryMmap::from_ranges(&[(GuestAddress(0), size)])?;
    Ok(Self { mem: gm })
}}