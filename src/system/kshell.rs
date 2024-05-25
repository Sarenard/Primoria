use lazy_static::lazy_static;
use spin::Mutex;

use crate::drivers::tty::TTY;
use crate::drivers::vga::VgaTty;
use crate::kprintln;

pub struct KShell {
    start_row: usize, // row where the current line starts
    end_row: usize,   // row where the current line strated last time it was drawn
    buffer: [char; 2048],
    pos: usize,
    tty: VgaTty,
}

const PROMPT: &[u8] = b"> ";

lazy_static! {
    pub static ref KSHELL: Mutex<KShell> = Mutex::new(KShell {
        start_row: 0,
        end_row: 0,
        buffer: ['\0'; 2048],
        pos: 0,
        tty: VgaTty {},
    });
}

impl KShell {
    pub fn init(&mut self) {
        kprintln!("Welcome to Primoria!");
        (self.start_row, _) = self.tty.get_pos();
        self.end_row = self.start_row;
        self.draw_line();
    }
    pub fn keypressed(&mut self, key: char) {
        match key {
            '\n' => {
                self.tty.putchar(b'\n');
                // execute command
                self.exec();
                self.buffer.fill('\0');
                self.pos = 0;
                (self.start_row, _) = self.tty.get_pos();
            }
            '\t' => {}
            '\x08' => {
                // backspace
                if self.pos > 0 {
                    self.pos -= 1;
                }
                self.buffer[self.pos] = '\0';
            }
            _ => {
                if self.pos < self.buffer.len() {
                    self.buffer[self.pos] = key;
                    self.pos += 1;
                }
            }
        }
        self.draw_line();
    }

    fn draw_line(&mut self) {
        self.tty.set_pos(self.start_row, 0);
        if self.end_row >= self.start_row {
            self.tty.clear_lines(1 + self.end_row - self.start_row);
        }

        for &c in PROMPT {
            self.tty.putchar(c);
        }
        for c in self.buffer {
            if c == '\0' {
                break;
            }
            self.tty.putchar(c as u8);
        }
        (self.end_row, _) = self.tty.get_pos();
    }

    fn exec(&mut self) {
        let cmd_start = match self.next_non_white(0) {
            Some(i) => i,
            None => return,
        };
        let cmd_end = self.next_white(cmd_start);
        if self.streq(cmd_start, cmd_end, "quit") {
            kprintln!("EXIT!");
        }
    }

    fn next_white(&self, mut start: usize) -> usize {
        loop {
            if start >= self.buffer.len() || self.buffer[start] == '\0' {
                return start;
            }
            if self.buffer[start].is_whitespace() {
                return start;
            }
            start += 1;
        }
    }
    fn next_non_white(&self, mut start: usize) -> Option<usize> {
        loop {
            if start >= self.buffer.len() || self.buffer[start] == '\0' {
                return None;
            }
            if !self.buffer[start].is_whitespace() {
                return Some(start);
            }
            start += 1;
        }
    }
    fn streq(&self, start: usize, end: usize, string: &str) -> bool {
        if end - start != string.len() {
            return false;
        }
        for (n, c) in string.char_indices() {
            if c != self.buffer[start + n] {
                return false;
            }
        }
        true
    }
}
