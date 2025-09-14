use anyhow::Result;


pub struct BlockSpec { pub path: String, pub read_only: bool }


pub struct DevicesCfg {
    pub console_stdio: bool,
    pub rootfs: BlockSpec,
    pub workspace: Option<BlockSpec>,
}


pub fn attach_devices(_cfg: &DevicesCfg) -> Result<()> {
    // TODO: instanciar vm_superio::Serial (ttyS0) e mapear em PIO/MMIO
    // TODO: instanciar virtio-blk (rootfs/workspace) e registrar no event-manager
    Ok(())
}