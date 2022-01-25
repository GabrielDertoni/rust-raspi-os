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
)]

mod utils;
mod boot;
mod drivers;
mod print;
mod allocators;
mod error;

use core::panic::PanicInfo;
use tock_registers::interfaces::Readable;

use error::KError;

use drivers::{GPIO, MiniUART, mu_recv, mu_send, mu_is_setup};

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
    mu_println!("[INFO] initialized in exception level {}", get_current_exception_level());
    mu_println!("[INFO] core {:x}", cortex_a::registers::MPIDR_EL1.get() & 0xff);

    loop {
        let byte = mu_recv();
        match byte {
            b'\r' => mu_send(b'\n'),
            127   => mu_print!("\x08 \x08"),
            byte  => mu_send(byte),
        }
    }
}

/// Gets the current exception level
fn get_current_exception_level() -> u64 {
    use cortex_a::registers::CurrentEL;
    let reg = CurrentEL;
    reg.read(CurrentEL::EL)
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if mu_is_setup() {
        mu_println!("{}", info);
    }
    utils::inifinite_loop();
}
