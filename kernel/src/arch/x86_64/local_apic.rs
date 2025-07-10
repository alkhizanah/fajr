use spin::mutex::Mutex;

use crate::{memory, mp::MAX_CPU_COUNT, paging};

use super::{cpu::Cpu, msr::ModelSpecificRegister};

static LOCAL_APICS: Mutex<[LocalApic; MAX_CPU_COUNT]> =
    Mutex::new([const { LocalApic(0) }; MAX_CPU_COUNT]);

#[derive(Clone, Copy)]
pub struct LocalApic(usize);

#[repr(usize)]
pub enum LocalApicRegister {
    Id = 0x20,
    Version = 0x30,
    Eoi = 0xb0,
    TimerLvt = 0x320,
    TimerInit = 0x380,
    TimerDiv = 0x3e0,
}

impl LocalApic {
    pub fn get() -> LocalApic {
        LOCAL_APICS.lock()[Cpu::get().id as usize]
    }

    pub fn write(&self, register: LocalApicRegister, value: u32) {
        unsafe {
            ((self.0 + register as usize) as *mut u32).write_volatile(value);
        }
    }

    pub fn read(&self, register: LocalApicRegister) -> u32 {
        unsafe { ((self.0 + register as usize) as *mut u32).read_volatile() }
    }
}

pub fn init() {
    let cpu = Cpu::get();

    let mut local_apics = LOCAL_APICS.lock();

    let local_apic = &mut local_apics[cpu.id as usize];

    let apic_base_msr = ModelSpecificRegister::ApicBase.read();

    let local_apic_phys_addr = apic_base_msr as usize & 0xFFFFF000;
    let local_apic_virt_addr = paging::offset(local_apic_phys_addr);

    paging::get_active_table()
        .map(
            memory::align_down(local_apic_virt_addr, paging::MIN_PAGE_SIZE),
            memory::align_down(local_apic_phys_addr, paging::MIN_PAGE_SIZE),
        )
        .set_writable(true)
        .set_write_through(true)
        .set_cachability(false);

    local_apic.0 = local_apic_virt_addr;

    ModelSpecificRegister::ApicBase.write(apic_base_msr | (1 << 11));

    local_apic.write(LocalApicRegister::TimerInit, 0x19FBD0);
    local_apic.write(LocalApicRegister::TimerLvt, 32 | (1 << 17));
    local_apic.write(LocalApicRegister::TimerDiv, 16);
}

pub extern "x86-interrupt" fn handle_timer_tick() {
    LocalApic::get().write(LocalApicRegister::Eoi, 0);
}
