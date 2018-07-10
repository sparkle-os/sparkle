//! Representation and operations on page tables.

use super::{Frame, FrameAllocator};
use core::marker::PhantomData;
use core::ops::{Index, IndexMut};
use multiboot2::ElfSection;

pub trait TableLevel {}
pub enum Level4 {}
pub enum Level3 {}
pub enum Level2 {}
pub enum Level1 {}
impl TableLevel for Level4 {}
impl TableLevel for Level3 {}
impl TableLevel for Level2 {}
impl TableLevel for Level1 {}
pub trait HierarchicalLevel: TableLevel {
    type NextLevel: TableLevel;
}
impl HierarchicalLevel for Level4 {
    type NextLevel = Level3;
}
impl HierarchicalLevel for Level3 {
    type NextLevel = Level2;
}
impl HierarchicalLevel for Level2 {
    type NextLevel = Level1;
}

pub const P4: *mut Table<Level4> = 0xffffffff_fffff000 as *mut _;

/// Upper bound on entries per page table
pub const ENTRY_COUNT: usize = 512;

pub struct Table<L: TableLevel> {
    entries: [Entry; ENTRY_COUNT],
    level: PhantomData<L>,
}

impl<L> Table<L>
where
    L: TableLevel,
{
    pub fn zero(&mut self) {
        for entry in self.entries.iter_mut() {
            entry.set_unused();
        }
    }
}

impl<L> Table<L>
where
    L: HierarchicalLevel,
{
    fn next_table_address(&self, index: usize) -> Option<usize> {
        let entry_flags = self[index].flags();
        if entry_flags.contains(EntryFlags::PRESENT) && !entry_flags.contains(EntryFlags::HUGE_PAGE)
        {
            let table_address = self as *const _ as usize;
            Some((table_address << 9) | (index << 12))
        } else {
            None
        }
    }

    pub fn next_table(&self, index: usize) -> Option<&Table<L::NextLevel>> {
        self.next_table_address(index)
            .map(|addr| unsafe { &*(addr as *const _) })
    }

    pub fn next_table_mut(&self, index: usize) -> Option<&mut Table<L::NextLevel>> {
        self.next_table_address(index)
            .map(|addr| unsafe { &mut *(addr as *mut _) })
    }

    pub fn next_table_create<A>(
        &mut self,
        index: usize,
        allocator: &mut A,
    ) -> &mut Table<L::NextLevel>
    where
        A: FrameAllocator,
    {
        if self.next_table(index).is_none() {
            assert!(!self.entries[index].flags().contains(EntryFlags::HUGE_PAGE),
                "Attempted to create a subtable for a hugepage; we do not currently support hugepages.");
            let frame = allocator.alloc_frame().expect(
                "Attempted to allocate a frame for a subtable, but no frames are available!",
            );

            self.entries[index].set(frame, EntryFlags::PRESENT | EntryFlags::WRITABLE);
            // Zero the new table
            self.next_table_mut(index).unwrap().zero();
        }

        self.next_table_mut(index).unwrap()
    }
}

impl<L> Index<usize> for Table<L>
where
    L: TableLevel,
{
    type Output = Entry;

    fn index(&self, index: usize) -> &Entry {
        &self.entries[index]
    }
}

impl<L> IndexMut<usize> for Table<L>
where
    L: TableLevel,
{
    fn index_mut(&mut self, index: usize) -> &mut Entry {
        &mut self.entries[index]
    }
}

/// A page table entry.
pub struct Entry(u64);

impl Entry {
    pub fn is_unused(&self) -> bool {
        self.0 == 0
    }

    pub fn set_unused(&mut self) {
        self.0 = 0;
    }

    pub fn flags(&self) -> EntryFlags {
        EntryFlags::from_bits_truncate(self.0)
    }

    pub fn pointed_frame(&self) -> Option<Frame> {
        if self.flags().contains(EntryFlags::PRESENT) {
            Some(Frame::containing_address(
                self.0 as usize & 0x000fffff_fffff000,
            ))
        } else {
            None
        }
    }

    pub fn set(&mut self, frame: Frame, flags: EntryFlags) {
        // Frame physical address must be page-aligned and smaller than 2^52
        assert!(
            frame.start_address() & !0x000fffff_fffff000 == 0,
            "Frame physical addresses must be page-aligned and smaller than 2^52!"
        );

        self.0 = (frame.start_address() as u64) | flags.bits();
    }
}

bitflags! {
    pub struct EntryFlags: u64 {
        const PRESENT =         1 << 0;
        const WRITABLE =        1 << 1;
        const USER_ACCESSIBLE = 1 << 2;
        const WRITE_THROUGH =   1 << 3;
        const NO_CACHE =        1 << 4;
        const ACCESSED =        1 << 5;
        const DIRTY =           1 << 6;
        const HUGE_PAGE =       1 << 7;
        const GLOBAL =          1 << 8;
        const NO_EXECUTE =      1 << 63;
    }
}

impl EntryFlags {
    pub fn from_elf_section_flags(section: &ElfSection) -> EntryFlags {
        use multiboot2::ElfSectionFlags;

        let mut flags = EntryFlags::empty();

        if section.flags().contains(ElfSectionFlags::ALLOCATED) {
            flags |= EntryFlags::PRESENT;
        }
        if section.flags().contains(ElfSectionFlags::WRITABLE) {
            flags |= EntryFlags::WRITABLE;
        }
        if !section.flags().contains(ElfSectionFlags::EXECUTABLE) {
            flags |= EntryFlags::NO_EXECUTE;
        }

        flags
    }
}
