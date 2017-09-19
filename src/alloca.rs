use spin::Mutex;
use sparkle_bump_alloc::Heap;
use alloc::allocator::{Alloc, Layout, AllocErr};

pub const HEAP_START: usize = 0o_000_001_000_000_0000;
pub const HEAP_SIZE: usize = 100 * 1024; // 100 KiB

static HEAP: Mutex<Option<Heap>> = Mutex::new(None);

pub fn heap_init(start: usize, size: usize) {
    *HEAP.lock() = Some(Heap::new(start, size));
}

pub struct Allocator;

unsafe impl<'a> Alloc for &'a Allocator {
    #[inline]
    unsafe fn alloc(&mut self, layout: Layout) -> Result<*mut u8, AllocErr> {
        if let Some(ref mut heap) = *HEAP.lock() {
            heap.allocate(layout.size(), layout.align())
                .ok_or(AllocErr::Exhausted { request: layout })
        } else {
            panic!("kheap: attempting alloc w/ uninitialized heap");
        }
    }

    #[inline]
    unsafe fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
        if let Some(ref mut heap) = *HEAP.lock() {
            heap.deallocate(ptr, layout.size(), layout.align());
        } else {
            panic!("kheap: attempting dealloc w/ uninitialized heap");
        }
    }

    #[inline]
    fn oom(&mut self, err: AllocErr) -> ! {
        panic!("kheap OOM: {:?}", err);
    }
}
