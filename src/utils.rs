use cortex_a::asm;
use tock_registers::interfaces::Readable;

/// Inifite loop that executes a wait for event (`wfe`) instruction on each iteration, so the
/// processor may enter low power mode.
#[inline(always)]
pub(crate) fn inifinite_loop() -> ! {
    loop {
        asm::wfe();
    }
}

/// Does nothing for approximately `cycles` CPU cycles.
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

/// Gets the current cpu id.
pub fn get_cpu() -> u64 {
    use cortex_a::registers::MPIDR_EL1;
    MPIDR_EL1.get() & 0xff
}

/// Gets the current exception level
pub fn get_current_exception_level() -> u64 {
    use cortex_a::registers::CurrentEL;
    CurrentEL.read(CurrentEL::EL)
}
