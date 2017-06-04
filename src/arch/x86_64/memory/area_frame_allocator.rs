use arch::x86_64::memory::{Frame, FrameAllocator};
use multiboot2::{MemoryArea, MemoryAreaIter};

pub struct AreaFrameAllocator {
    next_free_frame: Frame,

    current_area: Option<&'static MemoryArea>,
    areas: MemoryAreaIter,

    kernel_start: Frame,
    kernel_end: Frame,
    multiboot_start: Frame,
    multiboot_end: Frame,
}

impl FrameAllocator for AreaFrameAllocator {
    fn alloc_frame(&mut self) -> Option<Frame> {
        if let Some(area) = self.current_area {
            // This is the next frame up for allocation
            let frame = Frame {index: self.next_free_frame.index};

            // Calculate the current area's last frame
            let current_area_last_frame = {
                let addr = area.base_addr + area.length - 1;
                Frame::containing_address(addr as usize)
            };

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
                self.next_free_frame.index += 1; // We'll consider the next frame next time we need to alloc
                return Some(frame);
            }
            // The frame we were looking at wasn't valid; try again with our updated `next_free_frame`
            return self.alloc_frame();
        } else {
            None // no free frames left!
        }
    }

    fn dealloc_frame(&mut self, frame: Frame) {
        unimplemented!();
    }
}

impl AreaFrameAllocator {
    pub fn new(kernel_start: usize, kernel_end: usize,
           multiboot_start: usize, multiboot_end: usize,
           memory_areas: MemoryAreaIter) -> AreaFrameAllocator {
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
        self.current_area = self.areas.clone().filter(|area| {
            let address = area.base_addr + area.length - 1;
            Frame::containing_address(address as usize) >= self.next_free_frame
        }).min_by_key(|area| area.base_addr);

        if let Some(area) = self.current_area {
            let start_frame = Frame::containing_address(area.base_addr as usize);
            if self.next_free_frame < start_frame {
                self.next_free_frame = start_frame;
            }
        }
    }
}
