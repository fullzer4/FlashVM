use anyhow::Result;


pub struct VmLoop;


impl VmLoop {
    pub fn new() -> Result<Self> { Ok(Self) }
    pub fn run_until_exit(&mut self, _timeout_ms: Option<u64>) -> Result<i32> {
    // TODO(loop/phase1): loop de execução do vCPU usando kvm_run
    //  - chamar vcpu.run() em ciclo
    //  - tratar KVM_EXIT_IO (porta 0x3f8) → stdout
    //  - tratar KVM_EXIT_HLT → encerrar com status 0
    //  - tratar falhas (FAIL_ENTRY/SHUTDOWN) → erro ou status
    //  - respeitar timeout_ms

    // TODO(loop/phase2): substituir por event-manager
    //  - registrar serial/virtio-blk como subscribers
    //  - integrar I/O e interrupções conforme backend
        Ok(0)
    }
}