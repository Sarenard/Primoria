use pc_keyboard::{
    layouts, DecodedKey, Error, HandleControl, KeyEvent, Keyboard, KeyboardLayout, ScancodeSet,
    ScancodeSet1,
};
use spin::{Mutex, MutexGuard};

use crate::kprint;

#[derive(Clone, Copy)]
pub enum Keymap {
    Azerty,
    Qwerty,
}

static mut AZERTY_KEYBOARD: Option<Keyboard<layouts::Azerty, ScancodeSet1>> = None;
static mut QWERTY_KEYBOARD: Option<Keyboard<layouts::Uk105Key, ScancodeSet1>> = None;

static LAYOUT: Mutex<Keymap> = Mutex::new(Keymap::Azerty);

fn current_keyboard<'a>(layout: &'a MutexGuard<Keymap>) -> &'a mut dyn KeyboardImpl {
    unsafe {
        match **layout {
            Keymap::Azerty => match AZERTY_KEYBOARD {
                Some(ref mut keyboard) => keyboard,
                None => {
                    AZERTY_KEYBOARD = Some(Keyboard::new(
                        layouts::Azerty,
                        ScancodeSet1,
                        HandleControl::Ignore,
                    ));
                    AZERTY_KEYBOARD.as_mut().unwrap()
                }
            },
            Keymap::Qwerty => match QWERTY_KEYBOARD {
                Some(ref mut keyboard) => keyboard,
                None => {
                    QWERTY_KEYBOARD = Some(Keyboard::new(
                        layouts::Uk105Key,
                        ScancodeSet1,
                        HandleControl::Ignore,
                    ));
                    QWERTY_KEYBOARD.as_mut().unwrap()
                }
            },
        }
    }
}

// for testing
use crate::system::kshell::KSHELL;

pub fn handle_scancode(scancode: u8) {
    let layout = LAYOUT.lock();
    let keyboard = current_keyboard(&layout);
    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            // release the mutex here because keypressed handlers may try to aquire it
            drop(layout);
            match key {
                DecodedKey::Unicode(character) => {
                    // just for testing
                    KSHELL.lock().keypressed(character)
                }
                DecodedKey::RawKey(key) => {
                    // just for testing
                    KSHELL.lock().keypressed_raw(key)
                }
            }
        }
    }
}

trait KeyboardImpl {
    fn clear(&mut self);
    fn add_word(&mut self, word: u16) -> Result<Option<KeyEvent>, Error>;
    fn add_byte(&mut self, byte: u8) -> Result<Option<KeyEvent>, Error>;
    fn process_keyevent(&mut self, ev: KeyEvent) -> Option<DecodedKey>;
}
impl<T, S> KeyboardImpl for Keyboard<T, S>
where
    T: KeyboardLayout,
    S: ScancodeSet,
{
    fn clear(&mut self) {
        Keyboard::clear(self)
    }
    fn add_byte(&mut self, byte: u8) -> Result<Option<KeyEvent>, Error> {
        Keyboard::add_byte(self, byte)
    }
    fn add_word(&mut self, word: u16) -> Result<Option<KeyEvent>, Error> {
        Keyboard::add_word(self, word)
    }
    fn process_keyevent(&mut self, ev: KeyEvent) -> Option<DecodedKey> {
        Keyboard::process_keyevent(self, ev)
    }
}

pub fn set_keymap(keymap: Keymap) {
    let mut layout = LAYOUT.lock();
    let keyboard = current_keyboard(&layout);
    keyboard.clear();
    *layout = keymap;
}
