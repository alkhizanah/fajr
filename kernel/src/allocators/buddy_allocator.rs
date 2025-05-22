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

    fn split(&mut self, block: NonNull<Header>) -> NonNull<Header> {
        unsafe {
            let mut header = block.read();
            header.size /= 2;
            block.write(header);

            let buddy = block.byte_add(header.size);
            buddy.write(header);
            buddy
        }
    }

    fn merge(&mut self, result: &mut NonNull<Header>) {
        loop {
            unsafe {
                let result_header = result.read();

                let buddy =
                    result.map_addr(|addr| NonZero::new_unchecked(addr.get() ^ result_header.size));

                if buddy.addr() >= self.end {
                    break;
                }

                let buddy_header = buddy.read();

                if buddy_header.free {
                    if buddy < *result {
                        (*buddy.as_ptr()).size <<= 1;
                        *result = buddy;
                    } else if *result < buddy {
                        (*result.as_ptr()).size <<= 1;
                    }
                } else {
                    break;
                }
            }
        }
    }

    pub fn calculate_free_bytes(&self) -> usize {
        let mut amount = 0;

        let mut current: NonNull<Header> = NonNull::dangling().map_addr(|_| self.start);

        while current.addr() < self.end {
            unsafe {
                let header = current.read();

                if header.free {
                    amount += header.size - size_of::<Header>();
                }

                current = current.byte_add(header.size);
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
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let Ok(data) = self.allocate(layout) else {
            return core::ptr::null_mut();
        };

        data.as_ptr().cast()
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe {
            self.deallocate(ptr.as_mut().unwrap().into(), layout);
        }
    }
}

unsafe impl Allocator for LockedBuddyAllocator {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        let mut allocator = self.lock();

        let data_size = layout.pad_to_align().size();

        let allocation_size = size_of::<Header>() + data_size;

        let mut current: NonNull<Header> = NonNull::dangling().map_addr(|_| allocator.start);

        while current.addr() < allocator.end {
            unsafe {
                let mut header = current.read();

                if !header.free || header.size < allocation_size {
                    current = current.byte_add(header.size);

                    continue;
                }

                while header.size / 2 > allocation_size {
                    allocator.split(current);

                    header = current.read();
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
        let mut allocator = self.lock();

        unsafe {
            let mut allocation = ptr.byte_sub(size_of::<Header>()).cast::<Header>();

            (*allocation.as_ptr()).free = true;

            allocator.merge(&mut allocation);
        }
    }
}
