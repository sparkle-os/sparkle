//! Serial port driver.
use core::fmt;
use spin::RwLock;
use x86_64::instructions::port::Port;

macro_rules! port {
    ($name:ident, $addr:expr) => {
        lazy_static! {
            pub static ref $name: RwLock<SerialPort> = RwLock::new(SerialPort::new($addr));
        }
    };
}

port!(COM1, 0x3f8);

pub struct SerialPort {
    data_port: Port<u8>,
    status_port: Port<u8>,
}

impl SerialPort {
    pub fn new(port: u16) -> SerialPort {
        unsafe {
            // Disable all interrupts
            Port::<u8>::new(port + 1).write(0x00);
            // enable baud rate divisor (DLAB)
            Port::<u8>::new(port + 3).write(1 << 7);
            // set divisor to 38400 baud
            Port::<u8>::new(port + 0).write(0x03); // divisor high byte
            Port::<u8>::new(port + 1).write(0x00); // divisor low byte

            // 8 bit transmissions; no parity; one stop bit
            Port::<u8>::new(port + 3).write(0x03);
            // enable FIFO, clear, set 14 byte threshold
            Port::<u8>::new(port + 2).write(0xc7);
            // enable IRQs, set RTS/DSR.
            Port::<u8>::new(port + 4).write(0x0b);
        }

        SerialPort {
            data_port: Port::<u8>::new(port),
            status_port: Port::<u8>::new(port + 5),
        }
    }

    /// Returns `true` when the UART has new data available to read
    pub fn has_new_data(&self) -> bool {
        unsafe { self.status_port.read() & 1 != 0 }
    }

    /// Returns `true` when the UART still has queued data to send
    pub fn is_tx_empty(&self) -> bool {
        unsafe { self.status_port.read() & 0x20 == 0x20 }
    }

    /// Reads a byte from the UART, blocking until data is available.
    pub fn read_byte(&self) -> u8 {
        while !self.has_new_data() {}
        unsafe { self.data_port.read() }
    }

    /// Writes a byte to the UART, blocking until it is possible to send.
    pub fn write_byte(&mut self, byte: u8) {
        while !self.is_tx_empty() {}
        unsafe { self.data_port.write(byte) }
    }
}

impl fmt::Write for SerialPort {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            self.write_byte(byte);
        }

        Ok(())
    }
}
