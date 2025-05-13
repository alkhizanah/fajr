use limine::memory_map::EntryType as MemoryEntryType;
use linked_list_allocator::LockedHeap;

use crate::{paging::virt_from_phys, requests::MEMORY_MAP_REQUEST};

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

pub fn init() {
    let heap = **MEMORY_MAP_REQUEST
        .get_response()
        .expect("could not ask limine to get the memory map")
        .entries()
        .iter()
        .filter(|a| a.entry_type == MemoryEntryType::USABLE)
        .max_by(|a, b| a.length.cmp(&b.length))
        .expect("could not find a usable memory entry");

    unsafe {
        ALLOCATOR
            .lock()
            .init(virt_from_phys(heap.base) as *mut u8, heap.length as usize);
    }
}
