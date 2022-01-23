//! Kernel main
//!

#![no_main]
#![no_std]

#![feature(fmt_internals)]
#![feature(extern_types)]
#![feature(format_args_nl)]

mod utils;
mod boot;
mod drivers;
mod print;
mod globals;

use core::panic::PanicInfo;
use core::fmt::{ self, Write };

unsafe fn kernel_init() -> ! {
    let gpio = drivers::GPIORegisters::get();
    let mini_uart = drivers::MiniUARTRegisters::get();
    mini_uart.init(gpio);
    globals::IS_MINI_UART_SETUP = true;

    kernel_main().unwrap();
    unreachable!();
}

unsafe fn kernel_main() -> fmt::Result {
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
            let mini_uart = drivers::MiniUARTRegisters::get();
            writeln!(mini_uart, "{}", info).unwrap_unchecked();
        }
    }
    utils::inifinite_loop();
}
