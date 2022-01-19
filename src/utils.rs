use cortex_a::asm;

#[inline(always)]
pub(crate) fn inifinite_loop() -> ! {
    loop {
        asm::wfe();
    }
}

pub fn delay_cycles(cycles: usize) {
    unsafe {
        // I am not sure if this `beq end` is necessary, but it might be in order to
        // prevent errors if `cycles` is 0.
        core::arch::asm!("adds x0, xzr, {}
                          beq 2f
                          1:
                            subs x0, x0, #1
                            bne 1b
                          2:",
                         in(reg) cycles);
    }
}

#[allow(dead_code)]
pub fn hacky_write(bytes: &[u8]) {
    for &b in bytes {
        unsafe {
            core::ptr::write_volatile(0x3F20_1000 as *mut u8, b);
        }
    }
}
