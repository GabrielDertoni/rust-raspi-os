use core::{
    fmt::{self, Write},
    sync::atomic::{AtomicBool, Ordering},
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
    pub unsafe fn get() -> &'static mut Self {
        let ptr = Self::REGS_ADDR as *mut Self;
        &mut *ptr
    }
}

static mut IN_USE: AtomicBool = AtomicBool::new(false);
static mut IS_SETUP: bool = false;

/// Structure that represents an exclusive handle to the Mini UART.
pub struct MiniUART {
    regs: &'static mut MiniUARTRegisters,
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
        // SAFETY: `IN_USE` is atomic.
        unsafe {
            // FIXME: I am not sure those orderings are correct.
            while let Err(_) = IN_USE.compare_exchange(false, true, Ordering::SeqCst, Ordering::Acquire) {
                // We may enter low power mode until an event occurs.
                // NOTE: When the `MiniUART` is dropped, if there is anyone in the queue waiting
                // for the lock, it uses the signal event (`sev`) instruction to wake up those
                // cores.
                cortex_a::asm::wfe();
            }
            MiniUART { regs: MiniUARTRegisters::get() }
        }
    }

    /// Checks whether the Mini UART is setup.
    pub fn is_setup(&self) -> bool {
        // SAFETY: This is only a read and so no race condition can occur. No other thread can
        // write while this read occurs because that would require an exclusive reference.
        unsafe { IS_SETUP }
    }

    /// Initializes the Mini UART with the default baud rate of ~115200 @ 250 MHz
    pub fn init_default(&mut self, gpio: &mut GPIO) {
        self.init(gpio, 270);
    }

    // FIXME: The link to the specs is not the original link. Should be replaced
    ///
    /// Initializes the Mini UART. The baud rate divisor is used to calculate the baud rate of the
    /// UART which is based on the clock frequency. To calculate the baud rate the following
    /// formula can be used [according to the spec](https://cs140e.sergio.bz/docs/BCM2837-ARM-Peripherals.pdf):
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

        self.regs.enables.write(1);
        self.regs.control.write(0);
        self.regs.ier.write(0);
        self.regs.lcr.write(0b11); // 8-bit mode

        // For rasp3, which has a clock frequency of 250 MHz
        self.regs.baud_rate.write(baud_divisor as u32);

        self.regs.control.write(3);

        // SAFETY: The thread that has acquired the `MiniUART` has has exclusive access to it
        // because this function requires `&mut`. So it is the only thread mutating this global.
        unsafe { IS_SETUP = true; }
    }

    /// Sends a single byte through the UART. Spins while there is no space in the UART send
    /// buffer.
    pub fn send(&mut self, byte: u8) {
        if !self.is_setup() {
            panic!("Mini UART is not setup while trying to send data");
        }

        while self.regs.lsr.read() & 0x20 == 0 {
            cortex_a::asm::nop();
        }

        self.regs.io.write(byte as u32);
    }

    /// Blocks until a byte is received through the UART. This function uses a spin lock to
    /// implement the blocking.
    pub fn recv(&mut self) -> u8 {
        if !self.is_setup() {
            panic!("Mini UART is not setup while trying to send data");
        }

        while self.regs.lsr.read() & 0x1 == 0 {
            cortex_a::asm::nop();
        }
        (self.regs.io.read() & 0xff) as u8
    }

    /// Writes a buffer of bytes to the UART.
    pub fn write(&mut self, buf: &[u8]) {
        for &byte in buf {
            self.send(byte);
        }
    }
}

impl Drop for MiniUART {
    fn drop(&mut self) {
        unsafe { IN_USE.store(false, Ordering::Release); }
        // Wake the cores up. If some core was waiting for the lock, it can now acquire it.
        // TODO: Maybe it is worth it to use some `WAITING` count and check whether there is
        // actually some thread waiting for the lock, and only then use `sev`.
        cortex_a::asm::sev();
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
    // SAFETY: This is only a read.
    unsafe { IS_SETUP }
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
        if !IS_SETUP {
            panic!("Mini UART is expected to be initialized before calling `_print`");
        }
        let mut mini_uart = MiniUART::acquire();
        mini_uart.write_fmt(args).unwrap_unchecked();
    }
}

/// Print through the Mini UART (MU)
#[macro_export]
macro_rules! mu_print {
    ($($tok:tt)*) => ({
        $crate::drivers::mini_uart::_mu_print(format_args!($($tok)*))
    });
}

/// Print through the Mini UART (MU) followed by a newline.
#[macro_export]
macro_rules! mu_println {
    () => ({
        $crate::drivers::mini_uart::_mu_print("\n");
    });

    ($($tok:tt)*) => ({
        $crate::drivers::mini_uart::_mu_print(format_args_nl!($($tok)*));
    });
}
