use core::arch::asm;

// TODO : add all of the other ones
// TODO : pack this into a Port object
/*
Can help :
https://docs.rs/x86_64/0.14.2/src/x86_64/instructions/port.rs.html
https://github.com/elydre/profanOS/blob/main/kernel/cpu/ports.c
*/

pub unsafe fn port_byte_in(port: u16) -> u8 {
    let value: u8;
    unsafe {
        asm!("in al, dx", out("al") value, in("dx") port, options(nomem, nostack, preserves_flags));
    }
    value
}

pub unsafe fn port_byte_out(port: u16, value: u8) {
    asm!("out dx, al", in("dx") port, in("al") value, options(nomem, nostack, preserves_flags));
}

pub unsafe fn port_word_out(port: u16, value: u16) {
    asm!("out dx, ax", in("dx") port, in("ax") value, options(nomem, nostack, preserves_flags));
}

pub unsafe fn port_long_out(port: u16, value: u32) {
    asm!("out dx, eax", in("dx") port, in("eax") value, options(nomem, nostack, preserves_flags));
}
