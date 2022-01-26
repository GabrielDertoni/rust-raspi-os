use core::{
    fmt::{self, Write},
    sync::atomic::{AtomicBool, AtomicPtr, Ordering},
};

use super::{
    gpio::{GPIOFunc, GPIO},
    Reg32, MMIO_BASE_ADDR,
};

#[repr(C)]
struct MiniUARTRegisters {
    // Unclear if this should be in this struct.
    irq_status: Reg32,
    enables: Reg32,
    _reserved: [Reg32; 14],
    io: Reg32,
    ier: Reg32,
    iir: Reg32,
    lcr: Reg32,
    mcr: Reg32,
    lsr: Reg32,
    msr: Reg32,
    scratch: Reg32,
    control: Reg32,
    status: Reg32,
    baud_rate: Reg32,
}

impl MiniUARTRegisters {
    const REGS_ADDR: usize = MMIO_BASE_ADDR + 0x215000;
    #[inline(always)]
    pub const fn get() -> *mut Self {
        Self::REGS_ADDR as *mut Self
    }
}

static LOCK: spin::Mutex<Option<&'static mut MiniUARTRegisters>> = spin::Mutex::new(None);

/// Structure that represents an exclusive handle to the Mini UART.
pub struct MiniUART {
    guard: spin::MutexGuard<'static, Option<&'static mut MiniUARTRegisters>>,
}

impl MiniUART {
    const TX_PIN: u8 = 14;
    const RX_PIN: u8 = 15;

    /// Acquires exclusively the Mini UART.
    ///
    /// This function will block until a handle can be give to the caller. If the same thread is
    /// already using the handle or the other thread depends on some result from this thread, **it
    /// will deadlock**.
    pub fn acquire() -> Self {
        MiniUART { guard: LOCK.lock() }
    }

    /// Checks whether the Mini UART is setup.
    pub fn is_setup() -> bool {
        MiniUART::acquire().guard.is_some()
    }

    /// Initializes the Mini UART with the default baud rate of ~115200 @ 250 MHz
    pub fn init_default(&mut self, gpio: &mut GPIO) {
        self.init(gpio, 270);
    }

    /// Initializes the Mini UART. The baud rate divisor is used to calculate the baud rate of the
    /// UART which is based on the clock frequency. To calculate the baud rate the following
    /// formula can be used [according to the spec.](https://github.com/raspberrypi/documentation/files/1888662/BCM2837-ARM-Peripherals.-.Revised.-.V2-1.pdf)
    /// section 2.2.1:
    ///
    /// ```
    ///                  system_clock_freq
    /// baudrate = ---------------------------
    ///             8 * (baudrate_divisor + 1)
    /// ```
    pub fn init(&mut self, gpio: &mut GPIO, baud_divisor: u16) {
        gpio.set_pin_func(Self::TX_PIN, GPIOFunc::AltFn5);
        gpio.set_pin_func(Self::RX_PIN, GPIOFunc::AltFn5);

        gpio.pin_enable(Self::TX_PIN);
        gpio.pin_enable(Self::RX_PIN);

        // SAFETY: We are assuming that the MMIO address is correct and that the compiler won't try
        // to do some funny things with the reference.
        let regs = unsafe { &mut *MiniUARTRegisters::get() };
        regs.enables.write(1);
        regs.control.write(0);
        regs.ier.write(0);
        regs.lcr.write(0b11); // 8-bit mode

        // For rasp3, which has a clock frequency of 250 MHz
        regs.baud_rate.write(baud_divisor as u32);
        regs.control.write(3);

        // Replace the `Option` with `Some` in order to signal that the Mini UART has been setup.
        self.guard.replace(regs);
    }

    /// Sends a single byte through the UART. Spins while there is no space in the UART send
    /// buffer.
    pub fn send(&mut self, byte: u8) {
        let regs = self
            .guard
            .as_mut()
            .expect("Mini UART is not setup while trying to send data");

        while regs.lsr.read() & 0x20 == 0 {
            cortex_a::asm::nop();
        }

        regs.io.write(byte as u32);
    }

    /// Blocks until a byte is received through the UART. This function uses a spin lock to
    /// implement the blocking.
    pub fn recv(&mut self) -> u8 {
        let regs = self
            .guard
            .as_mut()
            .expect("Mini UART is not setup while trying to receive data");

        while regs.lsr.read() & 0x1 == 0 {
            cortex_a::asm::nop();
        }

        (regs.io.read() & 0xff) as u8
    }

    /// Writes a buffer of bytes to the UART.
    pub fn write(&mut self, buf: &[u8]) {
        for &byte in buf {
            self.send(byte);
        }
    }
}

impl core::fmt::Write for MiniUART {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.write(s.as_bytes());
        Ok(())
    }
}

#[inline(always)]
pub fn mu_is_setup() -> bool {
    MiniUART::is_setup()
}

#[inline(always)]
pub fn mu_recv() -> u8 {
    MiniUART::acquire().recv()
}

#[inline(always)]
pub fn mu_send(byte: u8) {
    MiniUART::acquire().send(byte)
}

#[doc(hidden)]
pub fn _mu_print(args: fmt::Arguments) {
    unsafe {
        if !MiniUART::is_setup() {
            panic!("Mini UART is expected to be initialized before calling `_print`");
        }
        let mut mini_uart = MiniUART::acquire();
        mini_uart.write_fmt(args).unwrap_unchecked();
    }
}

/// Print through the Mini UART (MU)
pub macro mu_print($($tok:tt)*) {
    _mu_print(format_args!($($tok)*))
}

/// Print through the Mini UART (MU) followed by a newline.
pub macro mu_println {
    () => {
        _mu_print(format_args!("\n"))
    },

    ($($tok:tt)+) => {
        _mu_print(format_args_nl!($($tok)*));
    }
}
