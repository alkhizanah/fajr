use core::ptr::NonNull;

use limine::memory_map::EntryType as MemoryEntryType;
use spin::{Lazy, Mutex};

use crate::{
    allocators::page_allocator::{LockedPageAllocator, PageAllocator},
    paging,
    requests::MEMORY_MAP_REQUEST,
};

#[global_allocator]
pub static PAGE_ALLOCATOR: LockedPageAllocator = LockedPageAllocator(Lazy::new(|| {
    Mutex::new(unsafe {
        let heap = MEMORY_MAP_REQUEST
            .get_response()
            .expect("could not ask limine to get the memory map")
            .entries()
            .iter()
            .filter(|a| a.entry_type == MemoryEntryType::USABLE)
            .map(|a| {
                &mut *core::ptr::slice_from_raw_parts_mut(
                    paging::offset(a.base as usize) as *mut u8,
                    a.length as usize,
                )
            })
            .max_by(|a, b| a.len().cmp(&b.len()))
            .expect("could not find a large usable memory region");

        let heap_len = heap.len();
        let heap_start = NonNull::from(heap).cast();

        PageAllocator::new(heap_start, heap_len)
    })
}));
