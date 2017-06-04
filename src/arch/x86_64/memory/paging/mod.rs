//! Paging subsystem.
//!
//! Extremely ripped off from Phil Oppermann's tutorials, because I don't feel like writing
//! a paging system off the top of my head today.

use super::PAGE_SIZE;

mod entry;
mod table;

/// Upper bound on entries per page table
const ENTRY_COUNT: usize = 512;

/// Helper type aliases used to make function signatures more expressive
pub type PhysicalAddress = usize;
pub type VirtualAddress = usize;

/// A representation of a virtual page
pub struct Page {
    index: usize,
}
