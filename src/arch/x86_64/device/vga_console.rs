//! A basic console driver, using the VGA text-mode buffer.

use core::fmt;
use spin::Mutex;
use volatile::Volatile;
use x86::instructions::port as io;

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_pos: 0, row_pos: 0,
        style: CharStyle::new(Color::Cyan, Color::DarkGray),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

/// Foreground + background color pair for a VGA console cell.
#[derive(Clone, Copy, Debug)]
pub struct CharStyle(u8);
impl CharStyle {
    pub const fn new(foreground: Color, background: Color) -> CharStyle {
        CharStyle((background as u8) << 4 | (foreground as u8))
    }
}

/// Character/style pair.
#[derive(Clone, Copy, Debug)]
#[repr(C)]
struct VgaChar {
    ascii_char: u8,
    style: CharStyle,
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

/// Shadows the actual VGA text-mode buffer at `0xb8000`.
struct Buffer {
    chars: [[Volatile<VgaChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

/// Console-oriented abstraction layer used to write into the VGA text buffer.
/// Maintains a cursor and current style.
pub struct Writer {
    /// Current position of the 'write cursor'
    row_pos: usize,
    column_pos: usize,
    /// Current style we're writing with
    style: CharStyle,
    /// This is set up on initialization to shadow `0xb8000`, the VGA text-mode buffer.
    buffer: &'static mut Buffer,
}

#[allow(dead_code)]
impl Writer {
    /// Write a single byte into the VGA buffer at the cursor location.
    /// Increments the cursor location and wraps to the next line if necessary.
    pub fn write_byte(&mut self, byte: u8, style: CharStyle) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                let row = self.row_pos;
                let col = self.column_pos;

                self.write_byte_at(byte, row, col, style);

                self.column_pos += 1;
                if self.column_pos >= BUFFER_WIDTH {
                    self.new_line();
                }
            }
        }
    }

    /// Write a single byte at (row, col).
    pub fn write_byte_at(&mut self, byte: u8, row: usize, col: usize, style: CharStyle) {
        self.buffer.chars[row][col].write(VgaChar {
            ascii_char: byte,
            style,
        });
    }

    /// Clear the VGA buffer.
    pub fn clear_screen(&mut self) {
        let clear_style = VgaChar {
            ascii_char: b' ',
            style: self.style,
        };
        for row in 0..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                self.buffer.chars[row][col].write(clear_style);
            }
        }
    }

    pub fn set_style(&mut self, style: CharStyle) {
        self.style = style;
    }

    pub fn style(&self) -> CharStyle {
        self.style
    }

    pub fn styled(&mut self) -> StyledWriter {
        StyledWriter {
            style: self.style,
            inner: self,
        }
    }

    /// Move the _console cursor_ (blinky bar) to (row, col).
    fn move_cursor(&mut self, row: usize, col: usize) {
        assert!(
            row < BUFFER_HEIGHT,
            "attempted out-of-bounds (row) cursor move"
        );
        assert!(
            col < BUFFER_WIDTH,
            "attempted out-of-bounds (col) cursor move"
        );

        let pos: u16 = ((row * 80) + col) as u16;
        // Lovingly ripped off from wiki.osdev.org/Text_Mode_Cursor
        unsafe {
            io::outb(0x3d4, 0x0F);
            io::outb(0x3d5, (pos & 0xff) as u8);

            io::outb(0x3d4, 0x0e);
            io::outb(0x3d5, ((pos >> 8) & 0xff) as u8);
        }
    }

    /// Move the internal cursor to a new line, scrolling if necessary.
    fn new_line(&mut self) {
        self.column_pos = 0;
        self.row_pos += 1;

        if self.row_pos >= BUFFER_HEIGHT {
            self.scroll();
        }
    }

    /// Clear a `row` of the VGA buffer.
    fn clear_row(&mut self, row: usize) {
        let clear_style = VgaChar {
            ascii_char: b' ',
            style: self.style,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(clear_style);
        }
    }

    /// Scroll the buffer up by one row.
    fn scroll(&mut self) {
        for row in 0..(BUFFER_HEIGHT - 1) {
            for col in 0..BUFFER_WIDTH {
                let c = self.buffer.chars[row + 1][col].read();
                self.buffer.chars[row][col].write(c);
            }
        }

        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_pos = 0;
        self.row_pos = BUFFER_HEIGHT - 1;
    }

    pub fn write_str_with_style(&mut self, s: &str, style: CharStyle) {
        for byte in s.bytes() {
            self.write_byte(byte, style);
        }

        let row = self.row_pos;
        let col = self.column_pos;
        self.move_cursor(row, col);
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let sty = self.style();
        self.write_str_with_style(s, sty);

        Ok(())
    }
}

pub struct StyledWriter<'a> {
    inner: &'a mut Writer,
    style: CharStyle,
}

impl<'a> StyledWriter<'a> {
    pub fn set_style(mut self, style: CharStyle) -> Self {
        self.style = style; self
    }
}

impl<'a> fmt::Write for StyledWriter<'a> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.inner.write_str_with_style(s, self.style);

        Ok(())
    }
}
