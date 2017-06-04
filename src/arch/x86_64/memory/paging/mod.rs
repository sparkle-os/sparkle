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
use self::temporary_page::TemporaryPage;
use self::mapper::Mapper;

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

    /// Executes a closure, with a different page table recursively mapped
    pub fn with<F>(&mut self, table: &mut InactivePageTable, scratch_page: &mut TemporaryPage, f: F)
            where F: FnOnce(&mut Mapper) {
        use x86::shared::{tlb, control_regs};

        {
            // Backup the original P4 pointer
            let backup = Frame::containing_address(
                unsafe {control_regs::cr3()}
            );

            // Map a scratch page to the current p4 table
            let p4_table = scratch_page.map_table_frame(backup.clone(), self);

            // Overwrite main P4 recursive mapping
            self.p4_mut()[511].set(table.p4_frame.clone(), PRESENT | WRITABLE);
            unsafe {tlb::flush_all();} // flush *all* TLBs to prevent fuckiness

            // Execute f in context of the new page table
            f(self);

            // Restore the original pointer to P4
            p4_table[511].set(backup, PRESENT | WRITABLE);
            unsafe {tlb::flush_all();} // prevent fuckiness
        }

        scratch_page.unmap(self);
    }

    /// Switches to a new [`InactivePageTable`], making it active.
    ///
    /// Note: We don't need to flush the TLB here, as the CPU automatically flushes
    /// the TLB when the P4 table is switched.
    pub fn switch(&mut self, new_table: InactivePageTable) -> InactivePageTable {
        use x86::shared::{control_regs};

        let old_table = InactivePageTable {
            p4_frame: Frame::containing_address(unsafe {control_regs::cr3()}),
        };

        unsafe {
            control_regs::cr3_write(new_table.p4_frame.start_address());
        }

        old_table
    }
}

/// Owns an inactive P4 table.
pub struct InactivePageTable {
    p4_frame: Frame,
}

impl InactivePageTable {
    pub fn new(frame: Frame, active_table: &mut ActivePageTable, temporary_page: &mut TemporaryPage)
            -> InactivePageTable {
        {
            let table = temporary_page.map_table_frame(frame.clone(), active_table);

            // zero the new inactive page table
            table.zero();

            // set up a recursive mapping for this table
            table[511].set(frame.clone(), PRESENT | WRITABLE);
        }
        temporary_page.unmap(active_table);

        InactivePageTable { p4_frame: frame }
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
