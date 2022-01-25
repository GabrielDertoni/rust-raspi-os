.global _start
.global _child_spin
.section .text._start

_start:
    mrs x1, MPIDR_EL1
    and x1, x1, 0xff
    ldr x2, BOOT_CORE_ID
    cmp x1, x2
    bne _child_spin

    adr x0, __bss_start
    adr x1, __bss_end

.L_zero_bss_loop:
    cmp x0, x1
    beq .L_jump_rust
    str xzr, [x0], #8
    b   .L_zero_bss_loop

.L_jump_rust:
    adr x0, __boot_stack_end
    mov sp, x0
    b   _start_rust

_child_spin:
    wfe
    ldr x0, _child_target
    cbz x0, _child_spin

    mrs x2, MPIDR_EL1          // Get the cpu id
    and x2, x2, 0xff

    ldr x1, CHILD_STACK_SIZE_E // Get the child stack size constant
    mul x1, x1, x2             // Calculate the offset into the `CHILD_STACKS` array
    adr x2, CHILD_STACKS       // Get a pointer to the `CHILD_STACKS` array.
    add x2, x2, x1             // Apply the offset to the pointer.
    mov sp, x2                 // Set the stack pointer to that the pointer.
    br  x0

.section .data

.global _child_target
_child_target: .word 0
