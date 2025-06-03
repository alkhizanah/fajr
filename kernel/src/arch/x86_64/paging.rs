use core::{
    alloc::{GlobalAlloc, Layout},
    arch::asm,
    ops::{Deref, Index, IndexMut},
};

use bit_field::BitField;

use crate::{memory::PAGE_ALLOCATOR, paging};

const TABLE_ENTRY_COUNT: usize = 512;
pub const MIN_PAGE_SIZE: usize = 4096;

#[derive(Debug, Clone)]
#[repr(C, align(4096))]
pub struct PageTable {
    entries: [Entry; TABLE_ENTRY_COUNT],
}

impl PageTable {
    pub fn empty() -> Self {
        Self {
            entries: [Entry(0); TABLE_ENTRY_COUNT],
        }
    }

    pub fn translate(&self, virt: usize) -> Option<usize> {
        let offset = *PageTableOffset::from(virt);
        let indices = PageTableIndices::from(virt);

        let mut table = self;

        for (level, index) in [indices.p4_index, indices.p3_index, indices.p2_index]
            .into_iter()
            .enumerate()
            .map(|(l, i)| (4 - l, i))
        {
            let entry = &table[index];

            if !entry.is_present() {
                return None;
            }

            if entry.is_huge() {
                match level {
                    2 => {
                        return Some(
                            entry.get_phys() as usize
                                | ((indices.p1_index as usize) << 12)
                                | offset as usize,
                        );
                    }

                    3 => {
                        return Some(
                            entry.get_phys() as usize
                                | ((indices.p2_index as usize) << 12 << 9)
                                | ((indices.p1_index as usize) << 12)
                                | offset as usize,
                        );
                    }

                    _ => panic!("huge page bit must not be set in a level {} page", level),
                }
            }

            table = entry.get_page_table();
        }

        let entry = &table[indices.p1_index];

        if entry.is_present() {
            Some(entry.get_phys() as usize | offset as usize)
        } else {
            None
        }
    }

    pub fn map(&mut self, virt: usize, phys: usize) -> &mut Entry {
        debug_assert_eq!(virt % MIN_PAGE_SIZE, 0);
        debug_assert_eq!(phys % MIN_PAGE_SIZE, 0);

        let indices = PageTableIndices::from(virt);

        let mut table = self;

        for index in [indices.p4_index, indices.p3_index, indices.p2_index] {
            let entry = &mut table[index];

            if !entry.is_present() {
                let new_table_virt = unsafe { PAGE_ALLOCATOR.alloc(Layout::new::<PageTable>()) };

                unsafe {
                    new_table_virt.cast::<PageTable>().write(PageTable::empty());
                }

                let new_table_phys = get_active_table()
                    .translate(new_table_virt as usize)
                    .unwrap();

                entry.set_phys(new_table_phys as usize as u64);
                entry.set_present(true);
                entry.set_writable(true);
            } else if entry.is_huge() {
                todo!("mapping a huge page");
            }

            table = entry.get_page_table();
        }

        let entry = &mut table[indices.p1_index];

        let was_present = entry.is_present();

        entry.set_phys(phys as u64);
        entry.set_present(true);

        if was_present {
            unsafe { asm!("invlpg [{}]", in(reg) virt, options(preserves_flags)) }
        }

        entry
    }

    pub fn unmap(&mut self, virt: usize) {
        debug_assert_eq!(virt % MIN_PAGE_SIZE, 0);

        let indices = PageTableIndices::from(virt);

        let mut table = self;

        for index in [indices.p4_index, indices.p3_index, indices.p2_index] {
            let entry = &table[index];

            if !entry.is_present() {
                return;
            }

            if entry.is_huge() {
                todo!("unmapping a huge page");
            }

            table = entry.get_page_table();
        }

        let entry = &mut table[indices.p1_index];

        let was_present = entry.is_present();

        entry.set_present(false);

        if was_present {
            unsafe { asm!("invlpg [{}]", in(reg) virt, options(preserves_flags)) }
        }
    }
}

#[derive(Debug)]
#[repr(transparent)]
struct PageTableOffset(u16);

impl From<usize> for PageTableOffset {
    fn from(virt: usize) -> Self {
        Self((virt % (1 << 12)) as u16)
    }
}

impl Deref for PageTableOffset {
    type Target = u16;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug)]
struct PageTableIndices {
    p1_index: u16,
    p2_index: u16,
    p3_index: u16,
    p4_index: u16,
}

impl From<usize> for PageTableIndices {
    fn from(virt: usize) -> Self {
        Self {
            p1_index: ((virt >> 12) % 512) as u16,
            p2_index: ((virt >> 12 >> 9) % 512) as u16,
            p3_index: ((virt >> 12 >> 9 >> 9) % 512) as u16,
            p4_index: ((virt >> 12 >> 9 >> 9 >> 9) % 512) as u16,
        }
    }
}

