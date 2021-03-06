#![cfg_attr(feature = "cargo-clippy", allow(needless_return))]
use super::table::{self, EntryFlags, Level4, Table, ENTRY_COUNT};
use super::{Frame, Page, PhysicalAddress, VirtualAddress};
use arch::x86_64::memory::FrameAllocator;
use core::ptr::NonNull;

/// Owns the top-level active page table (P4).
pub struct Mapper {
    p4: NonNull<Table<Level4>>,
}

impl Mapper {
    /// There **must** be ***only one*** Mapper instance.
    /// Since we cannot guarantee this trivially, the constructor is unsafe.
    pub unsafe fn new() -> Mapper {
        Mapper {
            p4: NonNull::new_unchecked(table::P4),
        }
    }

    pub fn p4(&self) -> &Table<Level4> {
        unsafe { self.p4.as_ref() }
    }

    pub fn p4_mut(&mut self) -> &mut Table<Level4> {
        unsafe { self.p4.as_mut() }
    }

    /// Translates a given virtual address to a physical address.
    pub fn translate(&self, virtual_address: VirtualAddress) -> Option<PhysicalAddress> {
        let offset = virtual_address % Frame::SIZE; // offset into the frame
        self.page_to_frame(Page::containing_address(virtual_address))
            .map(|frame| frame.index() * Frame::SIZE + offset)
    }

    /// Translates a given virtual page to a physical frame.
    pub fn page_to_frame(&self, page: Page) -> Option<Frame> {
        let p3 = self.p4().next_table(page.p4_index());

        let handle_huge_page = || {
            p3.and_then(|p3| {
                let p3_entry = &p3[page.p3_index()];

                // Is this a 1GiB page?
                if let Some(start_frame) = p3_entry.pointed_frame() {
                    if p3_entry.flags().contains(EntryFlags::HUGE_PAGE) {
                        // 1GiB pages must be 1GiB-aligned
                        assert!(
                            start_frame.index() % (ENTRY_COUNT * ENTRY_COUNT) == 0,
                            "1GiB hugepages must be 1GiB-aligned"
                        );

                        return Some(Frame::new(
                            start_frame.index() + page.p2_index() * ENTRY_COUNT + page.p1_index(),
                        ));
                    }
                }

                if let Some(p2) = p3.next_table(page.p3_index()) {
                    let p2_entry = &p2[page.p2_index()];

                    // Is this a 2MiB page?
                    if let Some(start_frame) = p2_entry.pointed_frame() {
                        if p2_entry.flags().contains(EntryFlags::HUGE_PAGE) {
                            // 2MiB pages must be 2MiB-aligned
                            assert!(
                                start_frame.index() % ENTRY_COUNT == 0,
                                "2MiB pages must be 2MiB-aligned"
                            );

                            return Some(Frame::new(start_frame.index() + page.p1_index()));
                        }
                    }
                }

                // Didn't find a huge page
                return None;
            })
        };

        p3.and_then(|p3| p3.next_table(page.p3_index()))
            .and_then(|p2| p2.next_table(page.p2_index()))
            .and_then(|p1| p1[page.p1_index()].pointed_frame())
            .or_else(handle_huge_page)
    }

    /// Maps a virtual page to a physical frame.
    pub fn map_to<A>(&mut self, page: Page, frame: Frame, flags: EntryFlags, allocator: &mut A)
    where
        A: FrameAllocator,
    {
        let p3 = self.p4_mut().next_table_create(page.p4_index(), allocator);
        let p2 = p3.next_table_create(page.p3_index(), allocator);
        let p1 = p2.next_table_create(page.p2_index(), allocator);

        assert!(
            p1[page.p1_index()].is_unused(),
            "Attempting to map Page->Frame but a P1 entry for this Page already exists!"
        );
        p1[page.p1_index()].set(frame, flags | EntryFlags::PRESENT);
    }

    /// Maps a virtual page to a physical frame, automatically picking the frame.
    pub fn map<A>(&mut self, page: Page, flags: EntryFlags, allocator: &mut A)
    where
        A: FrameAllocator,
    {
        let frame = allocator
            .alloc_frame()
            .expect("Attempted to allocate a frame to map to a page, but no frames are available!");
        self.map_to(page, frame, flags, allocator);
    }

    /// Maps a physical frame to a page with the same address in virtual memory
    pub fn identity_map<A>(&mut self, frame: Frame, flags: EntryFlags, allocator: &mut A)
    where
        A: FrameAllocator,
    {
        let page = Page::containing_address(frame.start_address());
        self.map_to(page, frame, flags, allocator);
    }

    /// Unmaps a virtual page.
    #[allow(unused_variables)]
    pub fn unmap<A>(&mut self, page: Page, allocator: &mut A)
    where
        A: FrameAllocator,
    {
        assert!(
            self.translate(page.start_address()).is_some(),
            "Attempted to unmap a page which is not mapped!"
        );

        let p1 = self
            .p4_mut()
            .next_table_mut(page.p4_index())
            .and_then(|p3| p3.next_table_mut(page.p3_index()))
            .and_then(|p2| p2.next_table_mut(page.p2_index()))
            .expect("Mapping code does not support huge pages.");
        let frame = p1[page.p1_index()].pointed_frame().unwrap();
        p1[page.p1_index()].set_unused();

        use x86_64::instructions::tlb;
        tlb::flush(::x86_64::VirtAddr::new(page.start_address() as u64));

        // TODO free p(1,2,3) table if empty
        // allocator.dealloc_frame(frame);
    }
}
