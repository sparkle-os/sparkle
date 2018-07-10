//! Physical page frames.

/// A page frame in physical memory.
///
/// # Limitations
/// Currently, page frames are all the same size (4kib) and represented internally with an index,
/// starting with frame 0 at physical memory location `0x0`.
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Frame {
    index: usize,
}

impl Frame {
    /// The physical size of each frame.
    pub const SIZE: usize = 4096;

    pub fn new(index: usize) -> Frame {
        Frame { index }
    }

    pub fn index(&self) -> usize {
        self.index
    }

    /// Retrieves the frame containing a particular physical address.
    pub fn containing_address(address: usize) -> Frame {
        Frame {
            index: address / Self::SIZE,
        }
    }

    /// Returns the frame after this one.
    pub fn next_frame(&self) -> Frame {
        Frame {
            index: self.index + 1,
        }
    }

    /// Returns the starting address of a frame.
    pub fn start_address(&self) -> usize {
        self.index * Self::SIZE
    }

    /// Returns an iterator of all the frames between `start` and `end` (inclusive).
    pub fn range_inclusive(start: Frame, end: Frame) -> FrameIter {
        FrameIter { start, end }
    }

    /// Clones the Frame; we implement this instead of deriving Clone since deriving clone
    /// makes `.clone()` public, which would be illogical here (frames should not be cloned by end-users,
    /// as that could be used to cause, *e.g.*, double-free errors with the `FrameAllocator`).
    pub(super) fn clone(&self) -> Frame {
        Frame { index: self.index }
    }
}

/// An iterator over a range of page frames.
pub struct FrameIter {
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
