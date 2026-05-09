#ifndef BETAWISE_BW_APP_H
#define BETAWISE_BW_APP_H

#include <stdbool.h>
#include <stdint.h>

#include "os3k.h"

#define BW_STATUS_EXIT 0x07u
#define BW_STATUS_UNHANDLED 0x04u

#define BW_APP_STATE(Type)                                                    \
    static inline Type* State(void) {                                         \
        register char* a5 __asm__("a5");                                      \
        return (Type*)(a5 + 0x300);                                           \
    }

#define BW_APP_ENTRY(Handler)                                                 \
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

static inline uint32_t bw_message_id(uint32_t message) {
    return message & 0xffu;
}

static inline uint32_t bw_key_id(uint32_t param) {
    return param & 0xffu;
}

static inline bool bw_is_exit_key(uint32_t key) {
    return key == KEY_APPLETS || key == KEY_ESC;
}

static inline void bw_set_unhandled(uint32_t* status) {
    *status = BW_STATUS_UNHANDLED;
}

static inline void bw_set_exit(uint32_t* status) {
    *status = BW_STATUS_EXIT;
}

#endif
