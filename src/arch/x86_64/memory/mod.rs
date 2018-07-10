//! Memory management for x86_64 platforms.
//!
//! Heavly inspired/lovingly ripped off from Phil Oppermann's [os.phil-opp.com](http://os.phil-opp.com/).

pub(crate) mod paging;
mod stack_allocator;

use alloca;
use arch::x86_64;
use multiboot2::BootInformation;

use self::paging::frame_allocators::AreaFrameAllocator;
use self::paging::{table, ActivePageTable, Frame, FrameAllocator, Page};

pub use self::stack_allocator::{Stack, StackAllocator};

pub struct MemoryController<A>
where
    A: FrameAllocator,
{
    active_table: ActivePageTable,
    frame_allocator: A,
    stack_allocator: StackAllocator,
}

impl<A> MemoryController<A>
where
    A: FrameAllocator,
{
    /// Allocates and returns a stack.
    ///
    /// Note: `size` is given in pages.
    pub fn alloc_stack(&mut self, size: usize) -> Option<Stack> {
        self.stack_allocator
            .alloc_stack(&mut self.active_table, &mut self.frame_allocator, size)
    }
}

pub fn init(boot_info: &BootInformation) -> MemoryController<AreaFrameAllocator> {
    assert_first_call!("memory::init() can only be called once!");

    let memory_map_tag = boot_info
        .memory_map_tag()
        .expect("multiboot: Memory map tag required");
    let elf_sections_tag = boot_info
        .elf_sections_tag()
        .expect("multiboot: ELF sections tag required");

    debug!("memory areas:");
    for area in memory_map_tag.memory_areas() {
        debug!(
            "  start: {:#x}, length: {:#x}",
            area.start_address(),
            area.size()
        );
    }

    let kernel_start = elf_sections_tag
        .sections()
        .filter(|s| s.is_allocated())
        .map(|s| s.start_address())
        .min()
        .unwrap();
    let kernel_end = elf_sections_tag
        .sections()
        .filter(|s| s.is_allocated())
        .map(|s| s.end_address())
        .max()
        .unwrap();

    debug!(
        "kernel start: {:#x}, kernel end: {:#x}",
        kernel_start, kernel_end
    );
    debug!(
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
        active_table.map(page, table::EntryFlags::WRITABLE, &mut frame_allocator);
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
        active_table,
        frame_allocator,
        stack_allocator,
    }
}
