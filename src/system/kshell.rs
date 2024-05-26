use lazy_static::lazy_static;
use pc_keyboard::KeyCode;
use spin::Mutex;

use crate::drivers::keyboard::{set_keymap, Keymap};
use crate::drivers::tty::TTY;
use crate::drivers::vga::VgaTty;
use crate::kprintln;

pub struct KShell {
    start_row: usize, // row where the current line starts
    end_row: usize,   // row where the current line strated last time it was drawn
    buffer: [char; 2048],
    buf_len: usize,
    pos: usize,
    tty: VgaTty,
}

const PROMPT: &[u8] = b"> ";

lazy_static! {
    pub static ref KSHELL: Mutex<KShell> = Mutex::new(KShell {
        start_row: 0,
        end_row: 0,
        buffer: ['\0'; 2048],
        buf_len: 0,
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
                self.buf_len = 0;
                (self.start_row, _) = self.tty.get_pos();
            }
            '\t' => {
                if self.pos > 0 {
                    let word_start = self.prev_word_start(self.pos - 1);
                    for (cmd_name, _) in Self::BUILTINS {
                        if self.starts_with(word_start, self.pos, cmd_name) {
                            self.ins_str(&cmd_name[self.pos - word_start..]);
                            self.ins_str(" ");
                            break;
                        }
                    }
                }
            }
            '\x08' => {
                // backspace
                if self.pos > 0 {
                    self.pos -= 1;
                    self.del_char();
                }
            }
            _ => {
                self.ins_char(key);
            }
        }
        self.draw_line();
    }

    pub fn keypressed_raw(&mut self, key: KeyCode) {
        match key {
            KeyCode::ArrowLeft => {
                if self.pos > 0 {
                    self.pos -= 1;
                }
            }
            KeyCode::ArrowRight => {
                if self.pos < self.buf_len {
                    self.pos += 1;
                }
            }
            KeyCode::Home => self.pos = 0,
            KeyCode::End => self.pos = self.buf_len,
            _ => {}
        }
        self.draw_line();
    }

    // returns whether the character was inserted
    fn ins_char(&mut self, c: char) -> bool {
        if self.buf_len >= self.buffer.len() {
            return false;
        }
        for i in (0..(self.buf_len - self.pos)).rev() {
            self.buffer[self.pos + i + 1] = self.buffer[self.pos + i]
        }
        self.buffer[self.pos] = c;
        self.pos += 1;
        self.buf_len += 1;
        true
    }
    fn del_char(&mut self) {
        for i in 0..(self.buf_len - self.pos) {
            self.buffer[self.pos + i] = self.buffer[self.pos + i + 1];
        }
        self.buffer[self.buf_len] = '\0';
        self.buf_len -= 1;
    }
    fn ins_str(&mut self, string: &str) -> bool {
        for c in string.chars() {
            if !self.ins_char(c) {
                return false;
            }
        }
        true
    }

    fn draw_line(&mut self) {
        self.tty.set_pos(self.start_row, 0);
        if self.end_row >= self.start_row {
            self.tty.clear_lines(1 + self.end_row - self.start_row);
        }

        for &c in PROMPT {
            self.tty.putchar(c);
        }
        let mut cursor_pos = None;
        for (n, c) in self.buffer[..self.buf_len].iter().enumerate() {
            if n == self.pos {
                cursor_pos = Some(self.tty.get_pos());
            }
            self.tty.putchar(*c as u8);
        }
        let cursor_pos = match cursor_pos {
            Some(pos) => pos,
            None => self.tty.get_pos(),
        };
        (self.end_row, _) = self.tty.get_pos();
        self.tty.set_pos(cursor_pos.0, cursor_pos.1);
    }
}

impl KShell {
    /// Each function takes the current KShell
    /// and the position of the first character after the command name
    const BUILTINS: [(&'static str, fn(&KShell, usize)); 3] = [
        ("keymap", Self::cmd_keymap),
        ("help", Self::cmd_help),
        ("quit", Self::cmd_quit),
    ];

    fn exec(&mut self) {
        let cmd_start = match self.next_non_white(0) {
            Some(i) => i,
            None => return,
        };
        let cmd_end = self.next_white(cmd_start);
        for (cmd_str, cmd_func) in Self::BUILTINS {
            if self.streq(cmd_start, cmd_end, cmd_str) {
                cmd_func(self, cmd_end);
                return;
            }
        }
        kprintln!("Command not found");
    }

    fn cmd_quit(&self, _: usize) {
        kprintln!("EXIT! (well, no)");
    }

    fn cmd_keymap(&self, cmd_end: usize) {
        let keymap_start = match self.next_non_white(cmd_end) {
            Some(i) => i,
            None => {
                kprintln!("No keymap specified");
                return;
            }
        };
        let keymap_end = self.next_white(keymap_start);
        if self.streq(keymap_start, keymap_end, "azerty") {
            set_keymap(Keymap::Azerty);
        } else if self.streq(keymap_start, keymap_end, "qwerty") {
            set_keymap(Keymap::Qwerty);
        } else {
            kprintln!("Unknown keymap");
        }
    }

    fn cmd_help(&self, _: usize) {
        kprintln!("Primoria KShell");
        kprintln!("Commands:");
        for (cmd, _) in Self::BUILTINS {
            kprintln!("  {}", cmd);
        }
    }

    fn next_white(&self, mut start: usize) -> usize {
        loop {
            if start >= self.buf_len {
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
            if start >= self.buf_len {
                return None;
            }
            if !self.buffer[start].is_whitespace() {
                return Some(start);
            }
            start += 1;
        }
    }
    fn prev_word_start(&self, mut end: usize) -> usize {
        if !self.buffer[end].is_alphanumeric() {
            return end;
        }
        while end > 0 && self.buffer[end - 1].is_alphanumeric() {
            end -= 1;
        }
        end
    }
    fn streq(&self, start: usize, end: usize, string: &str) -> bool {
        end - start == string.len() && self.starts_with(start, end, string)
    }
    /// tests whether the given string starts with buffer[start..end]
    fn starts_with(&self, start: usize, end: usize, string: &str) -> bool {
        for (n, c) in string.char_indices() {
            if n >= end - start {
                break;
            }
            if c != self.buffer[start + n] {
                return false;
            }
        }
        true
    }
}
