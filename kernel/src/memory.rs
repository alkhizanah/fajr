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

    pub fn contains(&self, address: usize) -> bool {
        self.0
            .lock()
            .iter()
            .filter_map(|&a| a)
            .any(|a| a.contains(address))
    }
}

unsafe impl Send for ChainedPageAllocators {}
unsafe impl Sync for ChainedPageAllocators {}

unsafe impl GlobalAlloc for ChainedPageAllocators {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        unsafe {
            self.0
                .lock()
                .iter()
                .find_map(|&a| a?.alloc(layout).as_mut())
                .map_or(core::ptr::null_mut(), |a| a as *mut _)
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let allocator = unsafe {
            self.0
                .lock()
                .iter()
                .filter_map(|&a| a)
                .find(|a| a.contains(ptr.addr()))
                .unwrap_unchecked()
        };

        allocator.dealloc(ptr, layout);
    }

    unsafe fn realloc(&self, old_ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let allocator = unsafe {
            self.0
                .lock()
                .iter()
                .filter_map(|&a| a)
                .find(|a| a.contains(old_ptr.addr()))
                .unwrap_unchecked()
        };

        if allocator.resize(old_ptr, layout, new_size) {
            return old_ptr;
        }

        let new_layout = unsafe { Layout::from_size_align_unchecked(new_size, layout.align()) };

        let new_ptr = unsafe { self.alloc(new_layout) };

        if !new_ptr.is_null() {
            unsafe {
                core::ptr::copy_nonoverlapping(old_ptr, new_ptr, new_size);

                self.dealloc(old_ptr, layout);
            }
        }

        new_ptr
    }
}

#[global_allocator]
pub static PAGE_ALLOCATOR: ChainedPageAllocators = ChainedPageAllocators(Lazy::new(|| {
    Mutex::new({
        let mut regions = MEMORY_MAP_REQUEST
            .get_response()
            .expect("could not ask limine to get the memory map")
            .entries()
            .iter()
            .filter_map(|a| {
                (a.entry_type == MemoryEntryType::USABLE
                    && PageAllocator::can_be_used(a.length as usize))
                .then_some(unsafe {
                    &mut *core::ptr::slice_from_raw_parts_mut(
                        paging::offset(a.base as usize) as *mut u8,
                        a.length as usize,
                    )
                })
            });

        let mut page_allocators = [None; MAX_REGION_COUNT];

        page_allocators.fill_with(|| {
            let region = regions.next()?;
            let region_len = region.len();
            let region_start = NonNull::from(region).cast();

            Some(PageAllocator::new(region_start, region_len))
        });

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
