use core::{
    alloc::{GlobalAlloc, Layout},
    cmp::Ordering,
    ops::Deref,
    ptr::NonNull,
};

use bit_field::BitField;
use spin::{lazy::Lazy, mutex::Mutex};

use crate::paging::MIN_PAGE_SIZE;

pub struct PageAllocator {
    heap_start: NonNull<u8>,
    page_count: usize,
}

impl PageAllocator {
    pub fn new(heap_start: NonNull<u8>, heap_len: usize) -> PageAllocator {
        // We intentionally use integer division to not overflow the heap
        let page_count = heap_len / MIN_PAGE_SIZE;

        let page_allocator = PageAllocator {
            heap_start,
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

    pub fn calculate_free_space(&self) -> usize {
        let mut amount = 0;

        for i in 0..self.page_count {
            if self.is_free(i) {
                amount += MIN_PAGE_SIZE;
            }
        }

        amount
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

    #[inline]
    fn get_page_index_of(&self, ptr: *mut u8) -> usize {
        (ptr.addr() - self.heap_start.addr().get()).div_ceil(MIN_PAGE_SIZE)
    }

    #[inline]
    pub fn contains(&self, address: usize) -> bool {
        unsafe {
            address > self.heap_start.addr().get()
                && address
                    < self
                        .heap_start
                        .byte_add(self.page_count * MIN_PAGE_SIZE)
                        .addr()
                        .get()
        }
    }

    pub fn alloc(&self, layout: Layout) -> *mut u8 {
        let needed_page_count = layout.size().div_ceil(MIN_PAGE_SIZE);

        for page_index in 0..self.page_count {
            if self.is_free(page_index) {
                let mut fits = true;

                for i in page_index..(page_index + needed_page_count) {
                    if !self.is_free(i) {
                        fits = false;

                        break;
                    }
                }

                if fits {
                    for i in page_index..(page_index + needed_page_count) {
                        self.set_free_bit(i, false);
                    }

                    return self.get_page(page_index).as_ptr();
                }
            }
        }

        core::ptr::null_mut()
    }

    pub fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let needed_page_count = layout.size().div_ceil(MIN_PAGE_SIZE);

        let page_index = self.get_page_index_of(ptr);

        for i in page_index..(page_index + needed_page_count) {
            self.set_free_bit(i, true);
        }

        println!("deallocated {needed_page_count} pages");
    }

    pub fn resize(&self, old_ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let old_needed_page_count = layout.size().div_ceil(MIN_PAGE_SIZE);
        let new_needed_page_count = new_size.div_ceil(MIN_PAGE_SIZE);

        match new_needed_page_count.cmp(&old_needed_page_count) {
            // We shouldn't reallocate with the same page count
            Ordering::Equal => old_ptr,

            // And if we are shrinking the page count, we should free the excess pages
            Ordering::Less => {
                let page_index = self.get_page_index_of(old_ptr);

                for i in (page_index + new_needed_page_count)..(page_index + old_needed_page_count)
                {
                    self.set_free_bit(i, true);
                }

                old_ptr
            }

            // Lastly, growing the page count requires us to check if there is some excess pages
            // which we can use, otherwise we have no choice but to reallocate
            Ordering::Greater => {
                let page_index = self.get_page_index_of(old_ptr);

                let mut fits = true;

                for i in (page_index + old_needed_page_count)..(page_index + new_needed_page_count)
                {
                    if !self.is_free(i) {
                        fits = false;

                        break;
                    }
                }

                if fits {
                    // We can expand our memory :)
                    for i in
                        (page_index + old_needed_page_count)..(page_index + new_needed_page_count)
                    {
                        self.set_free_bit(i, false);
                    }

                    old_ptr
                } else {
                    // We must reallocate :(
                    let new_layout =
                        unsafe { Layout::from_size_align_unchecked(new_size, layout.align()) };

                    let new_ptr = self.alloc(new_layout);

                    if !new_ptr.is_null() {
                        unsafe {
                            core::ptr::copy_nonoverlapping(old_ptr, new_ptr, new_size);
                        }

                        self.dealloc(old_ptr, layout);
                    }

                    new_ptr
                }
            }
        }
    }
}

#[repr(transparent)]
pub struct LockedPageAllocator(pub Lazy<Mutex<PageAllocator>>);

unsafe impl Send for LockedPageAllocator {}
unsafe impl Sync for LockedPageAllocator {}

unsafe impl GlobalAlloc for LockedPageAllocator {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        self.lock().alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        self.lock().dealloc(ptr, layout);
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        self.lock().resize(ptr, layout, new_size)
    }
}

impl Deref for LockedPageAllocator {
    type Target = Lazy<Mutex<PageAllocator>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
