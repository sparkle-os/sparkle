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

impl Page {
    /// Retrieves the page containing a given virtual address.
    pub fn containing_address(address: VirtualAddress) -> Page {
        assert!(address < 0x0000_8000_0000_0000 ||
            address >= 0xffff_8000_0000_0000,
            "invalid address: 0x{:x}", address);

        Page { index: address / PAGE_SIZE }
    }

    /// Returns the start (virtual) address of a page
    pub fn start_address(&self) -> VirtualAddress {
        self.index * PAGE_SIZE
    }

    fn p4_index(&self) -> usize {
        (self.index >> 27) & 0o777
    }
    fn p3_index(&self) -> usize {
        (self.index >> 18) & 0o777
    }
    fn p2_index(&self) -> usize {
        (self.index >> 9) & 0o777
    }
    fn p1_index(&self) -> usize {
        (self.index >> 0) & 0o777
    }
}
