#ifndef BETAWISE_SCREEN_LINES_H
#define BETAWISE_SCREEN_LINES_H

#include <stdbool.h>
#include <stdint.h>

#define APPLET_SCREEN_SAFE_COLS 41

void applet_screen_clear_line(char* line, uint8_t width);
void applet_screen_format_line(char* line, uint8_t width, const char* text);
bool applet_screen_same_line(const char* left, const char* right, uint8_t width);
void applet_screen_copy_line(char* target, const char* source, uint8_t width);
void applet_screen_invalidate_cache(char* cache);
void applet_screen_put_line(uint8_t row, const char* text, uint8_t width);
void applet_screen_put_cached_line(uint8_t row, const char* text, char* cache, uint8_t width);

#endif
