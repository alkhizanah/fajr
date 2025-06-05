use crate::requests::MP_REQUEST;

pub const MAX_CPU_COUNT: usize = 64;

pub fn boot_cpus() {
    let mut mp_request = MP_REQUEST.lock();

    let mp_respone = mp_request
        .get_response_mut()
        .expect("could ask limine for multiproccessing information");

    for cpu in mp_respone.cpus_mut().iter_mut().take(MAX_CPU_COUNT) {
        cpu.goto_address.write(crate::init_ap);
    }
}
