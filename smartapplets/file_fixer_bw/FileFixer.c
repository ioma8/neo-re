/* FileMaxSizeFixer v5 - direct memory access via byte operations */
#include <stdint.h>
#include "os3k.h"
#include "applet.h"

void _OS3K_ClearScreen(void);
void _OS3K_SetCursor(uint8_t row, uint8_t col, uint32_t mode);

#define MAXSIZE_ADDR 0x00000fda

void handle_msg(uint32_t msg, uint32_t param, uint32_t* st);
APPLET_ENTRY(handle_msg);

__attribute__((used)) void handle_msg(uint32_t msg, uint32_t param, uint32_t* st) {
    (void)param;
    if (msg == 0x18) { *st = 4; return; }  /* MSG_INIT */
    if (msg == 0x19) {  /* MSG_SETFOCUS */
        volatile uint8_t* p = (volatile uint8_t*)0x00000fda;
        uint32_t tbl = ((uint32_t)p[0]<<24)|((uint32_t)p[1]<<16)|((uint32_t)p[2]<<8)|p[3];
        int i, n = 0;
        _OS3K_ClearScreen();
        if (tbl < 0x400 || tbl > 0x700000) {
            PutStringCentered(2, "bad ptr");
            SleepCentiseconds(200);
            *st = 7; return;
        }
        for (i = 0; i < 8; i++) {
            volatile uint8_t* r = (volatile uint8_t*)(uintptr_t)(tbl + i * 0x48);
            uint32_t cur = ((uint32_t)r[8]<<24)|((uint32_t)r[9]<<16)|((uint32_t)r[10]<<8)|r[11];
            if (cur != 51200) {
                r[8]=0; r[9]=0; r[10]=0xC8; r[11]=0;
                n++;
            }
        }
        if (n) PutStringCentered(2, "fixed!");
        else PutStringCentered(2, "already ok");
        SleepCentiseconds(200);
        *st = 7;
        return;
    }
    *st = 4;
}
