use anyhow::Result;
use linux_loader::loader::{Elf, KernelLoader};
use linux_loader::cmdline::Cmdline;
use vm_memory::{GuestMemoryMmap, GuestAddress, Bytes};
use std::path::Path;


pub struct BootInfo { pub entry: GuestAddress, pub cmdline_addr: GuestAddress }


pub fn load_kernel(gm: &GuestMemoryMmap, kernel_path: &Path, cmdline: &str) -> Result<BootInfo> {
    // TODO(boot): trocar para loader de bzImage para x86_64 (mais simples) e preparar boot params
    //  - linux_loader::loader::BzImage::load(...)
    //  - calcular/usar cmdline_addr conforme protocolo
    //  - setar registradores da vCPU conforme retorno do loader ou layout esperado
    let mut kernel_image = std::fs::File::open(kernel_path)?;
    let k = Elf::load(gm, None, &mut kernel_image, None)?;


    // Cmdline
    let mut cmd = Cmdline::new(1024)?;
    cmd.insert_str(cmdline)?;
    // Escrever cmdline NUL-terminada na memória convidada
    let cstr = cmd.as_cstring()?;
    // TODO(boot): calcular cmdline_addr conforme layout real (abaixo de 1MiB, alinhamento conforme protocolo)
    let cmdline_addr = GuestAddress(0x20000);
    gm.write_slice(cstr.to_bytes_with_nul(), cmdline_addr)?;


    Ok(BootInfo { entry: k.kernel_load, cmdline_addr })
}