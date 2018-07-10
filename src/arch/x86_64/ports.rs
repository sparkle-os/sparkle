use core::marker::PhantomData;
use x86_64::instructions::port::*;

/// Marker trait for types that can be used as an IO port value type.
pub trait IoType {}
impl IoType for u8 {}
impl IoType for u16 {}
impl IoType for u32 {}

/// An x86 IO port, parametrized over the value type we're reading/writing.
pub struct Port<T>
where
    T: IoType,
{
    port: u16,
    value_t: PhantomData<T>,
}

impl<T> Port<T>
where
    T: IoType,
{
    pub const fn new(port: u16) -> Port<T> {
        Port {
            port,
            value_t: PhantomData,
        }
    }
}

macro_rules! make_port_impl {
    ($type:ident, $set_fn:ident, $get_fn:ident) => {
        #[allow(dead_code)]
        impl Port<$type> {
            pub unsafe fn read(&self) -> $type { $get_fn(self.port) }
            pub unsafe fn write(&mut self, value: $type) { $set_fn(self.port, value) }
        }
    }
}

make_port_impl!(u8, outb, inb);
make_port_impl!(u16, outw, inw);
make_port_impl!(u32, outl, inl);
