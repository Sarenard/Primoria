use lazy_static::lazy_static;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};

use crate::kprintln;
use crate::system::gdt;

use crate::kernel;

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.divide_error.set_handler_fn(divide_error);
        idt.page_fault.set_handler_fn(page_fault);
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt[InterruptIndex::Timer.as_usize()].set_handler_fn(kernel::timer_interrupt_handler);
        idt[InterruptIndex::Keyboard.as_usize()].set_handler_fn(keyboard_interrupt_handler);
        idt[InterruptIndex::System.as_usize()].set_handler_fn(kernel::system_interrupt_handler);
        idt
    };
}

pub fn init() {
    IDT.load();
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    kprintln!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn divide_error(stack_frame: InterruptStackFrame) {
    kprintln!("EXCEPTION: DIVIDE BY 0\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn page_fault(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    kprintln!(
        "EXCEPTION: page_fault\n{:#?}\n{:#?}\n",
        stack_frame,
        error_code
    );
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) -> ! {
    panic!(
        "EXCEPTION: DOUBLE FAULT\n{:#?};\n{}",
        stack_frame, error_code
    );
}

#[test_case]
fn test_breakpoint_exception() {
    // invoke a breakpoint exception (needs to not crash)
    x86_64::instructions::interrupts::int3();
}

use pic8259::ChainedPics;
use spin;

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub static PICS: spin::Mutex<ChainedPics> =
    spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard,
    System = 0x80,
}

impl InterruptIndex {
    pub fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    use x86_64::instructions::port::Port;

    use crate::drivers::keyboard;

    let mut port = Port::new(0x60);

    let scancode: u8 = unsafe { port.read() };
    keyboard::handle_scancode(scancode);

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}

// extern "x86-interrupt" fn timer_interrupt_handler(mut stack_frame: InterruptStackFrame) {
//     implemented in kernel.rs
// }

// pub extern "x86-interrupt" fn system_interrupt_handler(stack_frame: InterruptStackFrame) {
//     implemented in kernel.rs
// }
