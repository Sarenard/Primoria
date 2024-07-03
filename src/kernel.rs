use alloc::boxed::Box;
use core::mem::{size_of, swap};
use x86_64::instructions::interrupts::without_interrupts;
use x86_64::structures::idt::InterruptStackFrame;

use crate::system::idt::{InterruptIndex, PICS};

mod storage;

const STACK_SIZE: usize = 1024; // number of usize in the stack

// global kernel state
static mut STATE: State = State::DEFAULT;

pub fn init() {
    unsafe {
        STATE.thread_count = 1;
    }
}


#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct CpuRegs {
    pub rax: u64,
    pub rbx: u64,
    pub rcx: u64,
    pub rdx: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub rbp: u64,
    pub r8: u64,
    pub r9: u64,
    pub r10: u64,
    pub r11: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
}
impl CpuRegs {
    const DEFAULT: Self = Self {
        rax: 0,
        rbx: 0,
        rcx: 0,
        rdx: 0,
        rsi: 0,
        rdi: 0,
        rbp: 0,
        r8: 0,
        r9: 0,
        r10: 0,
        r11: 0,
        r12: 0,
        r13: 0,
        r14: 0,
        r15: 0,
    };
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct StackFrame {
    pub instruction_pointer: u64,
    pub code_segment: u64,
    pub cpu_flags: u64,
    pub stack_pointer: u64,
    pub stack_segment: u64,
}
impl StackFrame {
    const DEFAULT: Self = Self {
        instruction_pointer: 0,
        code_segment: 0,
        cpu_flags: 0,
        stack_pointer: 0,
        stack_segment: 0,
    };
}

/// only valid on single-CPU
struct State {
    threads: [Thread; 8],
    thread_count: usize,
    // threads[current_thread] means nothing,
    // except between a call to `switch_stack_frame` and `back_to_thread`,
    // where the cpu_regs field contains the registers of the new thread
    current_thread: usize,
}
impl State {
    const DEFAULT: Self = Self {
        threads: [Thread::DEFAULT; 8],
        thread_count: 0,
        current_thread: 0,
    };
}

struct Thread {
    stack_frame: StackFrame,
    cpu_regs: CpuRegs,
}
impl Thread {
    const DEFAULT: Self = Self {
        stack_frame: StackFrame::DEFAULT,
        cpu_regs: CpuRegs::DEFAULT,
    };
}

//
// kernel stuff
//

/// safety: must be called in a critical section
///
pub unsafe fn switch_stack_frame(stack_frame: &mut StackFrame) {
    crate::sprintln!("switch, tcount = {}", unsafe { STATE.thread_count });
    let cur = STATE.current_thread;
    // round robin
    let next = (cur + 1) % STATE.thread_count;
    let next = 0;

    // stack frame
    if cur != next {
        let cur_thread = &mut STATE.threads[cur];
        let next_thread = &mut STATE.threads[next];

        crate::sprintln!("switching to thread {}", next);

        // save the active stack frame
        swap(stack_frame, &mut cur_thread.stack_frame);
        // replace the active stack frame with the new one
        swap(stack_frame, &mut next_thread.stack_frame);

        // set the new registers
        swap(&mut cur_thread.cpu_regs, &mut next_thread.cpu_regs);
    }

    STATE.current_thread = next;
    crate::sprintln!("switched");
}

#[no_mangle]
unsafe extern "sysv64" fn get_current_regs(dest: *mut CpuRegs) {
    *dest = STATE.threads[STATE.current_thread].cpu_regs;
}

#[inline(never)]
pub unsafe fn back_to_thread(stack_frame: *mut StackFrame) -> ! {
    core::arch::asm!(
        "mov rsp, {sp}",

        "sub rsp, 15 * 8",
        "mov rdi, rsp",
        "call get_current_regs",

        "pop rax",
        "pop rbx",
        "pop rcx",
        "pop rdx",
        "pop rsi",
        "pop rdi",
        "pop rbp",
        "pop r8",
        "pop r9",
        "pop r10",
        "pop r11",
        "pop r12",
        "pop r13",
        "pop r14",
        "pop r15",

        "iretq",
        sp = in(reg) stack_frame,
        options(noreturn),
    );
}

#[inline(never)]
pub fn fork() -> usize {
    let ret_id: usize;
    unsafe {
        core::arch::asm!(
            "mov rax, 0xaa", // fork
            "int 0x80",
            out("rax") ret_id,
        );
    }
    return ret_id;
}
//
// interrupts
//

#[no_mangle]
unsafe extern "sysv64" fn save_regs_to_current(regs: *const CpuRegs) {
    STATE.threads[STATE.current_thread].cpu_regs = *regs;
}

pub extern "x86-interrupt" fn timer_interrupt_handler(mut stack_frame: InterruptStackFrame) {
    let stack_frame_addr = &mut stack_frame as *mut _ as usize;
    let stack_frame_ptr = stack_frame_addr as *mut StackFrame;
    unsafe {
        core::arch::asm!(
            "push r15",
            "push r14",
            "push r13",
            "push r12",
            "push r11",
            "push r10",
            "push r9",
            "push r8",
            "push rbp",
            "push rdi",
            "push rsi",
            "push rdx",
            "push rcx",
            "push rbx",
            "push rax",

            "mov rdi, rsp",
            "call save_regs_to_current",

            "pop rax",
            "pop rbx",
            "pop rcx",
            "pop rdx",
            "pop rsi",
            "pop rdi",
            "pop rbp",
            "pop r8",
            "pop r9",
            "pop r10",
            "pop r11",
            "pop r12",
            "pop r13",
            "pop r14",
            "pop r15",
        );

        crate::sprintln!("saved regs");
        switch_stack_frame(&mut *stack_frame_ptr);

        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());

        back_to_thread(stack_frame_ptr);
    }
}
pub extern "x86-interrupt" fn system_interrupt_handler(stack_frame: InterruptStackFrame) {
    let id: usize;
    unsafe {
        core::arch::asm!(
            "",
            out("rax") id,
        )
    }
    crate::sprintln!("System");

    let stack_frame_addr = &stack_frame as *const _ as usize;
    let stack_frame_ptr = stack_frame_addr as *mut StackFrame;
    unsafe {
        // TODO: syscall numbers

        // fork
        if id == 0xaa {
            without_interrupts(|| {
                if STATE.thread_count >= STATE.threads.len() {
                    panic!("too many threads");
                }

                let new_stack = Box::leak(Box::new([0usize; STACK_SIZE])) as *mut _ as usize;
                let mut new_stack_pointer = new_stack + STACK_SIZE * size_of::<usize>();
                crate::sprintln!(
                    "allocated new stack: [{:x}..{:x}]",
                    new_stack,
                    new_stack_pointer
                );

                // last value on the stack is 0, may help to crash the thing in case it reaches this point
                new_stack_pointer -= size_of::<usize>();
                *(new_stack_pointer as *mut u64) = 0x57;

                let child_id = STATE.thread_count;

                // TODO: %rax = child_id; // parent thread

                STATE.threads[child_id].stack_frame = *stack_frame_ptr;
                STATE.threads[child_id].stack_frame.stack_pointer = new_stack_pointer as u64;

                // TODO: STATE.threads[child_id].cpu_regs = *cpu_regs_ptr;
                // TODO: STATE.threads[child_id].cpu_regs.rax = 0; // child thread

                crate::sprintln!("before: {}", STATE.thread_count);
                STATE.thread_count += 1;
                crate::sprintln!("after: {}", STATE.thread_count);
            });
        }
    }
}
