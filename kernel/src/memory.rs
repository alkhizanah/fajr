use core::ptr::NonNull;

use lazy_static::lazy_static;
use limine::memory_map::EntryType as MemoryEntryType;
use spin::{lazy::Lazy, mutex::Mutex};

use crate::{
    allocators::buddy_allocator::{BuddyAllocator, LockedBuddyAllocator},
    paging::virt_from_phys,
    requests::MEMORY_MAP_REQUEST,
};

lazy_static! {
    static ref HEAP: Mutex<&'static mut [u8]> = Mutex::new(unsafe {
        let heap = **MEMORY_MAP_REQUEST
            .get_response()
            .expect("could not ask limine to get the memory map")
            .entries()
            .iter()
            .filter(|a| a.entry_type == MemoryEntryType::USABLE)
            .max_by(|a, b| a.length.cmp(&b.length))
            .expect("could not find a usable memory entry");

        core::ptr::slice_from_raw_parts_mut(
            virt_from_phys(heap.base) as *mut u8,
            heap.length as usize,
        )
        .as_mut()
        .unwrap_unchecked()
    });
}

#[global_allocator]
pub static BUDDY_ALLOCATOR: LockedBuddyAllocator = LockedBuddyAllocator(Lazy::new(|| {
    Mutex::new(unsafe {
        let mut heap = HEAP.lock();

        BuddyAllocator::new(
            NonNull::new(heap.as_mut_ptr()).unwrap_unchecked(),
            heap.len(),
        )
    })
}));
