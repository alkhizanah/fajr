#include <stddef.h>
#include <stdint.h>

#include "../../terminal.h"
#include "dtr.h"
#include "idt.h"

typedef struct {
    uint16_t address_low;
    uint16_t code_segment;
    uint16_t options;
    uint16_t address_middle;
    uint32_t address_high;
    uint32_t reserved;
} IdtEntry;

typedef IdtEntry Idt[256];

static Idt idt;

static void idt_set_handler(size_t index, uint64_t address) {
    IdtEntry *entry = idt + index;

    entry->address_low = address;
    entry->address_middle = address >> 16;
    entry->address_high = address >> 32;

    entry->code_segment = 0x08;

    entry->options = 0x8E00;
}

typedef struct {
    uint64_t rip;
    uint64_t cs;
    uint64_t rflags;
    uint64_t rsp;
    uint64_t ss;
} InterruptStackFrame;

__attribute__((interrupt)) static void idt_breakpoint(InterruptStackFrame *frame) {
    (void)frame;
    kprintf("breakpoint\n");
}

void idt_init(void) {
    idt_set_handler(3, (uint64_t)&idt_breakpoint);

    DescriptorTableRegister idtr = {
        .limit = sizeof(idt) - 1,
        .base = (uint64_t)&idt,
    };

    asm volatile("lidt (%0)" ::"r"(&idtr));
}
