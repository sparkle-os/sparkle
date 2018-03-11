use arch::x86_64::memory::{Frame, FrameAllocator};
use super::{ActivePageTable, Page, Table};
use super::table::Level1;
use super::VirtualAddress;

pub struct TemporaryPage {
    page: Page,
    allocator: TinyAllocator,
}

impl TemporaryPage {
    pub fn new<A>(page: Page, allocator: &mut A) -> TemporaryPage
    where
        A: FrameAllocator,
    {
        TemporaryPage {
            page: page,
            allocator: TinyAllocator::new(allocator),
        }
    }

    /// Maps the temporary page to the given frame, using the active table.
    /// Returns the start address of the temporary page in VRAM.
    pub fn map(&mut self, frame: Frame, active_table: &mut ActivePageTable) -> VirtualAddress {
        use super::entry::WRITABLE;

        assert!(
            active_table.page_to_frame(self.page).is_none(),
            "Temporary page is already mapped!"
        );
        active_table.map_to(self.page, frame, WRITABLE, &mut self.allocator);

        self.page.start_address()
    }

    /// Maps the temporary page to the given page table frame, using the active table.
    /// Returns a &mut reference to the now-mapped table.
    pub fn map_table_frame(
        &mut self,
        frame: Frame,
        active_table: &mut ActivePageTable,
    ) -> &mut Table<Level1> {
        unsafe { &mut *(self.map(frame, active_table) as *mut Table<Level1>) }
    }

    pub fn unmap(&mut self, active_table: &mut ActivePageTable) {
        active_table.unmap(self.page, &mut self.allocator);
    }
}

/// A tiny frame allocator which can only hold three frames,
/// used to temporarily create a P3, P2, and P1 table.
struct TinyAllocator([Option<Frame>; 3]);
impl TinyAllocator {
    fn new<A>(allocator: &mut A) -> TinyAllocator
    where
        A: FrameAllocator,
    {
        let mut f = || allocator.alloc_frame();
        let frames = [f(), f(), f()];
        TinyAllocator(frames)
    }
}
impl FrameAllocator for TinyAllocator {
    fn alloc_frame(&mut self) -> Option<Frame> {
        for frame in &mut self.0 {
            if frame.is_some() {
                return frame.take();
            }
        }
        None
    }

    fn dealloc_frame(&mut self, frame: Frame) {
        for stored_frame in &mut self.0 {
            if stored_frame.is_none() {
                *stored_frame = Some(frame);
                return;
            }
        }

        panic!("TinyAllocator can only hold 3 frames!");
    }
}
