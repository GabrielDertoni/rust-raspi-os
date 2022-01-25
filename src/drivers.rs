#![allow(dead_code)]

pub mod gpio;
pub mod mini_uart;

pub use gpio::GPIO;
pub use mini_uart::{mu_recv, mu_send, mu_is_setup, MiniUART};

pub const MMIO_BASE_ADDR: usize = 0x3F000000;

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
