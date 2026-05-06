OUTPUT_FORMAT(elf32-m68k)

SECTIONS
{
    . = 0x0;
    .text : {
        KEEP(*(.text.alpha_usb_entry))
        *(.text*)
        *(.rodata*)
    }
    /DISCARD/ : {
        *(.data*)
        *(.bss*)
        *(.comment*)
        *(.eh_frame*)
        *(.note*)
        *(.got*)
        *(.got.plt*)
        *(.rela*)
        *(.rel*)
        *(*)
    }
}
