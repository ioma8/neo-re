#ifndef BETAWISE_APPLET_H
#define BETAWISE_APPLET_H

#include <stdint.h>

#define APPLET_EXIT_STATUS 0x07u
#define APPLET_UNHANDLED_STATUS 0x04u

#define APPLET_STATE(Type)                                                    \
    static inline Type* State(void) {                                         \
        register char* a5 __asm__("a5");                                      \
        return (Type*)(a5 + 0x300);                                           \
    }

#define APPLET_ENTRY(Handler)                                                 \
    asm(                                                                      \
        ".section .text.alpha_usb_entry,\"ax\"\n"                             \
        ".global alpha_usb_entry\n"                                           \
        "alpha_usb_entry:\n"                                                  \
        "move.l 12(%sp),-(%sp)\n"                                             \
        "move.l 12(%sp),-(%sp)\n"                                             \
        "move.l 12(%sp),-(%sp)\n"                                             \
        "bsr " #Handler "\n"                                                  \
        "lea 12(%sp),%sp\n"                                                   \
        "rts\n"                                                               \
        ".text\n"                                                             \
    )

#endif
