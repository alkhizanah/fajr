#include <stdint.h>

typedef struct [[gnu::packed]] {
    uint16_t limit;
    uint64_t base;
} DescriptorTableRegister;
