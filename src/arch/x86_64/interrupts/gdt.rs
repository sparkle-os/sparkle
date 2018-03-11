use core::mem::size_of;
use x86::structures::tss::TaskStateSegment;
use x86::structures::gdt::SegmentSelector;
use x86::PrivilegeLevel;

pub struct Gdt {
    table: [u64; 8],
    next_free: usize,
}

impl Gdt {
    pub fn new() -> Gdt {
        Gdt {
            table: [0; 8],
            next_free: 1, // 0th entry is always 0
        }
    }

    pub fn add_entry(&mut self, entry: Descriptor) -> SegmentSelector {
        let idx = match entry {
            Descriptor::UserSegment(value) => self.push(value),
            Descriptor::SystemSegment(low, high) => {
                let idx = self.push(low);
                self.push(high);

                idx
            }
        };

        SegmentSelector::new(idx as u16, PrivilegeLevel::Ring0)
    }

    pub fn load(&'static self) {
        use x86::instructions::tables::{lgdt, DescriptorTablePointer};

        let ptr = DescriptorTablePointer {
            base: self.table.as_ptr() as u64,
            limit: (self.table.len() * size_of::<u64>() - 1) as u16,
        };

        unsafe {
            lgdt(&ptr);
        }
    }

    fn push(&mut self, value: u64) -> usize {
        if self.next_free < self.table.len() {
            let idx = self.next_free;
            self.table[idx] = value;
            self.next_free += 1;

            idx
        } else {
            // we don't need more than a handful of GDT entries.
            // hitting this would indicate dev error
            panic!("gdt: tried to push() but it's full!");
        }
    }
}

pub enum Descriptor {
    UserSegment(u64),
    SystemSegment(u64, u64),
}

impl Descriptor {
    pub fn kernel_code_segment() -> Descriptor {
        let flags = USER_SEGMENT | PRESENT | EXECUTABLE | LONG_MODE;

        Descriptor::UserSegment(flags.bits())
    }

    pub fn tss_segment(tss: &'static TaskStateSegment) -> Descriptor {
        use bit_field::BitField;

        let ptr = tss as *const _ as u64;

        let mut low = PRESENT.bits();
        // point to the TSS (low)
        low.set_bits(16..40, ptr.get_bits(0..24));
        low.set_bits(56..64, ptr.get_bits(24..32));

        // limit of the TSS
        low.set_bits(0..16, (size_of::<TaskStateSegment>() - 1) as u64);

        // type. 0b1001 => available, 64 bit, tss
        low.set_bits(40..44, 0b1001);

        let mut high = 0;
        high.set_bits(0..32, ptr.get_bits(32..64));

        Descriptor::SystemSegment(low, high)
    }
}

bitflags! {
    struct DescriptorFlags: u64 {
        const CONFORMING    = 1 << 42;
        const EXECUTABLE    = 1 << 43;
        const USER_SEGMENT  = 1 << 44;
        const PRESENT       = 1 << 47;
        const LONG_MODE     = 1 << 53;
    }
}
