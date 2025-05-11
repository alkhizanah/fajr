use core::arch::asm;

use bitflags::bitflags;
use lazy_static::lazy_static;

use super::DescriptorTableRegister;

#[derive(Debug, PartialEq)]
struct GlobalDescriptorTable<const MAX: usize = 8> {
    table: [u64; MAX],
    len: usize,
}

impl<const MAX: usize> GlobalDescriptorTable<MAX> {
    pub const fn empty() -> Self {
        Self {
            table: [0; MAX],
            len: 1,
        }
    }

    pub fn push(&mut self, entry: Entry) {
        debug_assert_ne!(self.len, MAX);
        self.table[self.len] = entry.0;
        self.len += 1;
    }

    pub fn register(&'static self) -> DescriptorTableRegister {
        DescriptorTableRegister {
            address: self.table.as_ptr() as u64,
            size: (self.len * size_of::<u64>() - 1) as u16,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Entry(u64);

bitflags! {
    /// Flags for a GDT descriptor. Not all flags are valid for all descriptor types.
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy)]
    pub struct EntryFlags: u64 {
        /// Set by the processor if this segment has been accessed. Only cleared by software.
        /// _Setting_ this bit in software prevents GDT writes on first use.
        const ACCESSED          = 1 << 40;
        /// For 32-bit data segments, sets the segment as writable. For 32-bit code segments,
        /// sets the segment as _readable_. In 64-bit mode, ignored for all segments.
        const WRITABLE          = 1 << 41;
        /// For code segments, sets the segment as “conforming”, influencing the
        /// privilege checks that occur on control transfers. For 32-bit data segments,
        /// sets the segment as "expand down". In 64-bit mode, ignored for data segments.
        const CONFORMING        = 1 << 42;
        /// This flag must be set for code segments and unset for data segments.
        const EXECUTABLE        = 1 << 43;
        /// This flag must be set for user segments (in contrast to system segments).
        const USER_SEGMENT      = 1 << 44;
        /// These two bits encode the Descriptor Privilege Level (DPL) for this descriptor.
        /// If both bits are set, the DPL is Ring 3, if both are unset, the DPL is Ring 0.
        const DPL_RING_3        = 3 << 45;
        /// Must be set for any segment, causes a segment not present exception if not set.
        const PRESENT           = 1 << 47;
        /// Available for use by the Operating System
        const AVAILABLE         = 1 << 52;
        /// Must be set for 64-bit code segments, unset otherwise.
        const LONG_MODE         = 1 << 53;
        /// Use 32-bit (as opposed to 16-bit) operands. If [`LONG_MODE`][Self::LONG_MODE] is set,
        /// this must be unset. In 64-bit mode, ignored for data segments.
        const DEFAULT_SIZE      = 1 << 54;
        /// Limit field is scaled by 4096 bytes. In 64-bit mode, ignored for all segments.
        const GRANULARITY       = 1 << 55;

        /// Bits `0..=15` of the limit field (ignored in 64-bit mode)
        const LIMIT_0_15        = 0xFFFF;
        /// Bits `16..=19` of the limit field (ignored in 64-bit mode)
        const LIMIT_16_19       = 0xF << 48;
        /// Bits `0..=23` of the base field (ignored in 64-bit mode, except for fs and gs)
        const BASE_0_23         = 0xFF_FFFF << 16;
        /// Bits `24..=31` of the base field (ignored in 64-bit mode, except for fs and gs)
        const BASE_24_31        = 0xFF << 56;
    }
}

impl EntryFlags {
    const COMMON: Self = Self::from_bits_truncate(
        Self::USER_SEGMENT.bits()
            | Self::PRESENT.bits()
            | Self::WRITABLE.bits()
            | Self::ACCESSED.bits()
            | Self::LIMIT_0_15.bits()
            | Self::LIMIT_16_19.bits()
            | Self::BASE_0_23.bits()
            | Self::BASE_24_31.bits()
            | Self::GRANULARITY.bits(),
    );

    const KERNEL_CODE: Self = Self::from_bits_truncate(
        Self::COMMON.bits() | Self::LONG_MODE.bits() | Self::EXECUTABLE.bits(),
    );

    const KERNEL_DATA: Self =
        Self::from_bits_truncate(Self::COMMON.bits() | EntryFlags::DEFAULT_SIZE.bits());

    const USER_CODE: Self =
        Self::from_bits_truncate(Self::KERNEL_CODE.bits() | EntryFlags::DPL_RING_3.bits());

    const USER_DATA: Self =
        Self::from_bits_truncate(Self::KERNEL_DATA.bits() | EntryFlags::DPL_RING_3.bits());
}

impl Entry {
    const KERNEL_CODE_SEGMENT: Entry = Entry(EntryFlags::KERNEL_CODE.bits());
    const KERNEL_DATA_SEGMENT: Entry = Entry(EntryFlags::KERNEL_DATA.bits());
    const USER_CODE_SEGMENT: Entry = Entry(EntryFlags::USER_CODE.bits());
    const USER_DATA_SEGMENT: Entry = Entry(EntryFlags::USER_DATA.bits());
}

lazy_static! {
    static ref GDT: GlobalDescriptorTable = {
        let mut gdt = GlobalDescriptorTable::empty();

        gdt.push(Entry::KERNEL_CODE_SEGMENT); // 0x08
        gdt.push(Entry::KERNEL_DATA_SEGMENT); // 0x10
        gdt.push(Entry::USER_CODE_SEGMENT); // 0x18
        gdt.push(Entry::USER_DATA_SEGMENT); // 0x20

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
    }
}
