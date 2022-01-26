use core::{
    ptr,
    sync::atomic::{AtomicPtr, Ordering},
};

use crate::utils::get_cpu;

#[no_mangle]
#[link_section = ".text._start_arguments"]
pub static BOOT_CORE_ID: u64 = 0;

core::arch::global_asm!(include_str!("boot/boot.S"));

extern "C" {
    /// Some docs
    #[link_name = "_child_target"]
    #[allow(improper_ctypes)]
    static mut CHILD_TARGET: unsafe fn();
}

pub const CHILD_STACK_SIZE: usize = 64 * 1024;
#[no_mangle]
pub static CHILD_STACK_SIZE_E: usize = CHILD_STACK_SIZE;

pub static CHILD_TASKS: [AtomicPtr<()>; 4] = [const { AtomicPtr::new(ptr::null_mut()) }; 4];
#[no_mangle]
pub static CHILD_STACKS: [[u8; CHILD_STACK_SIZE]; 4] = [[0; CHILD_STACK_SIZE]; 4];

/// Entry point of the Rust language in the kernel. This function is called from assembly.
#[no_mangle]
pub unsafe fn _start_rust() -> ! {
    CHILD_TARGET = child_loop;
    crate::kernel_init();
}

#[no_mangle]
pub unsafe fn child_loop() {
    let cpu = get_cpu();
    loop {
        // NOTE: If I don't use read_volatile here, for some reason, rust assumes that no other
        // thread can write and change `CHILD_TASKS` so it optimizes it away. This is very weird
        // and unexpected behaviour!
        // FIXME: There may be a better memory ordering for this.
        let ptr = CHILD_TASKS[cpu as usize].load(Ordering::SeqCst);
        if !ptr.is_null() {
            CHILD_TASKS[cpu as usize].store(ptr::null_mut(), Ordering::SeqCst);
            (core::mem::transmute::<*mut (), fn()>(ptr))();
        }
        cortex_a::asm::wfe();
    }
}
