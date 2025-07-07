use spin::mutex::Mutex;

use crate::{memory, mp::MAX_CPU_COUNT, paging};

use super::msr::ModelSpecificRegister;

static LOCAL_APICS: Mutex<[LocalApic; MAX_CPU_COUNT]> = Mutex::new([const { LocalApic(0) }; MAX_CPU_COUNT]);

#[derive(Clone, Copy)]
pub struct LocalApic(usize);

#[repr(usize)]
pub enum LocalApicRegister {
    Id = 0x20,
    Version = 0x30,
    Eoi = 0xb0,
    TimerLvt = 0x320,
    TimerInit = 0x380,
}

impl LocalApic {
    pub fn get(cpu_id: u32) -> LocalApic {
        LOCAL_APICS.lock()[cpu_id as usize]
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

pub fn init(cpu_id: u32) {
    let mut local_apics = LOCAL_APICS.lock();

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

    local_apics[cpu_id as usize].0 = local_apic_virt_addr;

    ModelSpecificRegister::ApicBase.write(apic_base_msr | (1 << 11));
}
