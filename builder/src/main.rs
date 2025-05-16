use std::{
    fs,
    process::{Command, exit},
};

enum Arch {
    X86_64,
}

impl Arch {
    fn as_str(&self) -> &'static str {
        match self {
            Arch::X86_64 => "x86_64",
        }
    }
}

pub fn main() {
    let mut args = std::env::args();

    args.next().expect("program should be the first argument");

    let mut arch = Arch::X86_64;
    let mut rust_profile = "dev".to_string();
    let mut only_build = false;
    let mut iso = true;
    let mut bios = true;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "only" => {
                if !args.next().is_some_and(|x| x.as_str() == "build") {
                    eprintln!("expected 'build' after 'only'");
                    exit(1);
                }

                only_build = true;
            }
            "with" => {
                let Some(key) = args.next() else {
                    eprintln!("expected a key");
                    exit(1);
                };

                match key.as_str() {
                    "arch" => {
                        let Some(arg) = args.next() else {
                            eprintln!("expected an architecture");
                            exit(1);
                        };

                        arch = match arg.as_str() {
                            "x86_64" => Arch::X86_64,

                            _ => {
                                eprintln!("unknown architecture: {arg}");
                                exit(1);
                            }
                        }
                    }

                    "profile" => {
                        rust_profile = args.next().unwrap_or_else(|| {
                            eprintln!("expected a rust profile");
                            exit(1);
                        });
                    }

                    "hdd" => iso = false,
                    "bios" => bios = true,
                    "uefi" => bios = false,

                    _ => {
                        eprintln!("unknown key: {key}");
                        exit(1);
                    }
                }
            }

            _ => {
                eprintln!("unknown command: {arg}");
                exit(1);
            }
        }
    }

    if !fs::exists("limine").is_ok_and(|exists| exists) {
        Command::new("git")
            .args([
                "clone",
                "https://github.com/limine-bootloader/limine.git",
                "--branch=v9.x-binary",
                "--depth=1",
            ])
            .spawn()
            .unwrap()
            .wait()
            .unwrap();

        Command::new("make")
            .args(["-C", "limine"])
            .spawn()
            .unwrap()
            .wait()
            .unwrap();
    }

    let ovmf_code = "ovmf-code-".to_string() + arch.as_str() + ".fd";
    let ovmf_vars = "ovmf-vars-".to_string() + arch.as_str() + ".fd";

    if !bios && !fs::exists("ovmf").is_ok_and(|exists| exists) {
        fs::create_dir_all("ovmf").unwrap();

        Command::new("curl")
            .args([
                "-Lo",
                ovmf_code.as_str(),
                ("https://github.com/osdev0/edk2-ovmf-nightly/releases/latest/download/"
                    .to_string()
                    + ovmf_code.as_str())
                .as_str(),
            ])
            .spawn()
            .unwrap()
            .wait()
            .unwrap();

        Command::new("curl")
            .args([
                "-Lo",
                ovmf_vars.as_str(),
                ("https://github.com/osdev0/edk2-ovmf-nightly/releases/latest/download/"
                    .to_string()
                    + ovmf_vars.as_str())
                .as_str(),
            ])
            .spawn()
            .unwrap()
            .wait()
            .unwrap();
    }

    let rust_target = arch.as_str().to_string() + "-unknown-none";

    let rust_profile_subdir = match rust_profile.as_str() {
        "dev" => "debug",
        "release" => "release",

        _ => {
            println!("unknown rust profile: {rust_profile}");
            exit(1);
        }
    };

    let image_path = "fajr-".to_string() + arch.as_str() + if iso { ".iso" } else { ".hdd" };

    Command::new("cargo")
        .args([
            "build",
            "-p",
            "fajr_kernel",
            "--target",
            rust_target.as_str(),
            "--profile",
            rust_profile.as_str(),
        ])
        .env("RUSTFLAGS", "-C relocation-model=static")
        .spawn()
        .unwrap()
        .wait()
        .unwrap();

    fs::copy(
        "target/".to_string()
            + rust_target.as_str()
            + "/"
            + rust_profile_subdir
            + "/"
            + "fajr_kernel",
        "kernel/kernel",
    )
    .unwrap();

    if iso {
        if fs::exists("iso_root").is_ok_and(|exists| exists) {
            fs::remove_dir_all("iso_root").unwrap();
        }

        fs::create_dir_all("iso_root/boot").unwrap();
        fs::create_dir_all("iso_root/boot/limine").unwrap();
        fs::create_dir_all("iso_root/EFI/BOOT").unwrap();

        fs::copy("kernel/kernel", "iso_root/boot/kernel").unwrap();
        fs::copy("limine.conf", "iso_root/boot/limine.conf").unwrap();

        fs::copy(
            "limine/limine-bios.sys",
            "iso_root/boot/limine/limine-bios.sys",
        )
        .unwrap();
        fs::copy(
            "limine/limine-bios-cd.bin",
            "iso_root/boot/limine/limine-bios-cd.bin",
        )
        .unwrap();
        fs::copy(
            "limine/limine-uefi-cd.bin",
            "iso_root/boot/limine/limine-uefi-cd.bin",
        )
        .unwrap();
        fs::copy("limine/BOOTX64.EFI", "iso_root/EFI/BOOT/BOOTX64.EFI").unwrap();
        fs::copy("limine/BOOTIA32.EFI", "iso_root/EFI/BOOT/BOOTIA32.EFI").unwrap();

        Command::new("xorriso")
            .args([
                "-as",
                "mkisofs",
                "-b",
                "boot/limine/limine-bios-cd.bin",
                "--no-emul-boot",
                "-boot-load-size",
                "4",
                "-boot-info-table",
                "--efi-boot",
                "boot/limine/limine-uefi-cd.bin",
                "--efi-boot-part",
                "--efi-boot-image",
                "--protective-msdos-label",
                "iso_root",
                "-o",
                image_path.as_str(),
            ])
            .spawn()
            .unwrap()
            .wait()
            .unwrap();

        Command::new("./limine/limine")
            .args(["bios-install", image_path.as_str()])
            .spawn()
            .unwrap()
            .wait()
            .unwrap();

        fs::remove_dir_all("iso_root").unwrap();
    } else {
        if fs::exists(image_path.as_str()).is_ok_and(|exists| exists) {
            fs::remove_file(image_path.as_str()).unwrap();
        }

        Command::new("dd")
            .args([
                "if=/dev/zero",
                "bs=1M",
                "count=0",
                "seek=64",
                ("of=".to_string() + image_path.as_str()).as_str(),
            ])
            .spawn()
            .unwrap()
            .wait()
            .unwrap();

        Command::new("sgdisk")
            .args([image_path.as_str(), "-n", "1:2048", "-t", "1:ef00"])
            .spawn()
            .unwrap()
            .wait()
            .unwrap();

        Command::new("./limine/limine")
            .args(["bios-install", image_path.as_str()])
            .spawn()
            .unwrap()
            .wait()
            .unwrap();

        let image_path = image_path.clone() + "@@1M";

        Command::new("mformat")
            .args(["-i", image_path.as_str()])
            .spawn()
            .unwrap()
            .wait()
            .unwrap();

        Command::new("mmd")
            .args([
                "-i",
                image_path.as_str(),
                "::/EFI",
                "::/EFI/BOOT",
                "::/boot",
                "::/boot/limine",
            ])
            .spawn()
            .unwrap()
            .wait()
            .unwrap();

        Command::new("mcopy")
            .args(["-i", image_path.as_str(), "kernel/kernel", "::/boot"])
            .spawn()
            .unwrap()
            .wait()
            .unwrap();

        Command::new("mcopy")
            .args(["-i", image_path.as_str(), "limine.conf", "::/boot/limine"])
            .spawn()
            .unwrap()
            .wait()
            .unwrap();

        Command::new("mcopy")
            .args([
                "-i",
                image_path.as_str(),
                "limine/limine-bios.sys",
                "::/boot/limine",
            ])
            .spawn()
            .unwrap()
            .wait()
            .unwrap();

        Command::new("mcopy")
            .args([
                "-i",
                image_path.as_str(),
                "limine/BOOTX64.EFI",
                "::/EFI/BOOT",
            ])
            .spawn()
            .unwrap()
            .wait()
            .unwrap();

        Command::new("mcopy")
            .args([
                "-i",
                image_path.as_str(),
                "limine/BOOTIA32.EFI",
                "::/EFI/BOOT",
            ])
            .spawn()
            .unwrap()
            .wait()
            .unwrap();
    }

    if !only_build {
        let qemu_program = "qemu-system-".to_string() + arch.as_str();

        if bios {
            if iso {
                Command::new(qemu_program)
                    .args(["-M", "q35", "-cdrom", image_path.as_str(), "-boot", "d"])
                    .spawn()
                    .unwrap()
                    .wait()
                    .unwrap();
            } else {
                Command::new(qemu_program)
                    .args(["-M", "q35", "-hda", image_path.as_str()])
                    .spawn()
                    .unwrap()
                    .wait()
                    .unwrap();
            }
        } else {
            Command::new(qemu_program)
                .args([
                    "-M",
                    "q35",
                    "-drive",
                    ("if=pflash,unit=0,format=raw,file=ovmf/".to_string()
                        + ovmf_code.as_str()
                        + ",readonly=on")
                        .as_str(),
                    "-drive",
                    ("if=pflash,unit=1,format=raw,file=ovmf/".to_string() + ovmf_vars.as_str())
                        .as_str(),
                    if iso { "-cdrom" } else { "-hda" },
                    image_path.as_str(),
                ])
                .spawn()
                .unwrap()
                .wait()
                .unwrap();
        }
    }
}
