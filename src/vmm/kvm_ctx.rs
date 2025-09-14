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
    let mut vcpus = Vec::new();
    for _i in 0..cpus { vcpus.push(vm.create_vcpu(0)?); /* TODO: regs/sregs/APIC por vCPU */ }
        // TODO: IRQ routing, pit/clk mínimos se necessário
        Ok(Self { kvm, vm, vcpus })
}}
