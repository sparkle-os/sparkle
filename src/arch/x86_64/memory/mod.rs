//! Memory management for x86_64 platforms.
//!
//! Heavly inspired/lovingly ripped off from Phil Oppermann's [os.phil-opp.com](http://os.phil-opp.com/).

mod area_frame_allocator;
mod paging;

pub use self::area_frame_allocator::AreaFrameAllocator;

pub use self::paging::remap_kernel;

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

    /// Returns an iterator of all the frames between `start` and `end` (inclusive).
    fn range_inclusive(start: Frame, end: Frame) -> FrameIter {
        FrameIter {
            start: start,
            end: end,
        }
    }

    /// Clones the Frame; we implement this instead of deriving Clone since deriving clone
    /// makes `.clone()` public, which would be illogical here (frames should not be cloned by end-users,
    /// as that could be used to cause, *e.g.*, double-free errors with the `FrameAllocator`).
    fn clone(&self) -> Frame {
        Frame { index: self.index }
    }
}

struct FrameIter {
    start: Frame,
    end: Frame,
}

impl Iterator for FrameIter {
    type Item = Frame;

    fn next(&mut self) -> Option<Frame> {
        if self.start <= self.end {
            let frame = self.start.clone();
            self.start.index += 1;
            Some(frame)
        } else {
            None
        }
    }
}


/// A trait which can be implemented by any frame allocator, to make the frame allocation system
/// pluggable.
pub trait FrameAllocator {
    fn alloc_frame(&mut self) -> Option<Frame>;
    fn dealloc_frame(&mut self, frame: Frame);
}
