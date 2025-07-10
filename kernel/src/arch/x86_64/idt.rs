use core::arch::asm;

use bit_field::BitField;
use lazy_static::lazy_static;

use crate::arch::local_apic;

use super::DescriptorTableRegister;

#[derive(Debug, PartialEq)]
#[repr(C, align(16))]
struct InterruptDescriptorTable {
    table: [Entry; 256],
}

impl InterruptDescriptorTable {
    pub const fn empty() -> Self {
        Self {
            table: [Entry::empty(); 256],
        }
    }

    pub fn register(&'static self) -> DescriptorTableRegister {
        DescriptorTableRegister {
            address: self as *const Self as u64,
            size: (size_of::<Self>() - 1) as u16,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
#[repr(C)]
struct Entry {
    address_low: u16,
    options: EntryOptions,
    address_middle: u16,
    address_high: u32,
    reserved: u32,
}

#[derive(Debug, PartialEq, Clone, Copy)]
#[repr(C)]
struct EntryOptions {
    cs: u16,
    bits: u16,
}

impl Entry {
    pub const fn empty() -> Self {
        Self {
            address_low: 0,
            address_middle: 0,
            address_high: 0,
            reserved: 0,
            options: EntryOptions::minimal(),
        }
    }

    pub fn set_handler_address(&mut self, address: u64) -> &mut EntryOptions {
        self.address_low = address as u16;
        self.address_middle = (address >> 16) as u16;
        self.address_high = (address >> 32) as u32;

        self.options.set_code_selector(0x08).set_present(true)
    }
}

impl EntryOptions {
    pub const fn minimal() -> Self {
        Self {
            cs: 0,
            bits: 0b1110_0000_0000,
        }
    }

    pub fn set_present(&mut self, value: bool) -> &mut Self {
        self.bits.set_bit(15, value);
        self
    }

    pub const fn set_code_selector(&mut self, cs: u16) -> &mut Self {
        self.cs = cs;
        self
    }

    pub fn set_stack_index(&mut self, index: u16) -> &mut Self {
        self.bits.set_bits(0..3, index + 1);
        self
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
#[repr(C)]
struct InterruptStackFrame {
    rip: u64,
    cs: u64,
    rflags: u64,
    rsp: u64,
    ss: u64,
}

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::empty();

        idt.table[0].set_handler_address(handle_division_error as usize as u64);
        idt.table[1].set_handler_address(handle_debug as usize as u64);
        idt.table[3].set_handler_address(handle_breakpoint as usize as u64);
        idt.table[4].set_handler_address(handle_overflow as usize as u64);
        idt.table[5].set_handler_address(handle_bound_range_exceeded as usize as u64);
        idt.table[6].set_handler_address(handle_invalid_opcode as usize as u64);
        idt.table[7].set_handler_address(handle_device_not_available as usize as u64);

        idt.table[8]
            .set_handler_address(handle_double_fault as usize as u64)
            .set_stack_index(0);

        idt.table[10].set_handler_address(handle_segmentation_fault as usize as u64);
        idt.table[11].set_handler_address(handle_segmentation_fault as usize as u64);
        idt.table[12].set_handler_address(handle_segmentation_fault as usize as u64);
        idt.table[13].set_handler_address(handle_general_protection_fault as usize as u64);
        idt.table[14].set_handler_address(handle_page_fault as usize as u64);
        idt.table[16].set_handler_address(handle_x87_floating_point_exception as usize as u64);
        idt.table[17].set_handler_address(handle_alignment_check as usize as u64);
        idt.table[18].set_handler_address(handle_machine_check as usize as u64);
        idt.table[19].set_handler_address(handle_simd_floating_point_exception as usize as u64);
        idt.table[20].set_handler_address(handle_virtualization_exception as usize as u64);
        idt.table[21].set_handler_address(handle_control_protection_exception as usize as u64);
        idt.table[28].set_handler_address(handle_hypervisor_injection_exception as usize as u64);
        idt.table[29].set_handler_address(handle_vmm_communication_exception as usize as u64);
        idt.table[30].set_handler_address(handle_security_exception as usize as u64);
        idt.table[32].set_handler_address(local_apic::handle_timer_tick as usize as u64);

        idt
    };
}

pub fn load() {
    unsafe {
        asm!("lidt [{}]", in(reg) &IDT.register(), options(readonly, nostack, preserves_flags));
    }
}

extern "x86-interrupt" fn handle_division_error(_: InterruptStackFrame) {
    panic!("division error");
}

extern "x86-interrupt" fn handle_debug(_: InterruptStackFrame) {
    println!("debug");
}

extern "x86-interrupt" fn handle_breakpoint(_: InterruptStackFrame) {
    println!("breakpoint");
}

extern "x86-interrupt" fn handle_overflow(_: InterruptStackFrame) {
    println!("overflow");
}

extern "x86-interrupt" fn handle_bound_range_exceeded(_: InterruptStackFrame) {
    panic!("bound range exceeded");
}

extern "x86-interrupt" fn handle_invalid_opcode(_: InterruptStackFrame) {
    panic!("invalid opcode");
}

extern "x86-interrupt" fn handle_device_not_available(_: InterruptStackFrame) {
    panic!("device not available");
}

extern "x86-interrupt" fn handle_double_fault(_: InterruptStackFrame, code: u64) {
    panic!("double fault: {}", code);
}

extern "x86-interrupt" fn handle_segmentation_fault(_: InterruptStackFrame, code: u64) {
    panic!("segmentation fault: {}", code);
}

extern "x86-interrupt" fn handle_general_protection_fault(_: InterruptStackFrame, code: u64) {
    panic!("general protection fault: {}", code);
}

extern "x86-interrupt" fn handle_page_fault(_: InterruptStackFrame, code: u64) {
    panic!("page fault: {}", code);
}

extern "x86-interrupt" fn handle_x87_floating_point_exception(_: InterruptStackFrame) {
    panic!("x87 floating point exception");
}

extern "x86-interrupt" fn handle_alignment_check(_: InterruptStackFrame, code: u64) {
    panic!("alignment check: {}", code);
}

extern "x86-interrupt" fn handle_machine_check(_: InterruptStackFrame) {
    panic!("machine check");
}

extern "x86-interrupt" fn handle_simd_floating_point_exception(_: InterruptStackFrame) {
    panic!("simd floating point exception");
}

extern "x86-interrupt" fn handle_virtualization_exception(_: InterruptStackFrame) {
    panic!("virtualization exception");
}

extern "x86-interrupt" fn handle_control_protection_exception(_: InterruptStackFrame, code: u64) {
    panic!("control protection exception: {}", code);
}

extern "x86-interrupt" fn handle_hypervisor_injection_exception(_: InterruptStackFrame) {
    panic!("hypervisor injection exception");
}

extern "x86-interrupt" fn handle_vmm_communication_exception(_: InterruptStackFrame, code: u64) {
    panic!("vmm communication exception: {}", code);
}

extern "x86-interrupt" fn handle_security_exception(_: InterruptStackFrame, code: u64) {
    panic!("security exception: {}", code);
}
