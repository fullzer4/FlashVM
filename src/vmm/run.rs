use anyhow::Result;
use crate::types::{ImageHandle, RunOptions, RunResult};
use super::{kvm_ctx::KvmContext, memory::GuestMem, boot, event_loop::VmLoop};
use std::path::PathBuf;


pub fn run_vm(image: &ImageHandle, code: &str, opts: &RunOptions) -> Result<RunResult> {
    // 1) memória + KVM
    let gm = GuestMem::create(opts.mem_mb)?;
    let _kvmx = KvmContext::new(opts.cpus)?;
    // TODO(kvm): registrar regiões de memória do gm no KVM (set_user_memory_region)


    // 2) cmdline mínima para guest-init
    let cmdline = format!(
        "console=ttyS0 root=/dev/vda ro init=/sbin/init FLASHVM_MODE=run FLASHVM_CODE_LEN={}",
        code.len()
    );


    // 3) carregar kernel
    let kernel_path = PathBuf::from(&image.kernel);
    let _boot = boot::load_kernel(&gm.mem, &kernel_path, &cmdline)?;
    // TODO(boot): trocar para BzImage (x86_64) e preparar boot params/registradores
    // TODO(vcpu): aplicar CPUID/MSRs/SREGs/REGs conforme bootinfo


    // 4) anexar devices (serial + vda=rootfs RO + vdb=workspace RW)
    // TODO(devices): fase 1: capturar serial via KVM_EXIT_IO (0x3f8)
    // TODO(devices): fase 2: vm_superio::serial::Serial + virtio-blk
    // TODO(fs): criar workspace.img (sparse + ext4) e gravar código/entrypoint


    // 5) loop até saída/timeout
    let mut vml = VmLoop::new()?;
    let status = vml.run_until_exit(opts.timeout_ms)?;


    // 6) coletar stdout/stderr + artefatos do workspace
    // TODO(coleta): montar workspace.img e empacotar conforme output_mode (diff/all/paths)
    Ok(RunResult { stdout: String::new(), stderr: String::new(), exit_status: status, outputs_dir: None })
}