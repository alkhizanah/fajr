use core::arch::asm;

use bit_field::BitField;
use lazy_static::lazy_static;

use super::{DescriptorTableRegister, tss::TaskStateSegment};

#[derive(Debug, PartialEq)]
struct GlobalDescriptorTable<const MAX: usize = 8> {
    table: [Entry; MAX],
    len: usize,
}

impl<const MAX: usize> GlobalDescriptorTable<MAX> {
    pub const fn empty() -> Self {
        Self {
            table: [Entry(0); MAX],
            len: 1,
        }
    }

    pub const fn push(&mut self, descriptor: Descriptor) {
        match descriptor {
            Descriptor::UserSegment(value) => {
                self.table[self.len] = Entry(value);
                self.len += 1;
            }

            Descriptor::SystemSegment(value_low, value_high) => {
                self.table[self.len] = Entry(value_low);
                self.len += 1;
                self.table[self.len] = Entry(value_high);
                self.len += 1;
            }
        }
    }

    pub fn register(&'static self) -> DescriptorTableRegister {
        DescriptorTableRegister {
            address: self.table.as_ptr() as u64,
            size: (self.len * size_of::<Entry>() - 1) as u16,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(transparent)]
struct Entry(u64);

#[derive(Debug, Clone, Copy, PartialEq)]
enum Descriptor {
    UserSegment(u64),
    SystemSegment(u64, u64),
}

#[allow(dead_code)]
mod descriptor_flags {
    /// Set by the processor if this segment has been accessed. Only cleared by software.
    /// _Setting_ this bit in software prevents GDT writes on first use.
    pub const ACCESSED: u64 = 1 << 40;
    /// For 32-bit data segments, sets the segment as writable. For 32-bit code segments,
    /// sets the segment as _readable_. In 64-bit mode, ignored for all segments.
    pub const WRITABLE: u64 = 1 << 41;
    /// For code segments, sets the segment as “conforming”, influencing the
    /// privilege checks that occur on control transfers. For 32-bit data segments,
    /// sets the segment as "expand down". In 64-bit mode, ignored for data segments.
    pub const CONFORMING: u64 = 1 << 42;
    /// This flag must be set for code segments and unset for data segments.
    pub const EXECUTABLE: u64 = 1 << 43;
    /// This flag must be set for user segments (in contrast to system segments).
    pub const USER_SEGMENT: u64 = 1 << 44;
    /// These two bits encode the Descriptor Privilege Level (DPL) for this descriptor.
    /// If both bits are set, the DPL is Ring 3, if both are unset, the DPL is Ring 0.
    pub const DPL_RING_3: u64 = 3 << 45;
    /// Must be set for any segment, causes a segment not present exception if not set.
    pub const PRESENT: u64 = 1 << 47;
    /// Available for use by the Operating System
    pub const AVAILABLE: u64 = 1 << 52;
    /// Must be set for 64-bit code segments, unset otherwise.
    pub const LONG_MODE: u64 = 1 << 53;
    /// Use 32-bit (as opposed to 16-bit) operands. If [`LONG_MODE`][LONG_MODE] is set,
    /// this must be unset. In 64-bit mode, ignored for data segments.
    pub const DEFAULT_SIZE: u64 = 1 << 54;
    /// Limit field is scaled by 4096 bytes. In 64-bit mode, ignored for all segments.
    pub const GRANULARITY: u64 = 1 << 55;

    /// Bits `0..=15` of the limit field (ignored in 64-bit mode)
    pub const LIMIT_0_15: u64 = 0xFFFF;
    /// Bits `16..=19` of the limit field (ignored in 64-bit mode)
    pub const LIMIT_16_19: u64 = 0xF << 48;
    /// Bits `0..=23` of the base field (ignored in 64-bit mode, except for fs and gs)
    pub const BASE_0_23: u64 = 0xFF_FFFF << 16;
    /// Bits `24..=31` of the base field (ignored in 64-bit mode, except for fs and gs)
    pub const BASE_24_31: u64 = 0xFF << 56;

    pub const COMMON: u64 = USER_SEGMENT
        | PRESENT
        | WRITABLE
        | ACCESSED
        | LIMIT_0_15
        | LIMIT_16_19
        | BASE_0_23
        | BASE_24_31
        | GRANULARITY;

    pub const KERNEL_CODE: u64 = COMMON | LONG_MODE | EXECUTABLE;

    pub const KERNEL_DATA: u64 = COMMON | DEFAULT_SIZE;

    pub const USER_CODE: u64 = KERNEL_CODE | DPL_RING_3;

    pub const USER_DATA: u64 = KERNEL_DATA | DPL_RING_3;
}

impl Descriptor {
    #[inline]
    pub const fn kernel_code_segment() -> Descriptor {
        Descriptor::UserSegment(descriptor_flags::KERNEL_CODE)
    }

    #[inline]
    pub const fn kernel_data_segment() -> Descriptor {
        Descriptor::UserSegment(descriptor_flags::KERNEL_DATA)
    }

    #[inline]
    pub const fn user_code_segment() -> Descriptor {
        Descriptor::UserSegment(descriptor_flags::USER_CODE)
    }

    #[inline]
    pub const fn user_data_segment() -> Descriptor {
        Descriptor::UserSegment(descriptor_flags::USER_DATA)
    }

    #[inline]
    pub fn task_state_segment(tss: &'static TaskStateSegment) -> Descriptor {
        let ptr = tss as *const _ as u64;

        let mut low = descriptor_flags::PRESENT;
        let mut high = 0;

        // address
        low.set_bits(16..40, ptr.get_bits(0..24));
        low.set_bits(56..64, ptr.get_bits(24..32));
        high.set_bits(0..32, ptr.get_bits(32..64));

        // size
        low.set_bits(0..16, (size_of::<TaskStateSegment>() - 1) as u64);

        // type (0b1001 means 64-bit available tss)
        low.set_bits(40..44, 0b1001);

        Descriptor::SystemSegment(low, high)
    }
}

lazy_static! {
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();

        tss.interrupt_stack_table[0] = {
            const IST_STACK_SIZE: usize = 20 * 1024;
            static mut IST_STACK: [u8; IST_STACK_SIZE] = [0; IST_STACK_SIZE];
            ((&raw const IST_STACK).addr() + IST_STACK_SIZE) as u64
        };

        tss
    };
}

lazy_static! {
    static ref GDT: GlobalDescriptorTable = {
        let mut gdt = GlobalDescriptorTable::empty();

        gdt.push(Descriptor::kernel_code_segment()); // 0x08
        gdt.push(Descriptor::kernel_data_segment()); // 0x10
        gdt.push(Descriptor::user_code_segment()); // 0x18
        gdt.push(Descriptor::user_data_segment()); // 0x20
        gdt.push(Descriptor::task_state_segment(&TSS)); // 0x28

        gdt
    };
}

pub fn init() {
    unsafe {
        asm!("lgdt [{}]", in(reg) &GDT.register(), options(readonly, nostack, preserves_flags));

        asm!(
            "push 0x08",
            "lea rax, [{}]",
            "push rax",
            "retfq",
            label {
                unsafe {
                    asm!(
                        "   mov ax, 0x10",
                        "   mov es, ax",
                        "   mov ss, ax",
                        "   mov ds, ax",
                        "   mov fs, ax",
                        "   mov gs, ax",
                    )
                }
            },
            options(preserves_flags)
        );

        asm!("ltr {0:x}", in(reg) 0x28, options(readonly, nostack, preserves_flags));
    }
}
