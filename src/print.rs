use crate::drivers;

#[doc(hidden)]
pub fn _print(args: core::fmt::Arguments) {
    drivers::mini_uart::_mu_print(args);
}

/// Print macro, syntax is like [`std::print`]. Current implementation prints to the Mini UART.
#[macro_export]
macro_rules! print {
    ($($tok:tt)*) => ({
        $crate::print::_print(format_args!($($tok)*))
    });
}

/// Println macro, syntax is like [`std::println`]. Current implementation prints to the Mini UART.
#[macro_export]
macro_rules! println {
    () => ({
        $crate::print::_print("\n");
    });

    ($($tok:tt)*) => ({
        $crate::print::_print(format_args_nl!($($tok)*));
    });
}
