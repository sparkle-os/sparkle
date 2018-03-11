//! Paging subsystem. *Note: uses recursive mapping.*
//!
//! Extremely ripped off from Phil Oppermann's tutorials, because I don't feel like writing
//! a paging system off the top of my head today.

#![cfg_attr(feature = "cargo-clippy", allow(unreadable_literal))]

use core::ops::{Add, Deref, DerefMut};
use multiboot2::BootInformation;
use super::PAGE_SIZE;
use super::{Frame, FrameAllocator};

mod entry;
mod table;
mod mapper;
mod temporary_page;

pub use self::entry::*;
use self::table::Table;
use self::temporary_page::TemporaryPage;
use self::mapper::Mapper;

/// Upper bound on entries per page table
const ENTRY_COUNT: usize = 512;

/// Helper type aliases used to make function signatures more expressive
pub type PhysicalAddress = usize;
pub type VirtualAddress = usize;

pub struct ActivePageTable {
    mapper: Mapper,
}

impl Deref for ActivePageTable {
    type Target = Mapper;
    fn deref(&self) -> &Mapper {
        &self.mapper
    }
}

impl DerefMut for ActivePageTable {
    fn deref_mut(&mut self) -> &mut Mapper {
        &mut self.mapper
    }
}

impl ActivePageTable {
    unsafe fn new() -> ActivePageTable {
        ActivePageTable {
            mapper: Mapper::new(),
        }
    }

    /// Executes a closure, with a different page table recursively mapped
    pub fn with<F>(&mut self, table: &mut InactivePageTable, scratch_page: &mut TemporaryPage, f: F)
    where
        F: FnOnce(&mut Mapper),
    {
        use x86::instructions::tlb;
        use x86::registers::control_regs;

        {
            // Backup the original P4 pointer
            let backup = Frame::containing_address(control_regs::cr3().0 as usize);

            // Map a scratch page to the current p4 table
            let p4_table = scratch_page.map_table_frame(backup.clone(), self);

            // Overwrite main P4 recursive mapping
            self.p4_mut()[511].set(table.p4_frame.clone(), PRESENT | WRITABLE);
            tlb::flush_all(); // flush *all* TLBs to prevent fuckiness

            // Execute f in context of the new page table
            f(self);

            // Restore the original pointer to P4
            p4_table[511].set(backup, PRESENT | WRITABLE);
            tlb::flush_all(); // prevent fuckiness
        }

        scratch_page.unmap(self);
    }

    /// Switches to a new [`InactivePageTable`], making it active.
    ///
    /// Note: We don't need to flush the TLB here, as the CPU automatically flushes
    /// the TLB when the P4 table is switched.
    pub fn switch(&mut self, new_table: InactivePageTable) -> InactivePageTable {
        use x86::registers::control_regs;

        let old_table = InactivePageTable {
            p4_frame: Frame::containing_address(control_regs::cr3().0 as usize),
        };

        unsafe {
            control_regs::cr3_write(
                ::x86::PhysicalAddress(new_table.p4_frame.start_address() as u64),
            );
        }

        old_table
    }
}

/// Owns an inactive P4 table.
pub struct InactivePageTable {
    p4_frame: Frame,
}

impl InactivePageTable {
    pub fn new(
        frame: Frame,
        active_table: &mut ActivePageTable,
        temporary_page: &mut TemporaryPage,
    ) -> InactivePageTable {
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
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Page {
    index: usize,
}

impl Page {
    /// Retrieves the page containing a given virtual address.
    pub fn containing_address(address: VirtualAddress) -> Page {
        assert!(
            address < 0x0000_8000_0000_0000 || address >= 0xffff_8000_0000_0000,
            "invalid address: {:#x}",
            address
        );

        Page {
            index: address / PAGE_SIZE,
        }
    }

    /// Returns the start (virtual) address of a page
    pub fn start_address(&self) -> VirtualAddress {
        self.index * PAGE_SIZE
    }

    pub fn range_inclusive(start: Page, end: Page) -> PageIter {
        PageIter {
            start: start,
            end: end,
        }
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

impl Add<usize> for Page {
    type Output = Page;

    fn add(self, rhs: usize) -> Page {
        Page {
            index: self.index + rhs,
        }
    }
}

#[derive(Clone)]
pub struct PageIter {
    start: Page,
    end: Page,
}

impl Iterator for PageIter {
    type Item = Page;

    fn next(&mut self) -> Option<Page> {
        if self.start <= self.end {
            let frame = self.start.clone();
            self.start.index += 1;
            Some(frame)
        } else {
            None
        }
    }
}

/// Remap the kernel
pub fn remap_kernel<A>(allocator: &mut A, boot_info: &BootInformation) -> ActivePageTable
where
    A: FrameAllocator,
{
    let mut scratch_page = TemporaryPage::new(Page { index: 0xabadcafe }, allocator);

    let mut active_table = unsafe { ActivePageTable::new() };
    let mut new_table = {
        let frame = allocator.alloc_frame().expect(
            "Attempted to allocate a frame for a new page table, but no frames are available!",
        );
        InactivePageTable::new(frame, &mut active_table, &mut scratch_page)
    };

    active_table.with(&mut new_table, &mut scratch_page, |mapper| {
        let elf_sections_tag = boot_info
            .elf_sections_tag()
            .expect("ELF sections tag required!");

        // -- Identity map the kernel sections
        for section in elf_sections_tag.sections() {
            if !section.is_allocated() {
                // section is not loaded to memory
                continue;
            }

            assert!(
                section.start_address() % PAGE_SIZE == 0,
                "ELF sections must be page-aligned!"
            );
            debug!(
                "Mapping section at addr: {:#x}, size: {:#x}",
                section.addr, section.size
            );

            let flags = EntryFlags::from_elf_section_flags(section);
            let start_frame = Frame::containing_address(section.start_address());
            let end_frame = Frame::containing_address(section.end_address() - 1);
            for frame in Frame::range_inclusive(start_frame, end_frame) {
                mapper.identity_map(frame, flags, allocator);
            }
        }

        // -- Identity map the VGA console buffer (it's only one frame long)
        let vga_buffer_frame = Frame::containing_address(0xb8000);
        mapper.identity_map(vga_buffer_frame, WRITABLE, allocator);

        // -- Identity map the multiboot info structure
        let multiboot_start = Frame::containing_address(boot_info.start_address());
        let multiboot_end = Frame::containing_address(boot_info.end_address() - 1);
        for frame in Frame::range_inclusive(multiboot_start, multiboot_end) {
            mapper.identity_map(frame, PRESENT | WRITABLE, allocator);
        }
    });

    let old_table = active_table.switch(new_table);
    info!("kremap: successful table switch");

    // Create a guard page in place of the old P4 table's page
    let old_p4_page = Page::containing_address(old_table.p4_frame.start_address());
    active_table.unmap(old_p4_page, allocator);
    info!(
        "kremap: guard page established at {:#x}",
        old_p4_page.start_address()
    );

    active_table
}
