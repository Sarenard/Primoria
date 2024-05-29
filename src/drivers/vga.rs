use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;

use vga::colors::Color16;
use vga::writers::{Graphics640x480x16, GraphicsWriter};

const WINDOW_HEIGHT: usize = 480;
const WINDOW_WIDTH: usize = 640;

pub struct VgaWriter {
    row_position: usize,
    column_position: usize,
    vga: Graphics640x480x16,
}

impl VgaWriter {
    pub fn write_byte(&mut self, byte: u8) {
        if self.column_position >= WINDOW_WIDTH {
            self.new_line();
        }
        match byte {
            b'\n' => self.new_line(),
            byte => {
                let row = self.row_position;
                let col = self.column_position;
                let color_code = Color16::White;
                self.vga.draw_character(col, row, byte as char, color_code);
            }
        }
        self.column_position += 8;
    }

    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // printable ASCII byte or newline
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                // not part of printable ASCII range
                _ => self.write_byte(0xfe),
            }
        }
    }

    #[allow(dead_code)]
    fn new_line(&mut self) {
        // TODO : scrolling
        self.row_position += 8;
        self.column_position = 0;
    }

    fn clear_row(&mut self, _row: usize) {
        // TODO
    }

}

impl fmt::Write for VgaWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

lazy_static! {
    pub static ref WRITER: Mutex<VgaWriter> = Mutex::new(VgaWriter {
        row_position: 0,
        column_position: 0,
        vga: Graphics640x480x16::new()
    });
}

pub fn init() {
    let writer = WRITER.lock();
    writer.vga.set_mode();
    writer.vga.clear_screen(Color16::Black);
}

#[macro_export]
macro_rules! kprint {
    ($($arg:tt)*) => ($crate::drivers::vga::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! kprintln {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::kprint!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    // to prevent deadlocks
    interrupts::without_interrupts(|| {
        WRITER.lock().write_fmt(args).unwrap();
    });
}

use super::tty::TTY;

pub struct VgaTty {}

impl TTY for VgaTty {
    fn clear_lines(&mut self, count: usize) {
        let mut writer = WRITER.lock();
        let row = writer.row_position;
        for i in 0..count {
            if row + i < WINDOW_HEIGHT {
                writer.clear_row(row + i);
            }
        }
    }
    fn get_pos(&self) -> (usize, usize) {
        let writer = WRITER.lock();
        (writer.row_position, writer.column_position)
    }
    fn set_pos(&mut self, row: usize, col: usize) {
        let mut writer = WRITER.lock();
        writer.row_position = row.min(WINDOW_HEIGHT - 1);
        writer.column_position = col.min(WINDOW_WIDTH - 1);
    }
    fn width(&self) -> usize {
        640
    }
    fn height(&self) -> usize {
        480
    }

    fn putchar(&mut self, c: u8) {
        let mut writer = WRITER.lock();
        writer.write_byte(c);
    }
}

// TESTS
#[test_case]
fn test_println_simple() {
    kprintln!("test_println_simple output");
}

#[test_case]
fn test_println_many() {
    for _ in 0..200 {
        kprintln!("test_println_many output");
    }
}

