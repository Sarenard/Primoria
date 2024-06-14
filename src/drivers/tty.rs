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

    cursor_col: usize,
    cursor_row: usize,

    /// current column used by putchar
    pub col: usize,
    /// current row used by putchar
    pub row: usize,
    /// current mode
    pub mode: TtyMode,
    /// current draw color
    pub color: Color16,
}

// primitives for drawing

fn draw_char_at(c: char, row: usize, col: usize, color: Color16) {
    let x = col * 8;
    let y = row * 8;
    vga_driver::draw_rect(x, y, 8, 8, Color16::Black);
    vga_driver::draw_char(c, x, y, color);
}

fn draw_cursor(row: usize, col: usize, color: Color16) {
    let x = col * 8 + 1;
    let y = row * 8 + 6;
    vga_driver::draw_rect(x, y, 6, 2, color);
}

impl Tty {
    pub fn new(width: usize, height: usize) -> Self {
        let mut buffer = Vec::new();
        buffer.resize(width * height, (Color16::White, '\0'));
        let ret = Self {
            buffer,
            width,
            height,
            cursor_col: 0,
            cursor_row: 0,
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

                self.render_pos(self.row, self.col);

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
            for i in self.height - offset..self.height {
                for j in 0..self.width {
                    self.buffer[j + i * self.width] = (self.color, '\0');
                }
            }
            self.render_all();
        }
    }

    pub fn clear_rect(&mut self, start_row: usize, start_col: usize, width: usize, height: usize) {
        for i in start_row..start_row + height {
            for j in start_col..start_col + width {
                self.buffer[j + i * self.width] = (self.color, '\0');
            }
        }
        let x = start_col * 8;
        let y = start_row * 8;
        vga_driver::draw_rect(x, y, width * 8, height * 8, Color16::Black);
        if self.cursor_row >= start_row
            && self.cursor_col >= start_col
            && self.cursor_row < start_row + height
            && self.cursor_col < start_col + width
        {
            self.render_cursor();
        }
    }

    pub fn get_cursor(&self) -> (usize, usize) {
        (self.cursor_row, self.cursor_col)
    }
    pub fn set_cursor(&mut self, row: usize, col: usize) {
        self.render_pos(self.cursor_row, self.cursor_col);
        self.cursor_row = row;
        self.cursor_col = col;
        self.render_cursor();
    }

    fn render_pos(&self, row: usize, col: usize) {
        let (color, c) = self.buffer[col + row * self.width];
        draw_char_at(c, row, col, color);
    }

    fn render_all(&self) {
        vga_driver::clear(Color16::Black);
        for i in 0..self.height {
            for j in 0..self.width {
                self.render_pos(i, j);
            }
        }
        draw_cursor(self.cursor_row, self.cursor_col, self.color);
    }

    fn render_cursor(&self) {
        draw_cursor(self.cursor_row, self.cursor_col, self.color);
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
