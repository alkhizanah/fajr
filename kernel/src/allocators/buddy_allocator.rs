use core::{
    alloc::{AllocError, Allocator, GlobalAlloc, Layout},
    num::NonZero,
    ops::Deref,
    ptr::NonNull,
};

use spin::{Lazy, Mutex};

pub struct BuddyAllocator {
    start: NonZero<usize>,
    end: NonZero<usize>,
}

#[derive(Clone, Copy, Debug)]
struct Header {
    free: bool,
    size: usize,
}

impl BuddyAllocator {
    pub fn new(heap_ptr: NonNull<u8>, heap_size: usize) -> Self {
        unsafe {
            heap_ptr.as_ptr().cast::<Header>().write(Header {
                free: true,
                size: 1 << heap_size.ilog2(),
            });

            Self {
                start: heap_ptr.addr(),
                end: heap_ptr.byte_add(heap_size).addr(),
            }
        }
    }

    pub fn calculate_free_bytes(&self) -> usize {
        let mut amount = 0;

        let mut current: NonNull<Header> = NonNull::dangling().with_addr(self.start);

        while current.addr() < self.end {
            unsafe {
                if current.read().free {
                    amount += current.read().size - size_of::<Header>();
                }

                current = current.byte_add(current.read().size);
            }
        }

        amount
    }
}

#[repr(transparent)]
pub struct LockedBuddyAllocator(pub Lazy<Mutex<BuddyAllocator>>);

impl Deref for LockedBuddyAllocator {
    type Target = Mutex<BuddyAllocator>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

unsafe impl GlobalAlloc for LockedBuddyAllocator {
    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let Ok(data) = self.allocate(layout) else {
            return core::ptr::null_mut();
        };

        data.as_ptr().cast()
    }

    #[inline]
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe {
            self.deallocate(ptr.as_mut().unwrap().into(), layout);
        }
    }
}

unsafe impl Allocator for LockedBuddyAllocator {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        let allocator = self.lock();

        let data_size = layout.pad_to_align().size();

        let allocation_size = size_of::<Header>() + data_size;

        let mut current: NonNull<Header> = NonNull::dangling().with_addr(allocator.start);

        while current.addr() < allocator.end {
            unsafe {
                if !current.read().free || current.read().size < allocation_size {
                    current = current.byte_add(current.read().size);

                    continue;
                }

                while (current.read().size >> 1) > allocation_size {
                    (*current.as_ptr()).size >>= 1;
                    let buddy = current.byte_add(current.read().size);
                    buddy.write(current.read());
                }

                (*current.as_ptr()).free = false;

                let data_ptr = current.byte_add(size_of::<Header>()).as_ptr().cast();

                let data = core::slice::from_raw_parts_mut(data_ptr, data_size);

                return Ok(data.into());
            }
        }

        Err(AllocError)
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, _layout: Layout) {
        let allocator = self.lock();

        unsafe {
            let mut allocation = ptr.byte_sub(size_of::<Header>()).cast::<Header>();

            (*allocation.as_ptr()).free = true;

            loop {
                let buddy = allocation
                    .map_addr(|addr| NonZero::new_unchecked(addr.get() ^ allocation.read().size));

                if buddy.addr() >= allocator.end {
                    break;
                }

                if !buddy.read().free {
                    break;
                }

                if buddy < allocation {
                    (*buddy.as_ptr()).size <<= 1;
                    allocation = buddy;
                } else if allocation < buddy {
                    (*allocation.as_ptr()).size <<= 1;
                }
            }
        }
    }
}
