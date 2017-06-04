//! A basic console driver, using the VGA text-mode buffer.

use core::fmt;
use core::ptr::Unique;
use spin::Mutex;
use volatile::Volatile;

pub static WRITER: Mutex<Writer> = Mutex::new(Writer {
    column_pos: 0, row_pos: 0,
    style: CharStyle::new(Color::Cyan, Color::DarkGray),
    buffer: unsafe {Unique::new(0xb8000 as *mut _)},
});

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum Color {
    Black      = 0,
    Blue       = 1,
    Green      = 2,
    Cyan       = 3,
    Red        = 4,
    Magenta    = 5,
    Brown      = 6,
    LightGray  = 7,
    DarkGray   = 8,
    LightBlue  = 9,
    LightGreen = 10,
    LightCyan  = 11,
    LightRed   = 12,
    Pink       = 13,
    Yellow     = 14,
    White      = 15,
}

#[derive(Clone, Copy, Debug)]
pub struct CharStyle(u8);
impl CharStyle {
    pub const fn new(foreground: Color, background: Color) -> CharStyle {
        CharStyle((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
struct VgaChar {
    ascii_char: u8,
    style: CharStyle,
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

/// Shadows the actual VGA text-mode buffer at 0xb8000.
struct Buffer {
    chars: [[Volatile<VgaChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

/// Abstraction layer used to write into the VGA text buffer
pub struct Writer {
    /// Current position of the 'write cursor'
    row_pos: usize, column_pos: usize,
    /// Current style we're writing with
    style: CharStyle,
    /// This is set up on initialization to shadow `0xb8000`, the VGA text-mode buffer.
    buffer: Unique<Buffer>,
}

#[allow(dead_code)]
impl Writer {
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_pos >= BUFFER_WIDTH {self.new_line();}

                let row = self.row_pos;
                let col = self.column_pos;
                let style = self.style;

                self.buffer().chars[row][col].write(VgaChar {
                    ascii_char: byte,
                    style: style,
                });

                self.column_pos += 1;
            }
        }
    }

    pub fn write_byte_at(&mut self, byte: u8, row: usize, col: usize) {
        self.row_pos = row; self.column_pos = col;
        let style = self.style;

        self.buffer().chars[row][col].write(VgaChar {ascii_char: byte, style: style});
    }

    pub fn clear_screen(&mut self) {
        let clear_style = VgaChar {ascii_char:  b' ', style: self.style};
        for row in 0..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                self.buffer().chars[row][col].write(clear_style);
            }
        }
    }

    pub fn set_style(&mut self, style: CharStyle) {
        self.style = style;
    }

    pub fn get_style(&self) -> CharStyle {
        self.style
    }

    fn new_line(&mut self) {
        self.column_pos = 0;
        self.row_pos += 1;

        if self.row_pos >= BUFFER_HEIGHT {
            self.scroll();
        }
    }

    fn clear_row(&mut self, row: usize) {
        let clear_style = VgaChar {ascii_char:  b' ', style: self.style};
        for col in 0..BUFFER_WIDTH {
            self.buffer().chars[row][col].write(clear_style);
        }
    }

    fn scroll(&mut self){
        for row in 0..(BUFFER_HEIGHT - 1) {
            for col in 0..BUFFER_WIDTH {
                let c = self.buffer().chars[row + 1][col].read();
                self.buffer().chars[row][col].write(c);
            }
        }

        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_pos = 0;
        self.row_pos = BUFFER_HEIGHT - 1;
    }

    fn buffer(&mut self) -> &mut Buffer {
        unsafe { self.buffer.as_mut() }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            self.write_byte(byte);
        }

        Ok(())
    }
}
