<p align="center">
    <h1 align="center"> Fajr </h1>
</p>

<p align="center">
    <h4 align="center"> A modern and elegant operating system for programmers </h4>
</p>

### Makefile targets:

- Running `make all` will compile the kernel (from the `kernel/` directory) and then generate a bootable ISO image.

- Running `make all-hdd` will compile the kernel and then generate a raw image suitable to be flashed onto a USB stick or hard drive/SSD.

- Running `make run` will build the kernel and a bootable ISO (equivalent to make all) and then run it using `qemu` (if installed).

- Running `make run-hdd` will build the kernel and a raw HDD image (equivalent to make all-hdd) and then run it using `qemu` (if installed).

>[!NOTE]
>The `run-uefi` and `run-hdd-uefi` targets are equivalent to their non `-uefi` counterparts except that they boot `qemu` using a UEFI-compatible firmware.
