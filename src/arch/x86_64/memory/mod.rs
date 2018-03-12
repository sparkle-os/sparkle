//! Memory management for x86_64 platforms.
//!
//! Heavly inspired/lovingly ripped off from Phil Oppermann's [os.phil-opp.com](http://os.phil-opp.com/).

mod area_frame_allocator;
mod stack_allocator;
mod paging;

use multiboot2::BootInformation;
use arch::x86_64;
use alloca;

pub use self::area_frame_allocator::AreaFrameAllocator;
use self::paging::{ActivePageTable, Page};

pub use self::stack_allocator::{Stack, StackAllocator};

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
        Frame {
            index: address / PAGE_SIZE,
        }
    }

    /// Returns the frame after this one.
    fn next_frame(&self) -> Frame {
        Frame {
            index: self.index + 1,
        }
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

pub struct MemoryController {
    active_table: ActivePageTable,
    frame_allocator: AreaFrameAllocator,
    stack_allocator: StackAllocator,
}

impl MemoryController {
    /// Allocates and returns a stack.
    ///
    /// Note: `size` is given in pages.
    pub fn alloc_stack(&mut self, size: usize) -> Option<Stack> {
        self.stack_allocator
            .alloc_stack(&mut self.active_table, &mut self.frame_allocator, size)
    }
}

pub fn init(boot_info: &BootInformation) -> MemoryController {
    assert_first_call!("memory::init() can only be called once!");

    let memory_map_tag = boot_info
        .memory_map_tag()
        .expect("multiboot: Memory map tag required");
    let elf_sections_tag = boot_info
        .elf_sections_tag()
        .expect("multiboot: ELF sections tag required");

    info!("memory areas:");
    for area in memory_map_tag.memory_areas() {
        info!("  start: {:#x}, length: {:#x}", area.base_addr, area.length);
    }

    let kernel_start = elf_sections_tag
        .sections()
        .filter(|s| s.is_allocated())
        .map(|s| s.addr)
        .min()
        .unwrap();
    let kernel_end = elf_sections_tag
        .sections()
        .filter(|s| s.is_allocated())
        .map(|s| s.addr + s.size)
        .max()
        .unwrap();

    info!(
        "kernel start: {:#x}, kernel end: {:#x}",
        kernel_start, kernel_end
    );
    info!(
        "multiboot start: {:#x}, multiboot end: {:#x}",
        boot_info.start_address(),
        boot_info.end_address()
    );

    let mut frame_allocator = AreaFrameAllocator::new(
        kernel_start as usize,
        kernel_end as usize,
        boot_info.start_address(),
        boot_info.end_address(),
        memory_map_tag.memory_areas(),
    );

    // Enable required CPU features
    x86_64::bits::enable_nxe(); // Enable NO_EXECUTE pages
    x86_64::bits::enable_wrprot(); // Disable writing to non-WRITABLE pages

    let mut active_table = paging::remap_kernel(&mut frame_allocator, boot_info);
    info!("paging: remapped kernel");

    use alloca::{HEAP_SIZE, HEAP_START};
    let heap_start_page = Page::containing_address(HEAP_START);
    let heap_end_page = Page::containing_address(HEAP_START + HEAP_SIZE - 1);

    // TODO: map these pages lazily
    for page in Page::range_inclusive(heap_start_page, heap_end_page) {
        active_table.map(page, paging::WRITABLE, &mut frame_allocator);
    }

    unsafe {
        alloca::heap_init(HEAP_START, HEAP_SIZE);
    }
    info!("kheap: initialized");

    let stack_allocator = {
        let alloc_start = heap_end_page + 1;
        let alloc_end = alloc_start + 100;
        let alloc_range = Page::range_inclusive(alloc_start, alloc_end);

        StackAllocator::new(alloc_range)
    };

    MemoryController {
        active_table: active_table,
        frame_allocator: frame_allocator,
        stack_allocator: stack_allocator,
    }
}
