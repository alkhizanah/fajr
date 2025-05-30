use core::{
    alloc::{GlobalAlloc, Layout},
    cmp::Ordering,
    ptr::NonNull,
};

use bit_field::BitField;
use spin::{lazy::Lazy, mutex::Mutex};

pub const MIN_PAGE_SIZE: usize = 4 * 1024;

pub struct PageAllocator {
    heap_start: NonNull<u8>,
    heap_end: NonNull<u8>,
    page_count: usize,
}

impl PageAllocator {
    pub fn new(heap_start: NonNull<u8>, heap_end: NonNull<u8>) -> PageAllocator {
        let heap_len = heap_end.addr().get() - heap_start.addr().get();

        let page_count = heap_len / MIN_PAGE_SIZE;

        let page_allocator = PageAllocator {
            heap_start,
            heap_end,
            page_count,
        };

        page_allocator.reserve_bitmap_pages();

        page_allocator
    }

    fn reserve_bitmap_pages(&self) {
        let needed_page_count = self.page_count.div_ceil(8).div_ceil(MIN_PAGE_SIZE);

        for i in 0..needed_page_count {
            self.set_free_bit(i, false);
        }

        for i in needed_page_count..self.page_count {
            self.set_free_bit(i, true);
        }
    }

    #[inline]
    fn is_free(&self, index: usize) -> bool {
        let byte_index = index / 8;
        let bit_index = index % 8;

        unsafe { (*self.heap_start.byte_add(byte_index).as_ptr()).get_bit(bit_index) }
    }

    #[inline]
    fn set_free_bit(&self, index: usize, value: bool) {
        let byte_index = index / 8;
        let bit_index = index % 8;

        unsafe {
            (*self.heap_start.byte_add(byte_index).as_ptr()).set_bit(bit_index, value);
        }
    }

    #[inline]
    fn get_page(&self, index: usize) -> NonNull<u8> {
        unsafe { self.heap_start.byte_add(index * MIN_PAGE_SIZE) }
    }

    fn get_free_pages(&self, needed_page_count: usize) -> Option<usize> {
        for i in 0..self.page_count {
            if self.is_free(i) {
                let mut fits = true;

                for j in i..(i + needed_page_count) {
                    if !self.is_free(j) {
                        fits = false;

                        break;
                    }
                }

                if fits {
                    return Some(i);
                }
            }
        }

        None
    }

    fn get_page_index_of(&self, address: usize) -> usize {
        (self.heap_end.addr().get() - address).div_ceil(MIN_PAGE_SIZE)
    }

    pub fn alloc(&self, layout: Layout) -> *mut u8 {
        let needed_page_count = layout.size().div_ceil(MIN_PAGE_SIZE);

        if let Some(first_page_index) = self.get_free_pages(needed_page_count) {
            for i in first_page_index..(first_page_index + needed_page_count) {
                self.set_free_bit(i, false);
            }

            return self.get_page(first_page_index).as_ptr();
        }

        core::ptr::null_mut()
    }

    pub fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let needed_page_count = layout.size().div_ceil(MIN_PAGE_SIZE);

        let first_page_index = self.get_page_index_of(ptr.addr());

        for i in first_page_index..(first_page_index + needed_page_count) {
            self.set_free_bit(i, true);
        }
    }

    fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let new_layout = unsafe { Layout::from_size_align_unchecked(new_size, layout.align()) };

        let new_ptr = self.alloc(new_layout);

        if !new_ptr.is_null() {
            unsafe {
                core::ptr::copy_nonoverlapping(ptr, new_ptr, layout.size().min(new_size));
            }

            self.dealloc(ptr, layout);
        }

        new_ptr
    }

    pub fn resize(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let previous_needed_page_count = layout.size().div_ceil(MIN_PAGE_SIZE);
        let new_needed_page_count = new_size.div_ceil(MIN_PAGE_SIZE);

        // Trying to not call realloc until it is really a must to do
        match new_needed_page_count.cmp(&previous_needed_page_count) {
            // That's simple, we shouldn't call realloc to allocate the same bytes
            Ordering::Equal => ptr,

            // And if we are shrinking the size, we should just make the excess pages be free
            Ordering::Less => {
                let first_page_index = self.get_page_index_of(ptr.addr());

                for i in (first_page_index + new_needed_page_count)
                    ..(first_page_index + previous_needed_page_count)
                {
                    self.set_free_bit(i, true);
                }

                ptr
            }

            // Now the fun part, growing the size requires us to check if there is some memory
            // after the end which we can use, if there isn't then we have no choice but
            // to reallocate
            Ordering::Greater => {
                let first_page_index = self.get_page_index_of(ptr.addr());

                let mut fits = true;

                for i in (first_page_index + previous_needed_page_count)
                    ..(first_page_index + new_needed_page_count)
                {
                    if !self.is_free(i) {
                        fits = false;

                        break;
                    }
                }

                if fits {
                    // Yay, we can just expand our memory!!
                    for i in (first_page_index + previous_needed_page_count)
                        ..(first_page_index + new_needed_page_count)
                    {
                        self.set_free_bit(i, false);
                    }

                    ptr
                } else {
                    // We couldn't escape from calling realloc :<
                    self.realloc(ptr, layout, new_size)
                }
            }
        }
    }
}

pub struct LockedPageAllocator(pub Lazy<Mutex<PageAllocator>>);

unsafe impl Send for LockedPageAllocator {}
unsafe impl Sync for LockedPageAllocator {}

unsafe impl GlobalAlloc for LockedPageAllocator {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        self.0.lock().alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        self.0.lock().dealloc(ptr, layout);
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        self.0.lock().resize(ptr, layout, new_size)
    }
}
