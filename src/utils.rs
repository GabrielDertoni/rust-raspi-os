use cortex_a::asm;

#[inline(always)]
pub(crate) fn inifinite_loop() -> ! {
    loop {
        asm::wfe();
    }
}

pub fn delay_cycles(cycles: usize) {
    unsafe {
        core::arch::asm!(
            "cbz {count:x}, 2f",
            "1:",
            "  subs {count:x}, {count:x}, #1",
            "  bne 1b",
            "2:",
            count = in(reg) cycles,
        );
    }
}
