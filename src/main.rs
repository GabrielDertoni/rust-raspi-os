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
)]

mod utils;
mod boot;
mod drivers;
mod print;
mod globals;
mod allocators;
mod error;

use core::panic::PanicInfo;

use error::KError;
use tock_registers::interfaces::Readable;

unsafe fn kernel_init() -> ! {
    let gpio = drivers::GPIORegisters::get();
    let mini_uart = drivers::MiniUARTRegisters::get();
    mini_uart.init(gpio);
    globals::IS_MINI_UART_SETUP = true;

    match kernel_main() {
        Err(e) => panic!("{}", e),
        Ok(impossible) => impossible,
    }
}

unsafe fn kernel_main() -> Result<!, KError> {
    let mini_uart = drivers::MiniUARTRegisters::get();

    mu_println!("Initializing kernel...");
    mu_println!("[INFO] initialized in exception level {}", get_exception_level());

    loop {
        let byte = mini_uart.recv();
        match byte {
            b'\r' => mini_uart.send(b'\n'),
            127   => mu_print!("\x08 \x08"),
            byte  => mini_uart.send(byte),
        }
    }
}

fn get_exception_level() -> u64 {
    use cortex_a::registers::CurrentEL;
    let reg = CurrentEL;
    reg.read(CurrentEL::EL)
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    unsafe {
        if globals::IS_MINI_UART_SETUP {
            mu_println!("{}", info);
        }
    }
    utils::inifinite_loop();
}
