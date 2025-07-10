use core::arch::asm;

use lazy_static::lazy_static;

use crate::mp::MAX_CPU_COUNT;

use super::Cpu;

#[derive(Debug, Clone, Copy)]
#[repr(C, packed(4))]
pub struct TaskStateSegment {
    reserved_1: u32,
    pub privilege_stack_table: [u64; 3],
    reserved_2: u64,
    pub interrupt_stack_table: [u64; 7],
    reserved_3: u64,
    reserved_4: u16,
    pub iomap_base: u16,
}

impl TaskStateSegment {
    pub const fn new() -> Self {
        Self {
            privilege_stack_table: [0; 3],
            interrupt_stack_table: [0; 7],
            iomap_base: size_of::<TaskStateSegment>() as u16,
            reserved_1: 0,
            reserved_2: 0,
            reserved_3: 0,
            reserved_4: 0,
        }
    }
}

impl Default for TaskStateSegment {
    fn default() -> Self {
        Self::new()
    }
}

lazy_static! {
    pub static ref TSS: [TaskStateSegment; MAX_CPU_COUNT] = {
        let mut tss = [const { TaskStateSegment::new() }; MAX_CPU_COUNT];

        for tss in tss.iter_mut() {
            tss.interrupt_stack_table[0] = {
                const IST_STACK_SIZE: usize = 20 * 1024;
                static mut IST_STACK: [u8; IST_STACK_SIZE] = [0; IST_STACK_SIZE];
                ((&raw const IST_STACK).addr() + IST_STACK_SIZE) as u64
            };
        }

        tss
    };
}

pub fn load() {
    let cpu = Cpu::get();

    unsafe {
        asm!("ltr {0:x}", in(reg) (0x28 + (cpu.id * 16)), options(readonly, nostack, preserves_flags));
    }
}
