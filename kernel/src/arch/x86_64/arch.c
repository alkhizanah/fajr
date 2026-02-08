#include "../../arch.h"
#include "gdt.h"
#include "idt.h"

void arch_init_bsp(void) {
    gdt_init();
    idt_init();
}
