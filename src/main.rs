//! Kernel main
//!

#![no_main]
#![no_std]

#![feature(fmt_internals, extern_types, format_args_nl, never_type)]

mod utils;
mod boot;
mod drivers;
mod print;
mod globals;

use core::panic::PanicInfo;

unsafe fn kernel_init() -> ! {
    let gpio = drivers::GPIORegisters::get();
    let mini_uart = drivers::MiniUARTRegisters::get();
    mini_uart.init(gpio);
    globals::IS_MINI_UART_SETUP = true;

    kernel_main().unwrap();
}

unsafe fn kernel_main() -> Result<!, ()> {
    let mini_uart = drivers::MiniUARTRegisters::get();

    mu_println!("Initializing kernel...");

    loop {
        let byte = mini_uart.recv();
        match byte {
            b'\r' => mini_uart.send(b'\n'),
            127   => mu_print!("\x08 \x08"),
            byte  => mini_uart.send(byte),
        }
    }
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
