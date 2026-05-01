#ifndef WRITE_OR_DIE_APP_STATE_H
#define WRITE_OR_DIE_APP_STATE_H

#include <stdint.h>

#define WOD_SCREEN_COLS 28
#define WOD_TEXT_ROWS 3
#define WOD_MAX_TEXT_BYTES 768

typedef struct {
    uint32_t len;
    uint32_t cursor;
    uint32_t viewport;
    char bytes[WOD_MAX_TEXT_BYTES];
} WodEditor_t;

#endif
