#include <stdbool.h>
#include <stdio.h>

#define NOB_IMPLEMENTATION
#include "nob.h"

typedef enum { x86_64 } Arch;

const char *arch_to_string(Arch arch) {
    switch (arch) {
    case x86_64:
        return "x86_64";
    }
}

Nob_Cmd cmd = {0};

int main(int argc, char **argv) {
    NOB_GO_REBUILD_URSELF(argc, argv);

    nob_shift(argv, argc);

    Arch arch = x86_64;

    bool only_build = false;
    bool only_ccls = false;
    bool iso = true;
    bool bios = true;

    while (argc) {
        const char *arg = nob_shift(argv, argc);

        if (strcmp(arg, "only") == 0) {
            if (!argc) {
                fprintf(stderr, "expected a key after 'only'\n");

                return 1;
            }

            const char *key = nob_shift(argv, argc);

            if (strcmp(key, "build") == 0) {
                only_build = true;
            } else if (strcmp(key, "ccls") == 0) {
                only_ccls = true;
            } else {
                fprintf(stderr, "unknown key after 'only': %s\n", key);

                return 1;
            }
        } else if (strcmp(arg, "with") == 0) {
            if (!argc) {
                fprintf(stderr, "expected a key after 'with'\n");

                return 1;
            }

            const char *key = nob_shift(argv, argc);

            if (strcmp(key, "arch") == 0) {
                if (!argc) {
                    fprintf(stderr, "expected an architecture after 'arch'\n");

                    return 1;
                }

                const char *arch_str = nob_shift(argv, argc);

                if (strcmp(arch_str, "x86_64") == 0) {
                    arch = x86_64;
                } else {
                    fprintf(stderr, "unsupported architecture: %s\n", arch_str);

                    return 1;
                }
            } else if (strcmp(key, "hdd") == 0) {
                iso = false;
            } else if (strcmp(key, "bios") == 0) {
                bios = true;
            } else if (strcmp(key, "uefi") == 0) {
                bios = false;
            } else {
                fprintf(stderr, "unknown key after 'with': %s\n", key);

                return 1;
            }
        } else {
            fprintf(stderr, "unknown command: %s\n", arg);

            return 1;
        }
    }

    if (!nob_file_exists("limine")) {
        nob_cmd_append(&cmd, "git", "clone");
        nob_cmd_append(&cmd, "https://codeberg.org/Limine/Limine.git",
                       "--branch=v10.x-binary", "--depth=1", "limine");
        nob_cmd_run(&cmd);

        nob_cmd_append(&cmd, "make", "-C", "limine");
        nob_cmd_run(&cmd);
    }

    if (!nob_file_exists("limine-protocol")) {
        nob_cmd_append(&cmd, "git", "clone");
        nob_cmd_append(&cmd, "https://codeberg.org/Limine/limine-protocol.git",
                       "--depth=1");
        nob_cmd_run(&cmd);
    }

    const char *ovmf_code =
        nob_temp_sprintf("ovmf-code-%s.fd", arch_to_string(arch));
    const char *ovmf_vars =
        nob_temp_sprintf("ovmf-vars-%s.fd", arch_to_string(arch));

    if (!bios && !nob_file_exists("ovmf")) {
        nob_mkdir_if_not_exists("ovmf");

        nob_cmd_append(
            &cmd, "curl", "-Lo", nob_temp_sprintf("ovmf/%s", ovmf_code),
            nob_temp_sprintf("https://github.com/osdev0/edk2-ovmf-nightly/"
                             "releases/latest/download/%s",
                             ovmf_code));
        nob_cmd_run(&cmd);

        nob_cmd_append(
            &cmd, "curl", "-Lo", nob_temp_sprintf("ovmf/%s", ovmf_vars),
            nob_temp_sprintf("https://github.com/osdev0/edk2-ovmf-nightly/"
                             "releases/latest/download/%s",
                             ovmf_vars));
        nob_cmd_run(&cmd);
    }

    char *image_name = nob_temp_sprintf("fajr-%s.%s", arch_to_string(arch),
                                        iso ? "iso" : "hdd");

    nob_cmd_append(&cmd, "clang", "-std=gnu23");

    nob_cc_flags(&cmd);

    if (!only_ccls) {
        nob_cc_inputs(&cmd, "kernel/src/main.c", "kernel/src/memory.c",
                      "kernel/src/terminal.c", "kernel/src/screen.c",
                      "kernel/src/psf2.c");
        nob_cc_output(&cmd, "kernel/kernel");
    }

    nob_cmd_append(&cmd, "-ffreestanding", "-Ilimine-protocol/include",
                   "-nostdlib", "-fPIE", "-fno-lto", "-fno-stack-protector",
                   "-fno-stack-check", "-ffunction-sections",
                   "-fdata-sections");

    switch (arch) {
    case x86_64:
        nob_cmd_append(&cmd, "-target", "x86_64-unknown-none-elf", "-m64",
                       "-march=x86-64", "-mabi=sysv", "-mno-80387", "-mno-mmx",
                       "-mno-sse", "-mno-sse2", "-mno-red-zone",
                       "-mcmodel=kernel");

        nob_cmd_append(&cmd, "-Tkernel/src/arch/x86_64/linker.ld");

        if (!only_ccls) {
            nob_cc_inputs(&cmd, "kernel/src/arch/x86_64/interrupts.c",
                          "kernel/src/arch/x86_64/gdt.c",
                          "kernel/src/arch/x86_64/idt.c",
                          "kernel/src/arch/x86_64/arch.c");
        }

        break;
    }

    if (only_ccls) {
        Nob_String_Builder sb = {0};

        for (size_t i = 0; i < cmd.count; i++) {
            nob_sb_append_cstr(&sb, cmd.items[i]);
            nob_sb_append_cstr(&sb, "\n");
        }

        nob_write_entire_file(".ccls", sb.items, sb.count * sizeof(char));

        return 0;
    }

    nob_cmd_run(&cmd);

    if (iso) {
        nob_mkdir_if_not_exists("iso_root");
        nob_mkdir_if_not_exists("iso_root/boot");
        nob_mkdir_if_not_exists("iso_root/boot/limine");
        nob_mkdir_if_not_exists("iso_root/EFI");
        nob_mkdir_if_not_exists("iso_root/EFI/BOOT");

        nob_copy_file("kernel/kernel", "iso_root/boot/kernel");
        nob_copy_file("limine.conf", "iso_root/boot/limine.conf");

        nob_copy_file("limine/limine-bios.sys",
                      "iso_root/boot/limine/limine-bios.sys");

        nob_copy_file("limine/limine-bios-cd.bin",
                      "iso_root/boot/limine/limine-bios-cd.bin");

        nob_copy_file("limine/limine-uefi-cd.bin",
                      "iso_root/boot/limine/limine-uefi-cd.bin");

        nob_copy_file("limine/BOOTX64.EFI", "iso_root/EFI/BOOT/BOOTX64.EFI");
        nob_copy_file("limine/BOOTIA32.EFI", "iso_root/EFI/BOOT/BOOTIA32.EFI");

        nob_cmd_append(&cmd, "xorriso", "-as", "mkisofs", "-b",
                       "boot/limine/limine-bios-cd.bin", "--no-emul-boot",
                       "-boot-load-size", "4", "-boot-info-table", "--efi-boot",
                       "boot/limine/limine-uefi-cd.bin", "--efi-boot-part",
                       "--efi-boot-image", "--protective-msdos-label",
                       "iso_root", "-o", image_name);

        nob_cmd_run(&cmd);
    } else {
        if (nob_file_exists(image_name)) {
            nob_delete_file(image_name);
        }

        nob_cmd_append(&cmd, "dd", "if=/dev/zero", "bs=1M", "count=0",
                       "seek=64", nob_temp_sprintf("of=%s", image_name));

        nob_cmd_run(&cmd);

        nob_cmd_append(&cmd, "sgdisk", image_name, "-n", "1:2048", "-t",
                       "1:ef00", "-m", "1");

        nob_cmd_run(&cmd);

        char *image_name_suffixed = nob_temp_sprintf("%s@@1M", image_name);

        nob_cmd_append(&cmd, "mformat", "-i", image_name_suffixed);

        nob_cmd_run(&cmd);

        nob_cmd_append(&cmd, "mmd", "-i", image_name_suffixed, "::/EFI",
                       "::/EFI/BOOT", "::/boot", "::/boot/limine");

        nob_cmd_run(&cmd);

        nob_cmd_append(&cmd, "mcopy", "-i", image_name_suffixed,
                       "kernel/kernel", "::/boot");

        nob_cmd_run(&cmd);

        nob_cmd_append(&cmd, "mcopy", "-i", image_name_suffixed, "limine.conf",
                       "::/boot/limine");

        nob_cmd_run(&cmd);

        nob_cmd_append(&cmd, "mcopy", "-i", image_name_suffixed,
                       "limine/limine-bios.sys", "::/boot/limine");

        nob_cmd_run(&cmd);

        nob_cmd_append(&cmd, "mcopy", "-i", image_name_suffixed,
                       "limine/BOOTX64.EFI", "::/EFI/BOOT");

        nob_cmd_run(&cmd);

        nob_cmd_append(&cmd, "mcopy", "-i", image_name_suffixed,
                       "limine/BOOTIA32.EFI", "::/EFI/BOOT");

        nob_cmd_run(&cmd);
    }

    nob_cmd_append(&cmd, "./limine/limine", "bios-install", image_name);

    nob_cmd_run(&cmd);

    if (!only_build) {
        const char *qemu_program =
            nob_temp_sprintf("qemu-system-%s", arch_to_string(arch));

        nob_cmd_append(&cmd, qemu_program, "-m", "4G", "-M", "q35");

        if (!bios) {
            nob_cmd_append(
                &cmd, "-drive",
                nob_temp_sprintf(
                    "if=pflash,unit=0,format=raw,file=ovmf/%s,readonly=on",
                    ovmf_code));

            nob_cmd_append(
                &cmd, "-drive",
                nob_temp_sprintf("if=pflash,unit=1,format=raw,file=ovmf/%s",
                                 ovmf_vars));
        } else if (iso) {
            nob_cmd_append(&cmd, "-boot", "d");
        }

        nob_cmd_append(&cmd, iso ? "-cdrom" : "-hda", image_name);

        // nob_cmd_append(&cmd, "-smp", "2");

        nob_cmd_run(&cmd);
    }
}
