use core::fmt::{ self, Write };
use crate::drivers;

#[doc(hidden)]
pub fn _mu_print(args: fmt::Arguments) {
    unsafe {
        if !crate::globals::IS_MINI_UART_SETUP {
            panic!("Mini UART is expected to be initialized before calling `_print`");
        }
        let mini_uart = drivers::MiniUARTRegisters::get();
        mini_uart.write_fmt(args).unwrap_unchecked();
    }
}

#[macro_export]
macro_rules! mu_print {
    ($($tok:tt)*) => ({
        $crate::print::_mu_print(format_args!($($tok)*))
    });
}

#[macro_export]
macro_rules! mu_println {
    () => ({
        $crate::print::_mu_print("\n");
    });

    ($($tok:tt)*) => ({
        $crate::print::_mu_print(format_args_nl!($($tok)*));
    });
}
