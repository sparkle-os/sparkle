//! Bullshit naïve bump allocator for kernel heap testing
//! Lovingly ripped off from^W^W^W inspired by phil-opp's rust os articles

#![feature(const_fn)]
#![no_std]


#[derive(Debug)]
pub struct Heap {
    /// Start of the heap we're allocating into.
    heap_start: usize,
    /// End of the heap we're allocating into.
    heap_end: usize,

    /// The next free address in the heap. Absolute address, *not* relative to `heap_start`!
    next_addr: usize,
}

impl Heap {
    /// Create a new allocator using the given range
    /// [heap_start, heap_start+heap_size] for the heap
    pub const fn new(heap_start: usize, heap_size: usize) -> Heap {
        Heap {
            heap_start: heap_start,
            heap_end: heap_start + heap_size,

            next_addr: heap_start,
        }
    }

    /// Allocate a chunk of memory with (size, alignment)
    pub fn allocate(&mut self, size: usize, align: usize) -> Option<*mut u8> {
        let alloc_start = align_up(self.next_addr, align);
        let alloc_end = alloc_start.saturating_add(size);

        if alloc_end <= self.heap_end {
            self.next_addr = alloc_end;
            Some(alloc_start as *mut u8)
        } else {
            None
        }
    }

    pub fn deallocate(&mut self, _ptr: *mut u8, _size: usize, _align: usize) {
        // nothing! just leak
    }
}

// unsafe impl Alloc for BumpAllocator {
//     #[inline]
//     unsafe fn alloc(&mut self, layout: Layout) -> Result<*mut u8, AllocErr> {
//         self.allocate(layout.size(), layout.align()).ok_or(AllocErr::Exhausted {request: layout})
//     }

//     unsafe fn dealloc(&mut self, _ptr: *mut u8, _layout: Layout) {
//         // just leak
//     }

//     fn oom(&mut self, err: AllocErr) -> ! {
//         panic!("kheap OOM: {:?}!", err);
//     }

//     // Just copy the whole thing up to a new block
//     unsafe fn realloc(&mut self, ptr: *mut u8, layout: Layout, new_layout: Layout) -> Result<*mut u8, AllocErr> {

//         use core::{ptr, cmp};

//         let copy_size = cmp::min(layout.size(), new_layout.size()); // copy len = min(size, size')
//         let new_ptr = self.alloc(new_layout)?; // alloc new block
//         ptr::copy(ptr, new_ptr, copy_size);
//         self.dealloc(ptr, layout); // dealloc old pointer

//         Ok(new_ptr)
//     }
// }

pub fn align_down(addr: usize, align: usize) -> usize {
    if align.is_power_of_two() {
        // unset bits after alignment bit
        addr & !(align - 1)
    } else if align == 0 {
        addr
    } else {
        panic!("`align` must be a power of 2");
    }
}

pub fn align_up(addr: usize, align: usize) -> usize {
    align_down(addr + align - 1, align)
}