impl Index<u16> for PageTable {
    type Output = Entry;

    fn index(&self, index: u16) -> &Self::Output {
        &self.entries[index as usize]
    }
}

impl IndexMut<u16> for PageTable {
    fn index_mut(&mut self, index: u16) -> &mut Self::Output {
        &mut self.entries[index as usize]
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(transparent)]
pub struct Entry(u64);

impl Entry {
    /// Get the physical address stored in this entry
    #[inline]
    pub fn get_phys(&self) -> u64 {
        self.0.get_bits(12..51) << 12
    }

    /// Set the physical address stored in this entry
    #[inline]
    pub fn set_phys(&mut self, phys: u64) -> &mut Entry {
        self.0.set_bits(12..51, phys >> 12);
        self
    }

    /// Get the page table that this entry is pointing to
    #[inline]
    pub fn get_page_table(&self) -> &'static mut PageTable {
        unsafe { &mut *(paging::offset(self.get_phys() as usize) as *mut PageTable) }
    }

    /// Whether the mapped frame or page table is loaded in memory.
    #[inline]
    pub fn is_present(&self) -> bool {
        self.0.get_bit(0)
    }

    /// Specifies whether the mapped frame or page table is loaded in memory.
    #[inline]
    pub fn set_present(&mut self, is_present: bool) -> &mut Self {
        self.0.set_bit(0, is_present);
        self
    }

    /// Controls whether writes to the mapped frames are allowed.
    ///
    /// If this bit is unset in a level 1 page table entry, the mapped frame is read-only.
    /// If this bit is unset in a higher level page table entry the complete range of mapped
    /// pages is read-only.
    #[inline]
    pub fn set_writable(&mut self, is_writable: bool) -> &mut Self {
        self.0.set_bit(1, is_writable);
        self
    }

    /// Controls whether accesses from userspace (i.e. ring 3) are permitted.
    #[inline]
    pub fn set_user_accessible(&mut self, is_user_accessible: bool) -> &mut Self {
        self.0.set_bit(2, is_user_accessible);
        self
    }

    /// If this bit is set, a “write-through” policy is used for the cache, else a “write-back”
    /// policy is used.
    #[inline]
    pub fn set_write_through(&mut self, can_write_through: bool) -> &mut Self {
        self.0.set_bit(3, can_write_through);
        self
    }

    /// Specifies whether the pointed entry is cachable.
    #[inline]
    pub fn set_cachability(&mut self, is_cachable: bool) -> &mut Self {
        // We do `!is_cachable` because enabling bit 4 disables the cachability
        self.0.set_bit(4, !is_cachable);
        self
    }

    /// Set by the CPU when the mapped frame or page table is accessed.
    #[inline]
    pub fn was_accessed(&self) -> bool {
        self.0.get_bit(5)
    }

    /// Set by the CPU on a write to the mapped frame.
    #[inline]
    pub fn was_written_to(&self) -> bool {
        self.0.get_bit(6)
    }

    /// Whether the entry maps a huge frame instead of a page table. Only allowed in
    /// P2 or P3 tables.
    #[inline]
    pub fn is_huge(&self) -> bool {
        self.0.get_bit(7)
    }

    /// Specifies that the entry maps a huge frame instead of a page table. Only allowed in
    /// P2 or P3 tables.
    #[inline]
    pub fn set_huge(&mut self, is_huge: bool) -> &mut Self {
        self.0.set_bit(7, is_huge);
        self
    }

    /// Indicates that the mapping is present in all address spaces, so it isn't flushed from
    /// the TLB on an address space switch.
    #[inline]
    pub fn set_global(&mut self, is_global: bool) -> &mut Self {
        self.0.set_bit(7, is_global);
        self
    }

    /// Whether code execution from the mapped frames is allowed.
    ///
    /// Can be only used when the no-execute page protection feature is enabled in the EFER
    /// register.
    #[inline]
    pub fn set_executability(&mut self, is_executable: bool) -> &mut Self {
        // We do `!is_executable` because enabling bit 63 disables the executability
        self.0.set_bit(63, !is_executable);
        self
    }
}

#[inline]
pub fn get_active_table() -> &'static mut PageTable {
    unsafe {
        let phys: usize;

        asm!("mov {}, cr3", out(reg) phys, options(preserves_flags));

        &mut *(paging::offset(phys) as *mut PageTable)
    }
}

#[inline]
pub fn set_active_table(p4_table: &'static mut PageTable) {
    let phys = get_active_table()
        .translate(p4_table as *const _ as usize)
        .unwrap();

    unsafe {
        asm!("mov cr3, {}", in(reg) phys, options(readonly, preserves_flags));
    }
}
