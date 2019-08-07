use super::Frame;
use multiboot2::{MemoryArea, MemoryAreaIter};

/// A trait which can be implemented by any frame allocator, to make the frame allocation system
/// pluggable.
pub trait FrameAllocator {
    fn alloc_frame(&mut self) -> Option<Frame>;
    fn dealloc_frame(&mut self, frame: Frame);
}

pub struct AreaFrameAllocator<'a> {
    next_free_frame: Frame,

    current_area: Option<&'a MemoryArea>,
    areas: MemoryAreaIter<'a>,

    kernel_start: Frame,
    kernel_end: Frame,
    multiboot_start: Frame,
    multiboot_end: Frame,
}

impl<'a> FrameAllocator for AreaFrameAllocator<'a> {
    fn alloc_frame(&mut self) -> Option<Frame> {
        if let Some(area) = self.current_area {
            // This is the next frame up for allocation
            let frame = self.next_free_frame.clone();

            // Calculate the current area's last frame
            let current_area_last_frame =
                { Frame::containing_address(area.end_address() as usize - 1) };

            // Check if the frame we're considering is OK; if it is, we'll return it,
            // if not, we'll update the frame we're looking at and try again.
            if frame > current_area_last_frame {
                // If the frame we wanted to allocate is past the end of the current frame,
                // switch to the next area
                self.next_area();
            } else if frame >= self.kernel_start && frame <= self.kernel_end {
                // The frame under consideration is used by the kernel,
                // so jump over the kernel code area.
                self.next_free_frame = self.kernel_end.next_frame();
            } else if frame >= self.multiboot_start && frame <= self.multiboot_end {
                // The frame under consideration is used by Multiboot,
                // so jump over the multiboot area.
                self.next_free_frame = self.multiboot_end.next_frame();
            } else {
                // Frame is unused!
                self.next_free_frame = Frame::new(self.next_free_frame.index() + 1); // We'll consider the next frame next time we need to alloc
                return Some(frame);
            }
            // The frame we were looking at wasn't valid; try again with our updated `next_free_frame`
            return self.alloc_frame();
        } else {
            None // no free frames left!
        }
    }

    /// EXTREME BADNESS
    /// TODO: fix this
    #[allow(unused_variables)]
    fn dealloc_frame(&mut self, frame: Frame) {
        unimplemented!();
    }
}

impl<'a> AreaFrameAllocator<'a> {
    pub fn new(
        kernel_start: usize,
        kernel_end: usize,
        multiboot_start: usize,
        multiboot_end: usize,
        memory_areas: MemoryAreaIter<'a>,
    ) -> AreaFrameAllocator<'a> {
        let mut allocator = AreaFrameAllocator {
            next_free_frame: Frame::containing_address(0x0),
            current_area: None,
            areas: memory_areas,
            kernel_start: Frame::containing_address(kernel_start),
            kernel_end: Frame::containing_address(kernel_end),
            multiboot_start: Frame::containing_address(multiboot_start),
            multiboot_end: Frame::containing_address(multiboot_end),
        };
        allocator.next_area();
        allocator
    }

    fn next_area(&mut self) {
        self.current_area = self
            .areas
            .clone()
            .filter(|area| {
                Frame::containing_address(area.end_address() as usize - 1) >= self.next_free_frame
            })
            .min_by_key(|area| area.start_address());

        if let Some(area) = self.current_area {
            let start_frame = Frame::containing_address(area.start_address() as usize);
            if self.next_free_frame < start_frame {
                self.next_free_frame = start_frame;
            }
        }
    }
}
