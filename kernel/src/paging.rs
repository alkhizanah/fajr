use lazy_static::lazy_static;

use crate::requests::HHDM_REQUEST;

lazy_static! {
    static ref HHDM_OFFSET: usize = HHDM_REQUEST
        .get_response()
        .expect("could not ask limine to get the higher half direct map offset")
        .offset() as usize;
}

pub fn offset(phys: usize) -> usize {
    phys + *HHDM_OFFSET
}
