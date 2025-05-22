use core::{
    alloc::{AllocError, Allocator, GlobalAlloc, Layout},
    ops::Deref,
    ptr::NonNull,
};

use spin::{Lazy, Mutex};

pub struct FirstFitAllocator {
    head: Option<NonNull<Header>>,
}

#[derive(Clone, Copy, Debug)]
struct Header {
    size: usize,
    next: Option<NonNull<Header>>,
}

impl FirstFitAllocator {
    pub fn new(heap_ptr: NonNull<u8>, heap_size: usize) -> Self {
        unsafe {
            let head_ptr = heap_ptr.as_ptr() as *mut Header;

            head_ptr.write(Header {
                size: heap_size,
                next: None,
            });

            let head = NonNull::new(head_ptr);

            Self { head }
        }
    }

    fn merge(&mut self, result: &mut NonNull<Header>) {
        unsafe {
            // Repeat until there isn't any block to merged
            loop {
                let mut previous: Option<NonNull<Header>> = None;
                let mut current = self.head;

                let mut merged_at_all = false;

                while let Some(block) = current {
                    let result_ptr = result.as_ptr();
                    let result_size = (*result_ptr).size;
                    let block_ptr = block.as_ptr();
                    let block_size = (*block_ptr).size;

                    let mut merged_now = false;

                    if block_ptr.byte_add(block_size) == result_ptr {
                        // If this block is before the result block, then it should be the one whom the
                        // result block will be merged into
                        (*block_ptr).size += result_size;
                        *result = block;

                        merged_at_all = true;
                        merged_now = true;
                    } else if result_ptr.byte_add(result_size) == block_ptr {
                        // However, if this block is after the result block, then the result block
                        // should be the one whom the block will be merged into
                        (*result_ptr).size += block_size;

                        merged_at_all = true;
                        merged_now = true;
                    }

                    if merged_now {
                        if let Some(previous) = previous {
                            (*previous.as_ptr()).next = (*block_ptr).next;
                        } else {
                            self.head = (*block_ptr).next;
                        }
                    }

                    previous = current;
                    current = (*block_ptr).next;
                }

                if !merged_at_all {
                    break;
                }
            }
        }
    }

    pub fn calculate_free_bytes(&self) -> usize {
        let mut amount = 0;

        unsafe {
            let mut current = self.head;

            while let Some(block) = current {
                let block_ptr = block.as_ptr();
                amount += (*block_ptr).size - size_of::<Header>();
                current = (*block_ptr).next;
            }
        }

        amount
    }
}

#[repr(transparent)]
pub struct LockedFirstFitAllocator(pub Lazy<Mutex<FirstFitAllocator>>);

unsafe impl Send for LockedFirstFitAllocator {}
unsafe impl Sync for LockedFirstFitAllocator {}

impl Deref for LockedFirstFitAllocator {
    type Target = Mutex<FirstFitAllocator>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

unsafe impl GlobalAlloc for LockedFirstFitAllocator {
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

unsafe impl Allocator for LockedFirstFitAllocator {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        let mut allocator = self.lock();

        let data_size = layout.pad_to_align().size();

        let allocation_size = size_of::<Header>() + data_size;

        let mut previous = None;
        let mut current = allocator.head;

        unsafe {
            while let Some(block) = current {
                let block_ptr = block.as_ptr();

                if (*block_ptr).size < allocation_size {
                    previous = current;
                    current = (*block_ptr).next;

                    continue;
                }

                if (*block_ptr).size > allocation_size {
                    let diff = (*block_ptr).size - allocation_size;

                    if diff > size_of::<Header>() {
                        (*block_ptr).size = allocation_size;

                        let new_block = block.byte_add((*block_ptr).size);

                        new_block.write(Header {
                            size: diff,
                            next: (*block_ptr).next,
                        });

                        (*block_ptr).next = Some(new_block);
                    }
                }

                if let Some(previous) = previous {
                    (*previous.as_ptr()).next = (*block.as_ptr()).next;
                } else {
                    allocator.head = (*block.as_ptr()).next;
                }

                let data = core::slice::from_raw_parts_mut(
                    block.byte_add(size_of::<Header>()).as_ptr().cast(),
                    data_size,
                );

                return Ok(data.into());
            }
        }

        Err(AllocError)
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, _layout: Layout) {
        let mut allocator = self.lock();

        unsafe {
            let mut header = ptr.byte_sub(size_of::<Header>()).cast::<Header>();

            allocator.merge(&mut header);

            if let Some(head) = allocator.head {
                (*header.as_ptr()).next = (*head.as_ptr()).next;
                (*head.as_ptr()).next = Some(header);
            } else {
                (*header.as_ptr()).next = None;
                allocator.head = Some(header);
            }
        }
    }
}
