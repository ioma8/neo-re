    .text
    .global alpha_usb_entry
alpha_usb_entry:
    movea.l 12(%sp), %a0
    clr.l (%a0)
    move.l 4(%sp), %d0
    cmpi.l #0x00000019, %d0
    beq.w focus
    cmpi.l #0x00030001, %d0
    beq.w usb_plug
    cmpi.l #0x00000026, %d0
    beq.w identity
    rts

identity:
    move.l #0x0000A130, (%a0)
    rts

usb_plug:
    clr.l -(%sp)
    clr.l -(%sp)
    pea 1
    jsr 0x0041F9A0
    pea 100
    jsr 0x00424780
    move.b #1, 0x00013CF9
    jsr 0x0044044E
    pea 100
    jsr 0x00424780
    lea 20(%sp), %sp
    jsr 0x0044047C
    jsr 0x00410B26
    moveq #0x11, %d1
    move.l %d1, (%a0)
    rts

focus:
    bsr.w trap_clear
    moveq #2, %d0
    lea line1(%pc), %a1
    bsr.w draw_line
    moveq #3, %d0
    lea line2(%pc), %a1
    bsr.w draw_line
    moveq #4, %d0
    lea line3(%pc), %a1
    bsr.w draw_line
    bsr.w trap_flush
idle:
    bsr.w trap_yield
    bra.s idle

draw_line:
    move.l #28, -(%sp)
    move.l #1, -(%sp)
    move.l %d0, -(%sp)
    bsr.w trap_set_row
    lea 12(%sp), %sp
draw_next:
    move.b (%a1)+, %d0
    beq.s draw_done
    andi.l #0x000000FF, %d0
    move.l %d0, -(%sp)
    bsr.w trap_draw_char
    addq.l #4, %sp
    bra.s draw_next
draw_done:
    rts

trap_clear:
    .short 0xA000
trap_set_row:
    .short 0xA004
trap_draw_char:
    .short 0xA010
trap_flush:
    .short 0xA098
trap_yield:
    .short 0xA25C

line1:
    .asciz "Now connect the NEO"
line2:
    .asciz "to your computer or"
line3:
    .asciz "smartphone via USB."
    .even
