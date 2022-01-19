#![allow(dead_code)]

use crate::utils::delay_cycles;

const MMIO_BASE_ADDR: usize = 0x3F000000;

#[repr(transparent)]
struct Reg32(u32);

impl Reg32 {
    unsafe fn new(addr: usize) -> &'static mut Reg32 {
        let ptr = addr as *mut Reg32;
        &mut *ptr
    }

    fn write(&mut self, val: u32) {
        unsafe {
            core::ptr::write_volatile(&mut self.0, val);
        }
    }

    fn read(&mut self) -> u32 {
        unsafe {
            core::ptr::read_volatile(&self.0)
        }
    }
}


#[repr(C)]
struct GPIOPinData {
    _reserved: Reg32,
    data: [Reg32; 2],
}

#[repr(C)]
pub struct GPIORegisters {
    func_select: [Reg32; 6],
    output_set: GPIOPinData,
    output_clear: GPIOPinData,
    level: GPIOPinData,
    event_detect_status: GPIOPinData,
    rising_edge_detect_enable: GPIOPinData,
    falling_edge_detect_enable: GPIOPinData,
    pin_high_detect_enable: GPIOPinData,
    pin_low_detect_enable: GPIOPinData,
    pin_async_rising_edge_detect: GPIOPinData,
    pin_async_falling_edge_detect: GPIOPinData,
    _reserved: Reg32,
    pullup_pulldown_enable: Reg32,
    pullup_pulldown_clocks: [Reg32; 2],
}

pub enum GPIOFunc {
    Input  = 0b000,
    Output = 0b001,
    AltFn0 = 0b100,
    AltFn1 = 0b101,
    AltFn2 = 0b110,
    AltFn3 = 0b111,
    AltFn4 = 0b011,
    AltFn5 = 0b010,
}

impl GPIORegisters {
    const REGS_ADDR: usize = MMIO_BASE_ADDR + 0x20000;

    /// # Safety
    ///
    /// This function doesn't garantee that there can always only be a single
    /// mutable reference. It is possible to create multiple aliassing to the
    /// same memory location by calling this function many times. As such, it is
    /// marked as unsafe.
    #[inline(always)]
    pub unsafe fn get() -> &'static mut Self {
        let ptr = Self::REGS_ADDR as *mut GPIORegisters;
        &mut *ptr
    }

    pub fn set_pin_func(&mut self, pin: u8, func: GPIOFunc) {
        let bit = (pin * 3) % 30;
        let reg_idx = (pin / 10) as usize;
        let mut reg = self.func_select[reg_idx].read();

        // Clear the 3 bits of the function select.
        reg &= !(0b111 << bit);
        // Set the bits to the desired values.
        reg |= ((func as u8) << bit) as u32;

        self.func_select[reg_idx].write(reg);
    }

    pub fn pin_enable(&mut self, pin: u8) {
        self.pullup_pulldown_enable.write(0);
        delay_cycles(150);
        self.pullup_pulldown_clocks[(pin / 32) as usize].write(1 << (pin % 32));
        delay_cycles(150);
        self.pullup_pulldown_enable.write(0);
        self.pullup_pulldown_clocks[(pin / 32) as usize].write(0);
    }
}


#[repr(C)]
pub struct MiniUARTRegisters {
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
    const TX_PIN: u8 = 14;
    const RX_PIN: u8 = 15;

    #[inline(always)]
    pub unsafe fn get() -> &'static mut Self {
        let ptr = Self::REGS_ADDR as *mut Self;
        &mut *ptr
    }

    pub fn init(&mut self, gpio: &mut GPIORegisters) {
        gpio.set_pin_func(Self::TX_PIN, GPIOFunc::AltFn5);
        gpio.set_pin_func(Self::RX_PIN, GPIOFunc::AltFn5);

        gpio.pin_enable(Self::TX_PIN);
        gpio.pin_enable(Self::RX_PIN);

        self.enables.write(1);
        self.control.write(0);
        self.ier.write(0);
        self.lcr.write(0b11); // 8-bit mode

        // For rasp3, which has a clock frequency of 250 MHz
        self.baud_rate.write(270); // 115200 @ 250 MHz

        self.control.write(3);
    }

    pub fn send(&mut self, byte: u8) {
        while self.lsr.read() & 0x20 == 0 {
            cortex_a::asm::nop();
        }

        self.io.write(byte as u32);
    }

    pub fn recv(&mut self) -> u8 {
        while self.lsr.read() & 0x1 == 0 {
            cortex_a::asm::nop();
        }
        (self.io.read() & 0xff) as u8
    }

    pub fn write(&mut self, buf: &[u8]) {
        for &byte in buf {
            self.send(byte);
        }
    }
}

impl core::fmt::Write for MiniUARTRegisters {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.write(s.as_bytes());
        Ok(())
    }
}
