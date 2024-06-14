use alloc::vec::Vec;
use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;
use vga::colors::Color16;

use crate::drivers::vga as vga_driver;

#[derive(Debug)]
pub struct Tty {
    buffer: Vec<(Color16, char)>,
    width: usize,
    height: usize,

    /// current column
    pub col: usize,
    /// current row
    pub row: usize,
    /// current mode
    pub mode: TtyMode,
    /// current draw color
    pub color: Color16,
}

impl Tty {
    pub fn new(width: usize, height: usize) -> Self {
        let mut buffer = Vec::new();
        buffer.resize(width * height, (Color16::White, '\0'));
        let ret = Self {
            buffer,
            width,
            height,
            col: 0,
            row: 0,
            mode: TtyMode::Scrolling,
            color: Color16::White,
        };
        ret
    }
    pub fn width(&self) -> usize {
        self.width
    }
    pub fn height(&self) -> usize {
        self.height
    }

    pub fn putchar(&mut self, c: char) {
        match c {
            '\n' => {
                if self.mode == TtyMode::Scrolling {
                    self.scroll(1);
                }
            }
            c => {
                self.buffer[self.col + self.row * self.width] = (self.color, c);
                if self.mode == TtyMode::Scrolling {
                    self.col += 1;
                    if self.col == self.width {
                        self.scroll(1);
                    }
                }
            }
        }
    }

    /// scroll a certain amount down
    pub fn scroll(&mut self, amount: usize) {
        self.row += amount;
        self.col = 0;
        if self.row >= self.height {
            let offset = (self.row - self.height + 1).min(self.height);
            self.row = self.height - 1;

            for i in 0..self.height - offset {
                for j in 0..self.width {
                    self.buffer[j + i * self.width] = self.buffer[j + (i + offset) * self.width];
                }
            }
            for i in self.height - offset - 1..self.height {
                for j in 0..self.width {
                    self.buffer[j + i * self.width] = (self.color, '\0');
                }
            }
        }
    }

    pub fn clear_row(&mut self, row: usize) {
        for j in 0..self.width {
            self.buffer[j + row * self.width] = (Color16::White, '\0');
        }
    }

    pub fn render(&mut self) {
        use x86_64::instructions::interrupts;
        let below_cursor = self.buffer[self.col + self.row * self.width];
        self.buffer[self.col + self.row * self.width] = (self.color, '_');

        vga_driver::clear(Color16::Black);
        for i in 0..self.height {
            for j in 0..self.width {
                let (color, c) = self.buffer[j + i * self.width];
                if c == '\0' {
                    break;
                }
                vga_driver::draw_char(c, j, i, color);
            }
        }
        self.buffer[self.col + self.row * self.width] = below_cursor;
    }

    pub fn write_string(&mut self, s: &str) {
        for c in s.chars() {
            self.putchar(c);
        }
    }
}

impl fmt::Write for Tty {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TtyMode {
    Scrolling,
    Fixed,
}

lazy_static! {
    pub static ref GLOBAL_TTY: Mutex<Tty> = Mutex::new(Tty::new(80, 25));
}
