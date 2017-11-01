use memory::paging::{self, Page, PageIter, ActivePageTable};
use memory::{PAGE_SIZE, FrameAllocator};

#[derive(Debug)]
pub struct Stack {
    top: usize,
    bottom: usize,
}

impl Stack {
    fn new(top: usize, bottom: usize) -> Stack {
        assert!(top > bottom,
            "Stack top must be higher in memory than the bottom");

        Stack {
            top: top,
            bottom: bottom,
        }
    }

    pub fn top(&self) -> usize    { self.top }
    pub fn bottom(&self) -> usize { self.bottom }
}


pub struct StackAllocator {
    range: PageIter,
}

impl StackAllocator {
    pub fn new(page_range: PageIter) -> StackAllocator {
        StackAllocator { range: page_range }
    }

    /// Allocate a stack.
    ///
    /// Note: `size` is given in pages.
    pub fn alloc_stack<A>(&mut self,
                 active_table: &mut ActivePageTable,
                 frame_alloc: &mut A,
                 size: usize) -> Option<Stack>
        where A: FrameAllocator
    {
        // zero-size stacks are nonsensical
        if size == 0 {
            return None;
        }

        let mut range = self.range.clone();

        // try to alloc stack, guard pages
        let guard_page = range.next();
        let stack_start = range.next();
        let stack_end = if size == 1 {
            stack_start
        } else {
            range.nth(size - 2)
        };

        match (guard_page, stack_start, stack_end) {
            (Some(_), Some(start), Some(end)) => {
                // writeback
                self.range = range;

                // map stack pages -> physical frames
                for page in Page::range_inclusive(start, end) {
                    active_table.map(page, paging::WRITABLE, frame_alloc);
                }

                // create a new stack

                let top = end.start_address() + PAGE_SIZE;

                Some(Stack::new(top, start.start_address()))
            }
            _ => None, // whoops not enough frames
        }
    }
}
