use primoria::drivers::vga as vga_driver;
use primoria::kernel::{thread_id, ticks};
use vga::colors::Color16;

pub fn simple_counter() {
    let (base_col, color, wait) = if primoria::kernel::fork() == 0 {
        (79, Color16::Green, false)
    } else {
        (69, Color16::Blue, true)
    };

    let mut n: u64 = 0;
    let mut digits = [0u8; 16];
    let mut prev_digits = [0u8; 16];
    let digit_chars = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];

    let mut prev = ticks();
    loop {
        let mut now = ticks();
        while wait && now - prev < 1 {
            now = ticks();
            core::hint::spin_loop();
        }
        prev = now;

        let mut digit_count = 0;
        let mut i = n;
        while i > 0 {
            digits[digit_count] = (i % 10) as u8;
            i /= 10;
            digit_count += 1;
        }
        for d in 0..digit_count {
            if digits[d] != prev_digits[d] {
                let col = base_col - d;
                vga_driver::draw_rect(col * 8, 24 * 8, 8, 8, Color16::Black);
                vga_driver::draw_char(digit_chars[digits[d] as usize], col * 8, 24 * 8, color);
            }
        }
        prev_digits = digits;
        n += 1;
    }
}

pub fn simple_loop() {
    let mut i: u64 = 0;
    loop {
        let cur = ticks();
        primoria::sprintln!("thread {} ({} x10 ticks)", thread_id(), i);
        while ticks() - cur < 10 {
            core::hint::spin_loop();
            i += 1;
        }
    }
}
