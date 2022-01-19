.global _start
.section .text._start

_start:
    mrs x1, MPIDR_EL1
    and x1, x1, 0xff
    ldr x2, BOOT_CORE_ID
    cmp x1, x2
    bne .L_spin

    adr x0, __bss_start
    adr x1, __bss_end

.L_zero_bss_loop:
    cmp x0, x1
    beq .L_jump_rust
    str xzr, [x0], #8
    b .L_zero_bss_loop

.L_jump_rust:
    adr x0, __boot_stack_end
    mov sp, x0
    b _start_rust

.L_spin:
    wfe
    b .L_spin
