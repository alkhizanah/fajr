#include "gdt.h"
#include "../../arch.h"

void arch_init_bsp(void) {
    gdt_init();
}
