use crate::drivers::tty::TTY;

pub struct KShell<'tty> {
    buffer: [u8; 2048],
    pos: usize,
    tty: &'tty dyn TTY,
}

const PROMPT: &[u8] = b"> ";

impl<'tty> KShell<'tty> {
    pub fn keypressed(&mut self, key: u8) {
        match key {
            b'\n' => {
                // execute command

                self.buffer.fill(0);
                self.pos = 0;
            }
            9 => { // TAB

            }
            _ => {
                if self.pos < self.buffer.len() {
                    self.buffer[self.pos] = key;
                    self.pos += 1;
                }
            }
        }
    }

    fn exec(&mut self) {

    }
}
