//! Representation and operations on page tables.

use core::ops::{Index, IndexMut};
use core::marker::PhantomData;
use ::arch::x86_64::memory::FrameAllocator;
use super::entry::*;
use super::ENTRY_COUNT;

pub trait TableLevel {}
pub enum Level4 {}
pub enum Level3 {}
pub enum Level2 {}
pub enum Level1 {}
impl TableLevel for Level4 {}
impl TableLevel for Level3 {}
impl TableLevel for Level2 {}
impl TableLevel for Level1 {}
trait HierarchicalLevel: TableLevel {type NextLevel: TableLevel;}
impl HierarchicalLevel for Level4 {type NextLevel = Level3;}
impl HierarchicalLevel for Level3 {type NextLevel = Level2;}
impl HierarchicalLevel for Level2 {type NextLevel = Level1;}

pub const P4: *mut Table<Level4> = 0xffffffff_fffff000 as *mut _;

pub struct Table<L: TableLevel> {
    entries: [Entry; ENTRY_COUNT],
    level: PhantomData<L>,
}

impl<L> Table<L> where L: TableLevel {
    pub fn zero(&mut self) {
        for entry in self.entries.iter_mut() {
            entry.set_unused();
        }
    }
}


impl<L> Index<usize> for Table<L> where L: TableLevel {
    type Output = Entry;

    fn index(&self, index: usize) -> &Entry {
         &self.entries[index]
    }
}

impl<L> IndexMut<usize> for Table<L> where L: TableLevel {
    fn index_mut(&mut self, index: usize) -> &mut Entry {
         &mut self.entries[index]
    }
}
