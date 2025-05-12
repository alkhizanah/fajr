#[derive(Debug, Clone, Copy)]
#[repr(C, packed(4))]
pub struct TaskStateSegment {
    reserved_1: u32,
    pub privilege_stack_table: [u64; 3],
    reserved_2: u64,
    pub interrupt_stack_table: [u64; 7],
    reserved_3: u64,
    reserved_4: u16,
    pub iomap_base: u16,
}

impl TaskStateSegment {
    pub const fn new() -> Self {
        Self {
            privilege_stack_table: [0; 3],
            interrupt_stack_table: [0; 7],
            iomap_base: size_of::<TaskStateSegment>() as u16,
            reserved_1: 0,
            reserved_2: 0,
            reserved_3: 0,
            reserved_4: 0,
        }
    }
}
