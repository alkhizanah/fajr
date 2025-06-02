use core::{
    alloc::{GlobalAlloc, Layout},
    cmp::Ordering,
    ptr::NonNull,
};

use limine::memory_map::EntryType as MemoryEntryType;
use spin::{Lazy, Mutex};

use crate::{allocators::page_allocator::PageAllocator, paging, requests::MEMORY_MAP_REQUEST};

const MAX_REGION_COUNT: usize = 128;

#[repr(transparent)]
pub struct ChainedPageAllocators(Lazy<Mutex<[Option<PageAllocator>; MAX_REGION_COUNT]>>);

impl ChainedPageAllocators {
    pub fn calculate_free_space(&self) -> usize {
        self.0
            .lock()
            .iter()
            .filter_map(|&a| a)
            .map(|a| a.calculate_free_space())
            .sum()
    }
}

unsafe impl Send for ChainedPageAllocators {}
unsafe impl Sync for ChainedPageAllocators {}

unsafe impl GlobalAlloc for ChainedPageAllocators {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        for page_allocator in self.0.lock().iter().filter_map(|&a| a) {
            let allocation = page_allocator.alloc(layout);

            if !allocation.is_null() {
                return allocation;
            }
        }

        core::ptr::null_mut()
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        for page_allocator in self.0.lock().iter().filter_map(|&a| a) {
            if page_allocator.contains(ptr.addr()) {
                page_allocator.dealloc(ptr, layout);

                break;
            }
        }
    }

    unsafe fn realloc(&self, old_ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        for page_allocator in self.0.lock().iter().filter_map(|&a| a) {
            if page_allocator.contains(old_ptr.addr()) {
                if !page_allocator.resize(old_ptr, layout, new_size) {
                    let new_layout =
                        unsafe { Layout::from_size_align_unchecked(new_size, layout.align()) };

                    let new_ptr = unsafe { self.alloc(new_layout) };

                    if !new_ptr.is_null() {
                        unsafe {
                            core::ptr::copy_nonoverlapping(old_ptr, new_ptr, new_size);

                            self.dealloc(old_ptr, layout);
                        }
                    }

                    return new_ptr;
                } else {
                    return old_ptr;
                }
            }
        }

        core::ptr::null_mut()
    }
}

#[global_allocator]
pub static PAGE_ALLOCATOR: ChainedPageAllocators = ChainedPageAllocators(Lazy::new(|| {
    Mutex::new(unsafe {
        let regions = MEMORY_MAP_REQUEST
            .get_response()
            .expect("could not ask limine to get the memory map")
            .entries()
            .iter()
            .filter_map(|a| {
                (a.entry_type == MemoryEntryType::USABLE).then_some(
                    &mut *core::ptr::slice_from_raw_parts_mut(
                        paging::offset(a.base as usize) as *mut u8,
                        a.length as usize,
                    ),
                )
            });

        let mut page_allocators = [const { None }; MAX_REGION_COUNT];

        let mut i = 0;

        for region in regions {
            let region_len = region.len();

            if !PageAllocator::can_be_used(region_len) {
                continue;
            }

            let region_start = NonNull::from(region).cast();

            page_allocators[i] = Some(PageAllocator::new(region_start, region_len));

            i += 1;

            if i >= page_allocators.len() {
                break;
            }
        }

        page_allocators.sort_by(|a, b| {
            if let Some(a) = a {
                if let Some(b) = b {
                    b.page_count.cmp(&a.page_count)
                } else {
                    Ordering::Less
                }
            } else {
                Ordering::Greater
            }
        });

        page_allocators
    })
}));
