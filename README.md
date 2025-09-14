# flashvm

**Python‑first microVM runner** built on **rust‑vmm** to execute Python code inside isolated VMs from a **prepared OCI image** (or an embedded cached image).

> Minimal Python API:
>
> - `prepare_image(meta: dict) -> ImageHandle`
> - `run(image: ImageHandle, code: str, opts: dict | None) -> RunResult`

---

## Features

- **Strong isolation** via KVM: read‑only rootfs + per‑run writable workspace.
- **Prepared image cache**: install `pip` deps during *prepare* and reuse across runs.
- **No external daemon**: a minimal Rust VMM (rust‑vmm crates), surfaced via PyO3.
- **Linux‑only** (x86_64 and aarch64 with KVM).

## Requirements

- Linux with **KVM** enabled (`/dev/kvm` accessible).
- Guest kernel (`vmlinux`/`bzImage`) and a minimal **guest init** script (you can ship them under `python/flashvm/assets/`).
- Python ≥ 3.8.

> Tip: add your user to the `kvm` group (and re‑login) to avoid running as root.

---

## Quickstart

### 1) Prepare image (cache)

The *prepare* phase happens on the Python side: you resolve the OCI image (or use an embedded one), install `pip` deps, and hand paths to the native module.

```python
import flashvm as fv

img = fv.prepare_image({
    "kernel": "/path/to/vmlinux",
    "rootfs_img": "/path/to/rootfs.img",  # ext4, squashfs, etc.
    # "initrd": "/path/to/initrd",        # optional
    # "cache_key": "python312-alpine+numpy1.26",
})
```

> Suggestion: keep a stable *cache key* per base OCI + installed `pip` deps.

### 2) Run isolated code

```python
res = fv.run(
    img,
    code="import platform; print('hello', platform.python_version())",
    opts={
        "cpus": 1,
        "mem_mb": 512,
        "timeout_ms": 15000,
        "output": "diff",   # diff|all|none|paths
    },
)
print(res.exit_status)
print(res.stdout)
print("artifacts:", res.outputs_dir)
```

> `run` creates a clean **workspace** (RW disk) per execution and returns `stdout/stderr`, `exit_status`, and a directory with collected artifacts per the chosen output mode.

---

## How it works (technical sketch)

- **kvm-ioctls / kvm-bindings**: create VM and vCPUs safely.
- **vm-memory**: manage guest physical memory mappings.
- **linux-loader**: load kernel + cmdline (and `initrd` if present).
- **vm-superio (serial)**: `ttyS0` console for logs / `stdout`.
- **vm-virtio (blk)**: two virtio‑blk disks — `vda` (rootfs, RO) and `vdb` (workspace, RW).
- **event-manager**: epoll‑based event loop for device I/O.
- **vmm-sys-util**: utilities (`EventFd`, ioctl helpers).
- **acpi_tables** (x86) / **vm-fdt** (arm64): boot tables when needed.

Networking is **off by default**. Enable explicitly if your workload requires it.

---

## Prepare best practices

- Build `rootfs.img` with **pip deps preinstalled** (inside a chroot/guest) to avoid ABI mismatches.
- Keep rootfs **read‑only** and a separate **workspace** for easy artifact collection and diffs.
- Prefer **wheels** (`pip --only-binary :all:`) in the prepare phase to reduce install time.

---

## Troubleshooting

- **`/dev/kvm` permission denied**: ensure your user is in the `kvm` group; re‑login after adding.
- **Missing kernel/init**: place assets under `python/flashvm/assets/` or pass absolute paths to `prepare_image`.
- **No console output**: check `console=ttyS0` in the kernel cmdline and that the serial device is mapped.
- **Timeouts**: increase `timeout_ms` or verify your `init` runs and exits correctly.

---

## Roadmap (high level)

- Full virtio‑blk workflow (diff, path filters, artifact compression).
- Optional `seccomp` hardening of the VMM process.
- Optional "direct OCI" mode (pull + unpack) behind a build feature.

---

## License

Apache‑2.0 OR BSD‑3‑Clause (dual license), in line with the rust‑vmm ecosystem.
