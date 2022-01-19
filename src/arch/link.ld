__binary_load_address = 0x80000;

SECTIONS
{
    .boot_stack (NOLOAD) :
    {
        . += __binary_load_address;
        __boot_stack_end = .;
    }

    .text : {
        KEEP(*(.text._start))
        *(.text._start_arguments) /* Constants (or statics in Rust speak) read by _start(). */
        *(.text._start_rust)      /* The Rust entry point */
        *(.text*)                 /* Everything else */
    }

    .rodata : ALIGN(8) { *(.rodata*) }
    .got    : ALIGN(8) { *(.got)     }

    .data : { *(.data) }

    .bss (NOLOAD) : ALIGN(16)
    {
        __bss_start = .;
        *(*.bss);
        . = ALIGN(16);
        __bss_end = .;
    }
}
