use primoria::drivers::vga as vga_driver;
use vga::colors::Color16;

pub fn simple_counter() {
    let mut n: u64 = 0;
    let mut digits = [0u8; 16];
    let mut prev_digits = [0u8; 16];
    let digit_chars = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];
    loop {
        let mut digit_count = 0;
        let mut i = n;
        while i > 0 {
            digits[digit_count] = (i % 10) as u8;
            i /= 10;
            digit_count += 1;
        }
        for d in 0..digit_count {
            if digits[d] != prev_digits[d] {
                let col = 79 - d;
                vga_driver::draw_rect(col * 8, 24 * 8, 8, 8, Color16::Black);
                vga_driver::draw_char(digit_chars[digits[d] as usize], col * 8, 24 * 8, Color16::Green);
            }
        }
        prev_digits = digits;
        n += 1;
    }
}
