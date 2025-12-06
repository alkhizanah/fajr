#include "../../arch.h"
#include "gdt.h"

void arch_init_bsp(void) { gdt_init(); }
