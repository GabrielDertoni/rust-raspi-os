//! Kernel main
//!

#![no_main]
#![no_std]

#![feature(fmt_internals)]
#![feature(extern_types)]

#![allow(dead_code)]

mod utils;
mod boot;
mod drivers;

use core::panic::PanicInfo;

unsafe fn kernel_init() -> ! {

    let gpio = drivers::GPIORegisters::get();
    let mini_uart = drivers::MiniUARTRegisters::get();
    mini_uart.init(gpio);
    mini_uart.write("hello, world\n".as_bytes());
    loop {
        let byte = mini_uart.recv();
        mini_uart.send(byte);
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    utils::inifinite_loop();
}
