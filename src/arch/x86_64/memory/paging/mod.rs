//! Paging subsystem. *Note: uses recursive mapping.*
//!
//! Extremely ripped off from Phil Oppermann's tutorials, because I don't feel like writing
//! a paging system off the top of my head today.

use super::PAGE_SIZE;
use super::{Frame, FrameAllocator};

mod entry;
mod table;
mod mapper;

use self::entry::*;
use self::table::{Table, Level4};

/// Upper bound on entries per page table
const ENTRY_COUNT: usize = 512;

/// Helper type aliases used to make function signatures more expressive
pub type PhysicalAddress = usize;
pub type VirtualAddress = usize;

pub struct ActivePageTable {
    mapper: Mapper
}

impl Deref for ActivePageTable {
    type Target = Mapper;
    fn deref(&self) -> &Mapper {&self.mapper}
}

impl DerefMut for ActivePageTable {
    fn deref_mut(&mut self) -> &mut Mapper {&mut self.mapper}
}

impl ActivePageTable {
    unsafe fn new() -> ActivePageTable {
        ActivePageTable {
            mapper: Mapper::new(),
        }
    }









        };


    }




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

/// Temporary function to test paging
pub fn test_paging<A>(allocator: &mut A) where A: FrameAllocator {
    let mut page_table = unsafe {ActivePageTable::new()};

    // test stuff

    let addr = 42 * 515 * 515 * 4096;
    let page = Page::containing_address(addr);
    let frame = allocator.alloc_frame().expect("no more frames");
    println!("vaddr->phys = {:?}, map to {:?}", page_table.translate(addr), frame);
    page_table.map_to(page, frame, EntryFlags::empty(), allocator);
    println!("vaddr->phys = {:?} (after mapping)", page_table.translate(addr));
    println!("next frame: {:?}", allocator.alloc_frame());

    println!("{:#x}", unsafe {
        *(Page::containing_address(addr).start_address() as *const u64)
    });

    page_table.unmap(Page::containing_address(addr), allocator);
    println!("vaddr->phys = {:?} (should be None)", page_table.translate(addr));

    // println!("{:#x}", unsafe {
    //     *(Page::containing_address(addr).start_address() as *const u64)
    // });
}
