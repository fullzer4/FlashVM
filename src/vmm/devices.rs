use anyhow::Result;


pub struct BlockSpec { pub path: String, pub read_only: bool }


pub struct DevicesCfg {
    pub console_stdio: bool,
    pub rootfs: BlockSpec,
    pub workspace: Option<BlockSpec>,
}


pub fn attach_devices(_cfg: &DevicesCfg) -> Result<()> {
    // TODO(devices/phase1): implementar console via KVM_EXIT_IO (porta 0x3f8) no loop do vCPU
    //  - Capturar writes no porto THR (0x3f8) e acumular em stdout.
    //  - Sem device model completo, suficiente para logs do guest.

    // TODO(devices/phase2): usar vm_superio::serial::Serial e despachar PIO para o device
    //  - Registrar no event loop e conectar a um Write do host para stdout.
    //  - Ref: https://docs.rs/vm-superio/latest/vm_superio/serial/struct.Serial.html

    // TODO(block): virtio-blk (vda=rootfs RO, vdb=workspace RW)
    //  - Adicionar dependência do crate virtio-blk (repo vm-virtio) quando implementar.
    //  - Registrar queues/eventfds no event-manager.
    //  - Ref: https://github.com/rust-vmm/vm-virtio
    Ok(())
}