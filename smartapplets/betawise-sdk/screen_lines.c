#include "screen_lines.h"

#include "os3k.h"

void _OS3K_SetCursor(uint8_t row, uint8_t col, CursorMode_e cursor_mode);

static uint8_t ClampWidth(uint8_t width) {
    return width > APPLET_SCREEN_SAFE_COLS ? APPLET_SCREEN_SAFE_COLS : width;
}

void applet_screen_clear_line(char* line, uint8_t width) {
    width = ClampWidth(width);
    for(uint8_t i = 0; i < width; i++) {
        line[i] = ' ';
    }
    line[width] = '\0';
}

void applet_screen_format_line(char* line, uint8_t width, const char* text) {
    width = ClampWidth(width);
    applet_screen_clear_line(line, width);
    for(uint8_t i = 0; i < width && text[i] != '\0'; i++) {
        line[i] = text[i];
    }
}

bool applet_screen_same_line(const char* left, const char* right, uint8_t width) {
    width = ClampWidth(width);
    for(uint8_t i = 0; i < width; i++) {
        if(left[i] != right[i]) return false;
    }
    return true;
}

void applet_screen_copy_line(char* target, const char* source, uint8_t width) {
    width = ClampWidth(width);
    for(uint8_t i = 0; i < width; i++) {
        target[i] = source[i];
    }
    target[width] = '\0';
}

void applet_screen_invalidate_cache(char* cache) {
    cache[0] = '\0';
}

void applet_screen_put_line(uint8_t row, const char* text, uint8_t width) {
    char line[APPLET_SCREEN_SAFE_COLS + 1];
    width = ClampWidth(width);
    applet_screen_format_line(line, width, text);
    _OS3K_SetCursor(row, 1, CURSOR_MODE_HIDE);
    PutStringRaw(line);
}

void applet_screen_put_cached_line(uint8_t row, const char* text, char* cache, uint8_t width) {
    char line[APPLET_SCREEN_SAFE_COLS + 1];
    width = ClampWidth(width);
    applet_screen_format_line(line, width, text);
    if(applet_screen_same_line(line, cache, width)) return;
    applet_screen_copy_line(cache, line, width);
    _OS3K_SetCursor(row, 1, CURSOR_MODE_HIDE);
    PutStringRaw(line);
}
