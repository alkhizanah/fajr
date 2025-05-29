use std::{
    ffi::OsStr,
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

fn exec<C>(command: C)
where
    C: AsRef<str>,
{
    let mut command = command.as_ref().split_whitespace();

    let program = command.next().unwrap();

    Command::new(program)
        .args(command)
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
}

fn exece<C, E, K, V>(command: C, env: E)
where
    C: ToString,
    E: Iterator<Item = (K, V)>,
    K: AsRef<OsStr>,
    V: AsRef<OsStr>,
{
    let command = command.to_string();
    let mut command = command.split_whitespace();

    let program = command.next().unwrap();

    Command::new(program)
        .args(command)
        .envs(env)
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
}

pub fn main() {
    let mut args = std::env::args();

    args.next().expect("program should be the first argument");

    let mut arch = Arch::X86_64;
    let mut rust_profile = "dev".to_string();
    let mut only_build = false;
    let mut iso = true;
    let mut bios = true;
    let mut clippy = false;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "only" => {
                if args.next().is_none_or(|x| x.as_str() != "build") {
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
                    "clippy" => clippy = true,

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

    let rust_target = arch.as_str().to_string() + "-unknown-none";

    let rust_profile_subdir = match rust_profile.as_str() {
        "dev" => "debug",
        "release" => "release",

        _ => {
            println!("unknown rust profile: {rust_profile}");
            exit(1);
        }
    };

    if clippy {
        exece(
            format!("cargo clippy -p fajr_kernel --target {rust_target}"),
            [("RUSTFLAGS", "-C relocation-model=static")].into_iter(),
        );

        return;
    }

    if !fs::exists("limine").is_ok_and(|exists| exists) {
        exec(
            "git clone https://github.com/limine-bootloader/limine.git --branch=v9.x-binary --depth=1",
        );

        exec("make -C limine");
    }

    let ovmf_code = "ovmf-code-".to_string() + arch.as_str() + ".fd";
    let ovmf_vars = "ovmf-vars-".to_string() + arch.as_str() + ".fd";

    if !bios && !fs::exists("ovmf").is_ok_and(|exists| exists) {
        fs::create_dir_all("ovmf").unwrap();

        exec(format!(
            "curl -Lo {ovmf_code} https://github.com/osdev0/edk2-ovmf-nightly/releases/latest/download/{ovmf_code}"
        ));

        exec(format!(
            "curl -Lo {ovmf_vars} https://github.com/osdev0/edk2-ovmf-nightly/releases/latest/download/{ovmf_vars}"
        ));
    }

    let image_path = "fajr-".to_string() + arch.as_str() + if iso { ".iso" } else { ".hdd" };

    exece(
        format!("cargo build -p fajr_kernel --target {rust_target} --profile {rust_profile}"),
        [("RUSTFLAGS", "-C relocation-model=static")].into_iter(),
    );

    fs::copy(
        format!("target/{rust_target}/{rust_profile_subdir}/fajr_kernel"),
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

        exec(format!(
            "xorriso -as mkisofs -b boot/limine/limine-bios-cd.bin --no-emul-boot -boot-load-size 4 -boot-info-table
            --efi-boot boot/limine/limine-uefi-cd.bin --efi-boot-part --efi-boot-image --protective-msdos-label iso_root -o {image_path}"
        ));

        exec(format!("./limine/limine bios-install {image_path}"));

        fs::remove_dir_all("iso_root").unwrap();
    } else {
        if fs::exists(image_path.as_str()).is_ok_and(|exists| exists) {
            fs::remove_file(image_path.as_str()).unwrap();
        }

        exec(format!(
            "dd if=/dev/zero bs=1M count=0 seek=64 of={image_path}"
        ));
        exec(format!("sgdisk {image_path} -n 1:2048 -t 1:ef00"));
        exec(format!("mformat -i {image_path}@@1M"));
        exec(format!(
            "mmd -i {image_path}@@1M ::/EFI ::/EFI/BOOT ::/boot ::/boot/limine"
        ));
        exec(format!("mcopy -i {image_path}@@1M kernel/kernel ::/boot"));
        exec(format!(
            "mcopy -i {image_path}@@1M limine.conf ::/boot/limine"
        ));
        exec(format!(
            "mcopy -i {image_path}@@1M limine/limine-bios.sys ::/boot/limine"
        ));
        exec(format!(
            "mcopy -i {image_path}@@1M limine/BOOTX64.EFI ::/EFI/BOOT"
        ));
        exec(format!(
            "mcopy -i {image_path}@@1M limine/BOOTIA32.EFI ::/EFI/BOOT"
        ));
        exec(format!("./limine/limine bios-install {image_path}"));
    }

    if !only_build {
        let qemu_program = "qemu-system-".to_string() + arch.as_str();

        if bios {
            if iso {
                exec(format!(
                    "{qemu_program} -m 4G -M q35 -cdrom {image_path} -boot d"
                ));
            } else {
                exec(format!("{qemu_program} -m 4G -M q35 -hda {image_path}"));
            }
        } else {
            exec(format!(
                "{qemu_program} -M q35 -drive if=pflash,unit=0,format=raw,file=ovmf/{ovmf_code},readonly=on
                -drive if=pflash,unit=1,format=raw,file=ovmf/{ovmf_vars} {} {image_path}",
                if iso { "-cdrom" } else { "-hda" }
            ));
        }
    }
}
