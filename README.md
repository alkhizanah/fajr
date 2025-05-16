<p align="center">
    <h1 align="center"> Fajr </h1>
</p>

<p align="center">
    <h4 align="center"> A modern and elegant operating system for programmers </h4>
</p>

### Building from source

- Running `cargo run` will build the kernel and a bootable ISO and then run it using `qemu` (if installed).

- Running `cargo run -- with hdd` will build the kernel and a raw HDD image and then run it using `qemu` (if installed).

- Running `cargo run -- only build` will only build the kernel and a bootable ISO.

- Running `cargo run -- only build with hdd` will only build the kernel and a raw HDD image.

> [!NOTE]
> Adding `with uefi` to each command will build with a UEFI-compatible firmware.
