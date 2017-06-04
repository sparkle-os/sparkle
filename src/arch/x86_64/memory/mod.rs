//! Memory management for x86_64 platforms.
//!
//! Heavly inspired/lovingly ripped off from Phil Oppermann's [os.phil-opp.com](http://os.phil-opp.com/).

mod area_frame_allocator;
mod paging;

pub use self::area_frame_allocator::AreaFrameAllocator;

/// The physical size of each frame.
pub const PAGE_SIZE: usize = 4096;

/// A representation of a frame in physical memory.
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Frame {
    index: usize,
}

impl Frame {
    /// Retrieves the frame containing a particular physical address.
    fn containing_address(address: usize) -> Frame {
        Frame {index: address/PAGE_SIZE}
    }

    /// Returns the frame after this one.
    fn next_frame(&self) -> Frame {
        Frame {index: self.index + 1}
    }

    fn start_address(&self) -> usize {
        self.index * PAGE_SIZE
    }
}

/// A trait which can be implemented by any frame allocator, to make the frame allocation system
/// pluggable.
pub trait FrameAllocator {
    fn alloc_frame(&mut self) -> Option<Frame>;
    fn dealloc_frame(&mut self, frame: Frame);
}
