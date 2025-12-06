#include <stdint.h>

#include "dtr.h"
#include "gdt.h"

typedef uint64_t GdtEntry;

/// Set by the processor if this segment has been accessed. Only cleared by
/// software. _Setting_ this bit in software prevents GDT writes on first use.
#define GDT_ENTRY_ACCESSED (1ll << 40)

/// For 32-bit data segments, sets the segment as writable. For 32-bit code
/// segments, sets the segment as _readable_. In 64-bit mode, ignored for all
/// segments.
#define GDT_ENTRY_WRITABLE (1ll << 41ll)

/// For code segments, sets the segment as “conforming”, influencing the
/// privilege checks that occur on control transfers. For 32-bit data segments,
/// sets the segment as "expand down". In 64-bit mode, ignored for data
/// segments.
#define GDT_ENTRY_CONFORMING (1ll << 42)

/// This flag must be set for code segments and unset for data segments.
#define GDT_ENTRY_EXECUTABLE (1ll << 43)

/// This flag must be set for user segments (in contrast to system segments).
#define GDT_ENTRY_USER_SEGMENT (1ll << 44)

/// These two bits encode the Descriptor Privilege Level (DPL) for this
/// descriptor. If both bits are set, the DPL is Ring 3, if both are unset, the
/// DPL is Ring 0.
#define GDT_ENTRY_DPL_RING_3 (3ll << 45)

/// Must be set for any segment, causes a segment not present exception if not
/// set.
#define GDT_ENTRY_PRESENT (1ll << 47)

/// Available for use by the Operating System
#define GDT_ENTRY_AVAILABLE (1ll << 52)

/// Must be set for 64-bit code segments, unset otherwise.
#define GDT_ENTRY_LONG_MODE (1ll << 53)

/// Use 32-bit (as opposed to 16-bit) operands. If [`LONG_MODE`][LONG_MODE] is
/// set, this must be unset. In 64-bit mode, ignored for data segments.
#define GDT_ENTRY_DEFAULT_SIZE (1ll << 54)

/// Limit field is scaled by 4096 bytes. In 64-bit mode, ignored for all
/// segments.
#define GDT_ENTRY_GRANULARITY (1ll << 55)

/// Bits `0..=15` of the limit field (ignored in 64-bit mode)
#define GDT_ENTRY_LIMIT_0_15 0xFFFFll

/// Bits `16..=19` of the limit field (ignored in 64-bit mode)
#define GDT_ENTRY_LIMIT_16_19 (0xFll << 48)

/// Bits `0..=23` of the base field (ignored in 64-bit mode, except for fs and
/// gs)
#define GDT_ENTRY_BASE_0_23 (0xFFFFFFll << 16)

/// Bits `24..=31` of the base field (ignored in 64-bit mode, except for fs and
/// gs)
#define GDT_ENTRY_BASE_24_31 (0xFFll << 56)

typedef GdtEntry Gdt[5];

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
