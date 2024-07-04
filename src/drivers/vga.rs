use core::fmt;

use vga::colors::Color16;
use vga::writers::{Graphics640x480x16, GraphicsWriter, PrimitiveDrawing};

use x86_64::instructions::interrupts::without_interrupts;

use crate::drivers::tty::GLOBAL_TTY;

const WINDOW_HEIGHT: usize = 480;
const WINDOW_WIDTH: usize = 640;

const VGA: Graphics640x480x16 = Graphics640x480x16;

pub fn init() {
    VGA.set_mode();
    VGA.clear_screen(Color16::Black);
}

pub fn draw_char(c: char, x: usize, y: usize, color: Color16) {
    without_interrupts(|| {
        VGA.draw_character(x, y, c, color);
    });
}
pub fn clear(color: Color16) {
    without_interrupts(|| {
        VGA.clear_screen(color);
    });
}
pub fn draw_rect(x: usize, y: usize, w: usize, h: usize, color: Color16) {
    without_interrupts(|| {
        for i in y..y + h {
            VGA.draw_line(
                (x as isize, i as isize),
                ((x + w) as isize, i as isize),
                color,
            );
        }
    });
}

pub fn height() -> usize {
    WINDOW_HEIGHT
}
pub fn width() -> usize {
    WINDOW_WIDTH
}

#[macro_export]
macro_rules! kprint {
    ($($arg:tt)*) => ($crate::drivers::vga::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! kprintln {
    () => ($crate::kprint!("\n"));
    ($($arg:tt)*) => ($crate::kprint!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    // to prevent deadlocks
    interrupts::without_interrupts(|| {
        GLOBAL_TTY.lock().write_fmt(args).unwrap();
    });
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
