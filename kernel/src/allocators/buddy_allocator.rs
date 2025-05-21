use core::{
    alloc::{AllocError, Allocator, GlobalAlloc, Layout},
    ops::Deref,
    ptr::NonNull,
};

use spin::{Lazy, Mutex};

pub struct BuddyAllocator {
    head: Option<NonNull<MemoryBlock>>,
}

#[derive(Clone, Copy, Debug)]
struct MemoryBlock {
    /// Size of the memory block including this header
    size: usize,
    next: Option<NonNull<MemoryBlock>>,
}

impl BuddyAllocator {
    pub fn new(heap_ptr: NonNull<u8>, heap_size: usize) -> Self {
        unsafe {
            let head_ptr = heap_ptr.as_ptr() as *mut MemoryBlock;

            head_ptr.write(MemoryBlock {
                size: 1 << heap_size.ilog2(),
                next: None,
            });

            let head = NonNull::new(head_ptr);

            Self { head }
        }
    }

    fn split(&mut self, left_block: NonNull<MemoryBlock>) -> NonNull<MemoryBlock> {
        unsafe {
            let left_ptr = left_block.as_ptr();
            (*left_ptr).size /= 2;

            let right_ptr = left_block.byte_add((*left_ptr).size);
            right_ptr.write(*left_ptr);

            (*left_ptr).next = Some(right_ptr);

            right_ptr
        }
    }

    fn merge(&mut self, merge_block: &mut NonNull<MemoryBlock>) {
        unsafe {
            loop {
                let mut previous_block: Option<NonNull<MemoryBlock>> = None;
                let mut current_block = self.head;

                let mut merged_at_all = false;

                while let Some(free_block) = current_block {
                    let merge_block_ptr = merge_block.as_ptr();
                    let merge_size = (*merge_block_ptr).size;
                    let free_block_ptr = free_block.as_ptr();
                    let free_size = (*free_block_ptr).size;

                    let mut merged_now = false;

                    if free_block_ptr.byte_add(free_size) == merge_block_ptr {
                        (*free_block_ptr).size += merge_size;
                        *merge_block = free_block;

                        merged_at_all = true;
                        merged_now = true;
                    } else if merge_block_ptr.byte_add(merge_size) == free_block_ptr {
                        (*merge_block_ptr).size += free_size;

                        merged_at_all = true;
                        merged_now = true;
                    }

                    if merged_now {
                        if let Some(previous_block) = previous_block {
                            (*previous_block.as_ptr()).next = (*free_block_ptr).next;
                        } else {
                            self.head = (*free_block_ptr).next;
                        }
                    }

                    previous_block = current_block;
                    current_block = (*free_block_ptr).next;
                }

                if !merged_at_all {
                    break;
                }
            }
        }
    }

    pub fn calculate_free_space(&self) -> usize {
        let mut amount = 0;

        unsafe {
            let mut current_block = self.head;

            while let Some(free_block) = current_block {
                let free_block_ptr = free_block.as_ptr();
                let free_size = (*free_block_ptr).size;

                amount += free_size - size_of::<MemoryBlock>();

                current_block = (*free_block_ptr).next;
            }
        }

        amount
    }
}

#[repr(transparent)]
pub struct LockedBuddyAllocator(pub Lazy<Mutex<BuddyAllocator>>);

unsafe impl Send for LockedBuddyAllocator {}
unsafe impl Sync for LockedBuddyAllocator {}

impl Deref for LockedBuddyAllocator {
    type Target = Mutex<BuddyAllocator>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

unsafe impl GlobalAlloc for LockedBuddyAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let Ok(allocation) = self.allocate(layout) else {
            return core::ptr::null_mut();
        };

        allocation.as_ptr() as *mut u8
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

        let aligned_data_size = layout.pad_to_align().size();

        let allocation_size = size_of::<MemoryBlock>() + aligned_data_size;

        let mut previous_block = None;
        let mut current_block = allocator.head;

        while let Some(free_block) = current_block {
            let free_block_ptr = free_block.as_ptr();

            unsafe {
                let mut free_size = (*free_block_ptr).size;

                if free_size < allocation_size {
                    previous_block = current_block;
                    current_block = (*free_block_ptr).next;

                    continue;
                }

                loop {
                    if (free_size / 2) > allocation_size {
                        allocator.split(free_block);

                        free_size = (*free_block_ptr).size;

                        continue;
                    }

                    let data_ptr = free_block
                        .as_ptr()
                        .cast::<u8>()
                        .byte_add(size_of::<MemoryBlock>());

                    let data = core::slice::from_raw_parts_mut(data_ptr, aligned_data_size);

                    if let Some(previous_block) = previous_block {
                        (*previous_block.as_ptr()).next = (*free_block.as_ptr()).next;
                    } else {
                        allocator.head = (*free_block.as_ptr()).next;
                    }

                    return Ok(data.into());
                }
            }
        }

        Err(AllocError)
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, _layout: Layout) {
        let mut allocator = self.lock();

        unsafe {
            let mut deallocation_block =
                ptr.byte_sub(size_of::<MemoryBlock>()).cast::<MemoryBlock>();

            allocator.merge(&mut deallocation_block);

            if let Some(head) = allocator.head {
                (*deallocation_block.as_ptr()).next = (*head.as_ptr()).next;
                (*head.as_ptr()).next = Some(deallocation_block);
            } else {
                (*deallocation_block.as_ptr()).next = None;
                allocator.head = Some(deallocation_block);
            }
        }
    }
}
