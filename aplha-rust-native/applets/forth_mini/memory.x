ENTRY(alpha_usb_entry)

SECTIONS
{
  . = 0;
  .text : {
    *(.text.alpha_usb_entry)
    *(.text .text.*)
    *(.rodata .rodata.*)
  }
}
