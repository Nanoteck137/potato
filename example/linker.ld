ENTRY(kernel_entry)

SECTIONS {
    . = 1M;

    .text :
    {
        *(.text .text.*)
    }

    .rodata :
    {
        *(.rodata .rodata.*)
    }

    .data.rel.ro :
    {
        *(.data.rel.ro.local*) *(.data.rel.ro .data.rel.ro.*)
    }
}
