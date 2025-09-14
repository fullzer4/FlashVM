use anyhow::Result;
use linux_loader::loader::{Elf, KernelLoader};
use linux_loader::cmdline::Cmdline;
use vm_memory::{GuestMemoryMmap, GuestAddress};
use std::path::Path;


pub struct BootInfo { pub entry: GuestAddress, pub cmdline_addr: GuestAddress }


pub fn load_kernel(gm: &GuestMemoryMmap, kernel_path: &Path, cmdline: &str) -> Result<BootInfo> {
    // TODO: suportar bzImage; por ora, ELF vmlinux
    let mut kernel_image = std::fs::File::open(kernel_path)?;
    let k = Elf::load(gm, None, &mut kernel_image, None)?;


    // Cmdline
    let mut cmd = Cmdline::new(1024)?;
    cmd.insert_str(cmdline)?;
    // TODO: alocar e escrever cmdline em memória convidada
    let cmdline_addr = GuestAddress(0x20000); // placeholder


    Ok(BootInfo { entry: k.kernel_load, cmdline_addr })
}