#include "../arch.h"

void interrupts_disable(void) {
    asm("cli"); // Clear Interrupts Flag
}

void interrupts_enable(void) {
    asm("sti"); // Set Interrupts Flag
}

void wait_for_interrupts(void) {
    asm("hlt"); // Halt
}
