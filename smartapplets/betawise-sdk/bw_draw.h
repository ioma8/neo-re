#ifndef BETAWISE_BW_DRAW_H
#define BETAWISE_BW_DRAW_H

#include <stdint.h>

#include "os3k.h"

typedef struct {
    int16_t x;
    int16_t y;
    int16_t w;
    int16_t h;
} BwRect;

static inline void bw_fill_rect(uint16_t x, uint16_t y, uint16_t w, uint16_t h, RopCode_e rop) {
    RasterOp(x, y, w, h, 0, rop);
}

static inline void bw_fill_rect_clipped(BwRect rect, int16_t max_w, int16_t max_h, RopCode_e rop) {
    if(rect.x < 0) {
        rect.w = (int16_t)(rect.w + rect.x);
        rect.x = 0;
    }
    if(rect.y < 0) {
        rect.h = (int16_t)(rect.h + rect.y);
        rect.y = 0;
    }
    if(rect.x + rect.w > max_w) rect.w = (int16_t)(max_w - rect.x);
    if(rect.y + rect.h > max_h) rect.h = (int16_t)(max_h - rect.y);
    if(rect.w > 0 && rect.h > 0) {
        RasterOp((uint16_t)rect.x, (uint16_t)rect.y, (uint16_t)rect.w, (uint16_t)rect.h, 0, rop);
    }
}

static inline void bw_clear_screen(void) {
    ClearScreen();
}

static inline void bw_hide_cursor(void) {
    SetCursorMode(CURSOR_MODE_HIDE);
}

#endif
