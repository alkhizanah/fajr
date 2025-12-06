#include "gdt.h"
#include "dtr.h"

static Gdt gdt;

#define GDT_ENTRY_COMMON                                                       \
    GDT_ENTRY_USER_SEGMENT | GDT_ENTRY_PRESENT | GDT_ENTRY_WRITABLE |          \
        GDT_ENTRY_ACCESSED | GDT_ENTRY_LIMIT_0_15 | GDT_ENTRY_LIMIT_16_19 |    \
        GDT_ENTRY_BASE_0_23 | GDT_ENTRY_BASE_24_31 | GDT_ENTRY_GRANULARITY

#define GDT_ENTRY_KERNEL_CODE                                                  \
    GDT_ENTRY_COMMON | GDT_ENTRY_LONG_MODE | GDT_ENTRY_EXECUTABLE

#define GDT_ENTRY_KERNEL_DATA GDT_ENTRY_COMMON | GDT_ENTRY_DEFAULT_SIZE

#define GDT_ENTRY_USER_CODE GDT_ENTRY_KERNEL_CODE | GDT_ENTRY_DPL_RING_3

#define GDT_ENTRY_USER_DATA GDT_ENTRY_KERNEL_DATA | GDT_ENTRY_DPL_RING_3

void gdt_init(void) {
    gdt[0] = 0;                     // 0x00
    gdt[1] = GDT_ENTRY_KERNEL_CODE; // 0x08
    gdt[2] = GDT_ENTRY_KERNEL_DATA; // 0x10
    gdt[3] = GDT_ENTRY_USER_CODE;   // 0x18
    gdt[4] = GDT_ENTRY_USER_DATA;   // 0x20

    DescriptorTableRegister gdtr = {
        .limit = sizeof(gdt) - 1,
        .base = (uint64_t)&gdt,
    };

    asm volatile("lgdt (%0)\n"
                 "pushq $0x08\n"
                 "leaq .reload_code_segment(%%rip), %%rax\n"
                 "pushq %%rax\n"
                 "lretq\n"
                 ".reload_code_segment:\n"
                 "mov $0x10, %%ax\n"
                 "mov  %%ax, %%ds\n"
                 "mov  %%ax, %%es\n"
                 "mov  %%ax, %%fs\n"
                 "mov  %%ax, %%gs\n"
                 "mov  %%ax, %%ss\n" ::"r"(&gdtr)
                 : "rax");
}
