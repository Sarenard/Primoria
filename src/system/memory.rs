use alloc::alloc::{GlobalAlloc, Layout};
use core::sync::atomic::{AtomicUsize, Ordering};

const KB: usize = 1024;
const MEM_SIZE: usize = 1024 * KB;
static mut MEMORY: [u8; MEM_SIZE] = [0; MEM_SIZE];

pub struct Allocator {
    index: AtomicUsize,
}

impl Allocator {
    const fn new() -> Allocator {
        Allocator {
            index: AtomicUsize::new(0),
        }
    }
}

unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let size = layout.size();
        let align = layout.align();
        
        let current_index = self.index.load(Ordering::SeqCst);
        
        // Align the current index
        let aligned_index = (current_index + align - 1) & !(align - 1);
        
        // Check if we have enough space
        if aligned_index + size > MEM_SIZE {
            return core::ptr::null_mut();
        }

        // Update the index atomically
        self.index.store(aligned_index + size, Ordering::SeqCst);

        &mut MEMORY[aligned_index] as *mut u8
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // TODO
    }
}

#[global_allocator]
static ALLOCATOR: Allocator = Allocator::new();