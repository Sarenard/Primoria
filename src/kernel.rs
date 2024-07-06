use alloc::boxed::Box;
use core::mem::size_of;
use x86_64::instructions::interrupts::without_interrupts;
use x86_64::structures::idt::InterruptStackFrame;

use crate::system::idt::{InterruptIndex, PICS};

const STACK_SIZE: usize = 1024; // number of usize in the stack

// global kernel state
static mut STATE: State = State::DEFAULT;

pub fn init() {
    unsafe {
        STATE.thread_count = 1;
    }
}

/// this function must be called exactly once
pub unsafe fn start(main: fn()) -> ! {
    let stack_pointer: usize;
    core::arch::asm!(
        "mov {sp}, rsp",
        sp = out(reg) stack_pointer,
    );
    STATE.threads[0].stack_end = stack_pointer;

    main();

    panic!("Thread 0 finished, nothing more to do");
}

pub fn launch(thread: fn()) -> usize {
    let id: usize;
    unsafe {
        core::arch::asm!(
            "int 0x80",
            in("rax") Syscall::LaunchThread as u64,
            in("rdi") thread,
            lateout("rax") id,
        );
    }
    return id;
}

pub fn thread_id() -> usize {
    unsafe { STATE.current_thread }
}

pub fn ticks() -> usize {
    unsafe { STATE.ticks }
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
    ticks: usize,
}
impl State {
    const DEFAULT: Self = Self {
        threads: [Thread::DEFAULT; 8],
        thread_count: 0,
        current_thread: 0,
        ticks: 0,
    };
}

struct Thread {
    stack_frame: StackFrame,
    cpu_regs: CpuRegs,
    stack_end: usize, // address past the end of the stack
}
impl Thread {
    const DEFAULT: Self = Self {
        stack_frame: StackFrame::DEFAULT,
        cpu_regs: CpuRegs::DEFAULT,
        stack_end: 0,
    };
}

//
// kernel stuff
//

/// safety: must be called in a critical section
///
pub unsafe fn switch_stack_frame(stack_frame: &mut StackFrame) {
    let cur = STATE.current_thread;
    // round robin
    let next = (cur + 1) % STATE.thread_count;

    if cur != next {
        let cur_thread = &mut STATE.threads[cur];
        let next_thread = &mut STATE.threads[next];

        cur_thread.stack_frame = *stack_frame;
        *stack_frame = next_thread.stack_frame;
    }

    STATE.current_thread = next;
}

#[no_mangle]
unsafe extern "sysv64" fn get_current_regs(dest: *mut CpuRegs) {
    *dest = STATE.threads[STATE.current_thread].cpu_regs;
}

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

//
// interrupts
//

#[repr(usize)]
#[derive(Debug, Clone, Copy)]
enum Syscall {
    LaunchThread = 0xaa,
}

#[no_mangle]
unsafe extern "sysv64" fn _save_regs_to_current(regs: *const CpuRegs) {
    STATE.threads[STATE.current_thread].cpu_regs = *regs;
}
macro_rules! save_regs_to_current {
    () => {
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
            "call _save_regs_to_current",
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
    };
}

pub extern "x86-interrupt" fn timer_interrupt_handler(mut stack_frame: InterruptStackFrame) {
    let stack_frame_addr = &mut stack_frame as *mut _ as usize;
    let stack_frame_ptr = stack_frame_addr as *mut StackFrame;
    unsafe {
        save_regs_to_current!();

        STATE.ticks += 1;

        switch_stack_frame(&mut *stack_frame_ptr);

        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());

        back_to_thread(stack_frame_ptr);
    }
}

#[naked]
pub extern "x86-interrupt" fn system_interrupt_handler(stack_frame: InterruptStackFrame) {
    unsafe {
        core::arch::asm!(
            "push r11",
            "push r10",
            "push r9",
            "push r8",
            "push rsi",
            "push rdi",
            "push rdx",
            "push rcx",
            "mov rsi, rdi",           // arg 2
            "mov rdi, rax",           // arg 1
            "lea rdx, [rsp + 8 * 8]", // stack_frame address
            "call syscall_impl",
            "pop rcx",
            "pop rdx",
            "pop rdi",
            "pop rsi",
            "pop r8",
            "pop r9",
            "pop r10",
            "pop r11",
            "iretq",
            options(noreturn),
        );
    }
}

#[no_mangle]
extern "sysv64" fn _thread_start(thread: extern "sysv64" fn()) -> ! {
    thread();
    unimplemented!("don't know what to do when a thread finished");
}

#[no_mangle]
extern "sysv64" fn syscall_impl(id: u64, arg2: u64, stack_frame: *const StackFrame) -> usize {
    unsafe {
        if id == Syscall::LaunchThread as u64 {
            let mut child_id = 0;
            without_interrupts(|| {
                if STATE.thread_count >= STATE.threads.len() {
                    panic!("too many threads");
                }

                // TODO: allocate the stack in a better place
                let new_stack = Box::leak(Box::new([0usize; STACK_SIZE])) as *mut _ as usize;
                let new_stack_addr = new_stack + STACK_SIZE * size_of::<usize>();
                crate::sprintln!("new stack pointer: {:x}", new_stack_addr);

                child_id = STATE.thread_count;

                // stack_end
                STATE.threads[child_id].stack_end = new_stack_addr;

                // stack_frame
                STATE.threads[child_id].stack_frame.instruction_pointer = _thread_start as u64;
                STATE.threads[child_id].stack_frame.code_segment = (*stack_frame).code_segment;
                // clear: CF, PF, AF, ZF, SF, TF, DF, OF,
                STATE.threads[child_id].stack_frame.cpu_flags = (*stack_frame).cpu_flags
                    & !(
                        // CF
                        0x0001
                        // PF
                        | 0x0004
                        // AF
                        | 0x0010
                        // ZF
                        | 0x0040
                        // SF
                        | 0x0080
                        // DF
                        | 0x0400
                        // OF
                        | 0x0800
                    );
                STATE.threads[child_id].stack_frame.stack_pointer = new_stack_addr as u64;
                STATE.threads[child_id].stack_frame.stack_segment = (*stack_frame).stack_segment;

                // cpu_regs
                // address to be executed by _thread_start
                STATE.threads[child_id].cpu_regs.rdi = arg2;

                STATE.thread_count += 1;
                crate::sprintln!("tcount: {}", STATE.thread_count);
            });
            return child_id;
        }
    }
    return 0;
}
