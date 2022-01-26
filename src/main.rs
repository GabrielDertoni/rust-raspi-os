//! Kernel main
//!

#![no_main]
#![no_std]
#![feature(
    fmt_internals,
    extern_types,
    format_args_nl,
    never_type,
    maybe_uninit_uninit_array,
    coerce_unsized,
    dispatch_from_dyn,
    unsize,
    const_mut_refs,
    bench_black_box,
    decl_macro,
    inline_const
)]
#![allow(dead_code, unused_imports)]

mod allocators;
mod boot;
mod drivers;
mod error;
mod print;
mod utils;

use core::{panic::PanicInfo, sync::atomic::Ordering};

use drivers::{mu_is_setup, mu_print, mu_println, mu_recv, mu_send, MiniUART, GPIO};
use error::KError;
use utils::{get_cpu, get_current_exception_level};

unsafe fn kernel_init() -> ! {
    // This scope is necessary because the GPIO and Mini UART are beeing acquired and will be
    // release only once dropped, which happens at the end of the scope.
    {
        let mut gpio = GPIO::acquire();
        let mut mini_uart = MiniUART::acquire();
        mini_uart.init_default(&mut gpio);
    }

    match kernel_main() {
        Err(e) => panic!("{}", e),
        Ok(impossible) => impossible,
    }
}

fn kernel_main() -> Result<!, KError> {
    mu_println!("Initializing kernel...");
    mu_println!(
        "[INFO] initialized in exception level {}",
        get_current_exception_level()
    );
    mu_println!("[INFO] core {:x}", get_cpu());

    boot::CHILD_TASKS[1].store(hello_from_cpu as *mut (), Ordering::SeqCst);
    boot::CHILD_TASKS[2].store(hello_from_cpu as *mut (), Ordering::SeqCst);
    boot::CHILD_TASKS[3].store(hello_from_cpu as *mut (), Ordering::SeqCst);

    cortex_a::asm::sev();

    loop {
        let byte = mu_recv();
        match byte {
            b'\r' => mu_send(b'\n'),
            127 => mu_print!("\x08 \x08"),
            byte => mu_send(byte),
        }
    }
}

#[no_mangle]
fn hello_from_cpu() {
    mu_println!("Hello, from cpu {}", get_cpu());
}

#[inline(never)]
#[no_mangle]
fn marker() {
    cortex_a::asm::nop();
    cortex_a::asm::nop();
    cortex_a::asm::nop();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if mu_is_setup() {
        mu_println!("{}", info);
    }
    marker();
    utils::inifinite_loop();
}
