use super::{Reg32, MMIO_BASE_ADDR};

use crate::utils::delay_cycles;

#[repr(C)]
struct GPIOPinData {
    _reserved: Reg32,
    data: [Reg32; 2],
}

#[repr(C)]
struct GPIORegisters {
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
    Input = 0b000,
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
}

pub struct GPIO {
    regs: &'static mut GPIORegisters,
}

impl GPIO {
    pub fn acquire() -> Self {
        // FIXME: This should be thread safe.
        unsafe {
            GPIO {
                regs: GPIORegisters::get(),
            }
        }
    }

    pub fn set_pin_func(&mut self, pin: u8, func: GPIOFunc) {
        let bit: u32 = (pin as u32 * 3) % 30;
        let reg_idx = (pin / 10) as usize;
        let mut reg = self.regs.func_select[reg_idx].read();

        // Clear the 3 bits of the function select.
        reg &= !(0b111 << bit);
        // Set the bits to the desired values.
        reg |= (func as u32) << bit;

        self.regs.func_select[reg_idx].write(reg);
    }

    pub fn pin_enable(&mut self, pin: u8) {
        self.regs.pullup_pulldown_enable.write(0);
        delay_cycles(150);
        self.regs.pullup_pulldown_clocks[(pin / 32) as usize].write(1 << (pin % 32));
        delay_cycles(150);
        self.regs.pullup_pulldown_enable.write(0);
        self.regs.pullup_pulldown_clocks[(pin / 32) as usize].write(0);
    }
}
