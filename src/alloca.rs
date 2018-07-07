//! Wires rust up to the kheap, so that `alloc::` works.

#![cfg_attr(feature="cargo-clippy", allow(inconsistent_digit_grouping))]

use spin::Mutex;
use linked_list_allocator::Heap;
use core::ptr::NonNull;
use alloc::allocator::{GlobalAlloc, Alloc, AllocErr, Layout};

/// Base location of the kheap.
pub const HEAP_START: usize = 0o_000_001_000_000_0000;
/// Size of the kheap.
pub const HEAP_SIZE: usize = 100 * 1024; // 100 KiB

/// Locked ownership of an optional kheap. Initialized on boot; is `None` before then.
static HEAP: Mutex<Option<Heap>> = Mutex::new(None);

/// Initialize the kheap. Called at boot.
pub unsafe fn heap_init(start: usize, size: usize) {
    *HEAP.lock() = Some(Heap::new(start, size));
}

/// Wraps whatever allocator backend we're using, and implements `alloc::allocator::Alloc`.
pub struct Allocator;

unsafe impl<'a> Alloc for &'a Allocator {
    unsafe fn alloc(&mut self, layout: Layout) -> Result<NonNull<u8>, AllocErr> {
        if let Some(ref mut heap) = *HEAP.lock() {
            heap.allocate_first_fit(layout)
        } else {
            panic!("kheap: attempting alloc w/ uninitialized heap");
        }
    }

    unsafe fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout) {
        if let Some(ref mut heap) = *HEAP.lock() {
            heap.deallocate(ptr, layout)
        } else {
            panic!("kheap: attempting dealloc w/ uninitialized heap");
        }
    }
}

unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if let Some(ref mut heap) = *HEAP.lock() {
            heap.allocate_first_fit(layout).ok().map_or(0 as *mut u8, |ptr| {ptr.as_ptr()})
        } else {
            panic!("kheap: attempting alloc w/ uninitialized heap");
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if let Some(ref mut heap) = *HEAP.lock() {
            heap.deallocate(NonNull::new_unchecked(ptr), layout)
        } else {
            panic!("kheap: attempting dealloc w/ uninitialized heap");
        }
    }
}
