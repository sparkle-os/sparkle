//! Paging subsystem. *Note: uses recursive mapping.*
//!
//! Extremely ripped off from Phil Oppermann's tutorials, because I don't feel like writing
//! a paging system off the top of my head today.

use core::ptr::Unique;
use super::PAGE_SIZE;
use super::{Frame, FrameAllocator};

mod entry;
mod table;

use self::entry::*;
use self::table::{Table, Level4};

/// Upper bound on entries per page table
const ENTRY_COUNT: usize = 512;

/// Helper type aliases used to make function signatures more expressive
pub type PhysicalAddress = usize;
pub type VirtualAddress = usize;

/// Owns the top-level active page table (P4).
pub struct ActivePageTable {
    p4: Unique<Table<Level4>>,
}

impl ActivePageTable {
    /// There **must** be ***only one*** ActivePageTable instance.
    /// Since we cannot guarantee this trivially, the constructor is unsafe.
    pub unsafe fn new() -> ActivePageTable {
        ActivePageTable {
            p4: Unique::new(table::P4),
        }
    }
}

/// A representation of a virtual page.
#[derive(Clone, Copy, Debug)]
pub struct Page {
    index: usize,
}
