use lazy_static::lazy_static;

use crate::requests::HHDM_REQUEST;

lazy_static! {
    static ref HHDM_OFFSET: u64 = HHDM_REQUEST
        .get_response()
        .expect("could not ask limine to get the higher half direct map offset")
        .offset();
}

pub fn virt_from_phys(phys: u64) -> u64 {
    phys + *HHDM_OFFSET
}
