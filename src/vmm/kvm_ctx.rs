use anyhow::Result;
use kvm_ioctls::{Kvm, VmFd, VcpuFd};


pub struct KvmContext {
pub kvm: Kvm,
pub vm: VmFd,
pub vcpus: Vec<VcpuFd>,
}


impl KvmContext {
pub fn new(cpus: u8) -> Result<Self> {
    let kvm = Kvm::new()?;
    let vm = kvm.create_vm()?;
    // TODO(kvm): registrar regiões de memória do convidado no KVM (set_user_memory_region)
    //  - Iterar GuestMemoryMmap no call-site (run.rs) ou expor API aqui para registrar.
    //  - Ref: https://docs.rs/kvm-ioctls/latest/kvm_ioctls/struct.VmFd.html#method.set_user_memory_region

    let mut vcpus = Vec::new();
    for i in 0..cpus { vcpus.push(vm.create_vcpu(i.into())?); /* TODO: regs/sregs/APIC por vCPU */ }
    // TODO(vcpu): configurar CPUID suportado, MSRs, SREGs e REGs
    //  - get_supported_cpuid → set_cpuid2
    //  - get_sregs → ajustar (modo de operação) → set_sregs
    //  - set_regs conforme loader (RIP/RSP/RFLAGS)
    //  - Ref: https://docs.rs/kvm-ioctls/latest/kvm_ioctls/struct.VcpuFd.html
        // TODO: IRQ routing, pit/clk mínimos se necessário
        Ok(Self { kvm, vm, vcpus })
}}
