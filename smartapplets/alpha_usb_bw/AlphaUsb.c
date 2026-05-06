/*
 * Alpha USB - Betawise C port of the Rust native Alpha USB SmartApplet.
 *
 * When launched, shows instructions to connect USB.
 * On USB plug (MSG_USB_PLUG), switches the NEO from HID keyboard mode
 * to direct USB mode (081e:bd04 -> 081e:bd01).
 *
 * Uses yield loop (like Rust idle_forever) to stay resident for USB events.
 * MSG_KEY escape/applets dispatched from headless validation proves exit works.
 */

#include <stdint.h>
#include "os3k.h"
#include "applet.h"

#define MSG_USB_PLUG      0x30001u
#define MSG_USB_MAC_INIT  0x10001u
#define MSG_USB_PC_INIT   0x20001u
#define STATUS_USB_HANDLED   0x11u

void _OS3K_ClearScreen(void);

__attribute__((used)) static void complete_hid_to_direct(void) {
    __asm__ volatile (
        ".short 0x42A7\n\t"
        ".short 0x42A7\n\t"
        ".short 0x4878, 0x0001\n\t"
        ".short 0x4EB9\n\t"
        ".long  0x0041F9A0\n\t"
        ".short 0x4878, 0x0064\n\t"
        ".short 0x4EB9\n\t"
        ".long  0x00424780\n\t"
        ".short 0x13FC, 0x0001, 0x0001, 0x3CF9\n\t"
        ".short 0x4EB9\n\t"
        ".long  0x0044044E\n\t"
        ".short 0x4878, 0x0064\n\t"
        ".short 0x4EB9\n\t"
        ".long  0x00424780\n\t"
        ".short 0x4FEF, 0x0014\n\t"
        ".short 0x4EB9\n\t"
        ".long  0x0044047C\n\t"
        :
        :
        : "memory"
    );
}

__attribute__((used)) static void mark_direct_connected(void) {
    __asm__ volatile (
        ".short 0x4EB9\n\t"
        ".long  0x00410B26\n\t"
        :
        :
        : "memory"
    );
}

void handle_msg(uint32_t message, uint32_t param, uint32_t* status);

APPLET_ENTRY(handle_msg);

__attribute__((used)) void handle_msg(uint32_t message, uint32_t param, uint32_t* status) {
    (void)param;

    if (message == MSG_INIT) {
        *status = APPLET_UNHANDLED_STATUS;
        return;
    }

    if (message == MSG_SETFOCUS) {
        _OS3K_ClearScreen();
        PutStringCentered(2, "Now connect the NEO");
        PutStringCentered(3, "to your computer or");
        PutStringCentered(4, "smartphone via USB.");
        for (;;) {
            SYS_A25C(0, 0);
        }
    }

    if (message == MSG_KEY) {
        *status = APPLET_EXIT_STATUS;
        return;
    }

    if (message == MSG_USB_PLUG) {
        complete_hid_to_direct();
        mark_direct_connected();
        *status = STATUS_USB_HANDLED;
        return;
    }

    if (message == MSG_USB_MAC_INIT || message == MSG_USB_PC_INIT) {
        *status = STATUS_USB_HANDLED;
        return;
    }

    *status = APPLET_UNHANDLED_STATUS;
}
