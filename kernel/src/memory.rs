use core::ptr::NonNull;

use lazy_static::lazy_static;
use limine::memory_map::EntryType as MemoryEntryType;
use spin::{lazy::Lazy, mutex::Mutex};

use crate::{
    allocators::buddy_allocator::{BuddyAllocator, LockedBuddyAllocator},
    paging::offset,
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

        &mut *core::ptr::slice_from_raw_parts_mut(
            offset(heap.base as usize) as *mut u8,
            heap.length as usize,
        )
    });
}

#[global_allocator]
pub static GLOBAL_BUDDY_ALLOCATOR: LockedBuddyAllocator = LockedBuddyAllocator(Lazy::new(|| {
    Mutex::new(unsafe {
        let mut heap = HEAP.lock();

        BuddyAllocator::new(
            NonNull::new(heap.as_mut_ptr()).unwrap_unchecked(),
            heap.len(),
        )
    })
}));
