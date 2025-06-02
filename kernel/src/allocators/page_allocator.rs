use core::{alloc::Layout, cmp::Ordering, ptr::NonNull};

use bit_field::BitField;

use crate::paging::MIN_PAGE_SIZE;

#[derive(Clone, Copy)]
pub struct PageAllocator {
    region_start: NonNull<u8>,
    pub page_count: usize,
}

impl PageAllocator {
    pub fn new(region_start: NonNull<u8>, region_len: usize) -> PageAllocator {
        // We intentionally use integer division to not overflow the region
        let page_count = region_len / MIN_PAGE_SIZE;

        let page_allocator = PageAllocator {
            region_start,
            page_count,
        };

        page_allocator.reserve_bitmap_pages();

        page_allocator
    }

    /// Checks whether a region can be used for page allocation
    #[inline(always)]
    pub fn can_be_used(region_len: usize) -> bool {
        region_len
            > (region_len / MIN_PAGE_SIZE)
                .div_ceil(8)
                .div_ceil(MIN_PAGE_SIZE)
    }

    fn reserve_bitmap_pages(&self) {
        let needed_page_count = self.page_count.div_ceil(8).div_ceil(MIN_PAGE_SIZE);
        (0..needed_page_count).for_each(|i| self.set_free_bit(i, false));
        (needed_page_count..self.page_count).for_each(|i| self.set_free_bit(i, true));
    }

    pub fn calculate_free_space(&self) -> usize {
        (0..self.page_count)
            .filter_map(|i| self.is_free(i).then_some(MIN_PAGE_SIZE))
            .sum()
    }

    #[inline(always)]
    fn is_free(&self, index: usize) -> bool {
        unsafe { (*self.region_start.byte_add(index / 8).as_ptr()).get_bit(index % 8) }
    }

    #[inline(always)]
    fn set_free_bit(&self, index: usize, value: bool) {
        unsafe {
            (*self.region_start.byte_add(index / 8).as_ptr()).set_bit(index % 8, value);
        }
    }

    #[inline(always)]
    fn get_page(&self, index: usize) -> NonNull<u8> {
        unsafe { self.region_start.byte_add(index * MIN_PAGE_SIZE) }
    }

    #[inline(always)]
    fn get_page_index_of(&self, ptr: *mut u8) -> usize {
        (ptr.addr() - self.region_start.addr().get()).div_ceil(MIN_PAGE_SIZE)
    }

    #[inline(always)]
    pub fn contains(&self, address: usize) -> bool {
        unsafe {
            address > self.region_start.addr().get()
                && address
                    < self
                        .region_start
                        .byte_add(self.page_count * MIN_PAGE_SIZE)
                        .addr()
                        .get()
        }
    }

    pub fn alloc(&self, layout: Layout) -> *mut u8 {
        let needed_page_count = layout.size().div_ceil(MIN_PAGE_SIZE);

        for page_index in 0..self.page_count {
            let page_indices = page_index..(page_index + needed_page_count);

            if page_indices.clone().all(|i| self.is_free(i)) {
                page_indices.for_each(|i| self.set_free_bit(i, false));

                return self.get_page(page_index).as_ptr();
            }
        }

        core::ptr::null_mut()
    }

    #[inline]
    pub fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let needed_page_count = layout.size().div_ceil(MIN_PAGE_SIZE);
        let page_index = self.get_page_index_of(ptr);
        let page_indices = page_index..(page_index + needed_page_count);
        page_indices.for_each(|i| self.set_free_bit(i, true));
    }

    /// Tries to resize the allocation without reallocating, returns whether the resize is
    /// successful, otherwise an allocation must be done
    pub fn resize(&self, old_ptr: *mut u8, layout: Layout, new_size: usize) -> bool {
        let old_needed_page_count = layout.size().div_ceil(MIN_PAGE_SIZE);
        let new_needed_page_count = new_size.div_ceil(MIN_PAGE_SIZE);

        match new_needed_page_count.cmp(&old_needed_page_count) {
            // We shouldn't reallocate with the same page count
            Ordering::Equal => true,

            // And if we are shrinking the page count, we should free the excess pages
            Ordering::Less => {
                let page_index = self.get_page_index_of(old_ptr);

                let page_indices =
                    (page_index + old_needed_page_count)..(page_index + new_needed_page_count);

                page_indices.for_each(|i| self.set_free_bit(i, true));

                true
            }

            // Lastly, growing the page count requires us to check if there is some excess pages
            // which we can use, otherwise we have no choice but to reallocate
            Ordering::Greater => {
                let page_index = self.get_page_index_of(old_ptr);

                let page_indices =
                    (page_index + old_needed_page_count)..(page_index + new_needed_page_count);

                if page_indices.clone().all(|i| self.is_free(i)) {
                    // We can expand our memory :)
                    page_indices.for_each(|i| self.set_free_bit(i, false));

                    true
                } else {
                    // We must reallocate :(
                    false
                }
            }
        }
    }
}
