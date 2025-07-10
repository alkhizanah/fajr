use spin::Mutex;

use crate::mp::MAX_CPU_COUNT;

use super::msr::ModelSpecificRegister;

static CPUS: Mutex<[Cpu; MAX_CPU_COUNT]> = Mutex::new([const { Cpu::new(0) }; MAX_CPU_COUNT]);

// Our own information about the CPU, not affiliated with limine's information
#[derive(Clone, Copy)]
#[repr(C)]
pub(super) struct Cpu {
    pub(super) id: u32,
}

impl Cpu {
    pub const fn new(id: u32) -> Cpu {
        Cpu { id }
    }

    pub fn get() -> &'static mut Cpu {
        unsafe { &mut *(ModelSpecificRegister::KernelGsBase.read() as *mut Cpu) }
    }

    pub fn set(cpu: Cpu) {
        let mut cpus = CPUS.lock();

        cpus[cpu.id as usize] = cpu;

        ModelSpecificRegister::KernelGsBase
            .write((&mut cpus[cpu.id as usize]) as *mut _ as usize as u64);
    }
}
