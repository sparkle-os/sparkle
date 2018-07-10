use super::{Frame, VirtualAddress};
use core::ops::Add;

/// A representation of a virtual page.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Page {
    index: usize,
}

impl Page {
    /// Retrieves the page containing a given virtual address.
    pub fn containing_address(address: VirtualAddress) -> Page {
        assert!(
            address < 0x0000_8000_0000_0000 || address >= 0xffff_8000_0000_0000,
            "invalid address: {:#x}",
            address
        );

        Page {
            index: address / Frame::SIZE,
        }
    }

    pub fn new(index: usize) -> Page {
        Page { index }
    }

    /// Returns the start (virtual) address of a page
    pub fn start_address(&self) -> VirtualAddress {
        self.index * Frame::SIZE
    }

    pub fn range_inclusive(start: Page, end: Page) -> PageIter {
        PageIter { start, end }
    }

    pub fn p4_index(&self) -> usize {
        (self.index >> 27) & 0o777
    }
    pub fn p3_index(&self) -> usize {
        (self.index >> 18) & 0o777
    }
    pub fn p2_index(&self) -> usize {
        (self.index >> 9) & 0o777
    }
    pub fn p1_index(&self) -> usize {
        (self.index >> 0) & 0o777
    }
}

impl Add<usize> for Page {
    type Output = Page;

    fn add(self, rhs: usize) -> Page {
        Page {
            index: self.index + rhs,
        }
    }
}

#[derive(Clone)]
pub struct PageIter {
    start: Page,
    end: Page,
}

impl Iterator for PageIter {
    type Item = Page;

    fn next(&mut self) -> Option<Page> {
        if self.start <= self.end {
            let frame = self.start.clone();
            self.start.index += 1;
            Some(frame)
        } else {
            None
        }
    }
}
