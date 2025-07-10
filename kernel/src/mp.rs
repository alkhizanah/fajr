use crate::requests::MP_REQUEST;

pub const MAX_CPU_COUNT: usize = 64;

extern "C" fn init_ap(limine_cpu: &limine::mp::Cpu) -> ! {
    crate::init_ap(limine_cpu.id);
}

/// Boot application processors by directing them into kernel code that they can execute
pub fn boot_ap() {
    let mut mp_request = MP_REQUEST.lock();

    let mp_respone = mp_request
        .get_response_mut()
        .expect("could ask limine for multiproccessing information");

    for limine_cpu in mp_respone.cpus_mut().iter_mut().take(MAX_CPU_COUNT) {
        limine_cpu.goto_address.write(init_ap);
    }
}
