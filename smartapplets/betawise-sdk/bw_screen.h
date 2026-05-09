#ifndef BETAWISE_BW_SCREEN_H
#define BETAWISE_BW_SCREEN_H

#include <stdbool.h>
#include <stdint.h>

#define BW_SCREEN_SAFE_COLS 41

void bw_screen_clear_line(char* line, uint8_t width);
void bw_screen_format_line(char* line, uint8_t width, const char* text);
bool bw_screen_same_line(const char* left, const char* right, uint8_t width);
void bw_screen_copy_line(char* target, const char* source, uint8_t width);
void bw_screen_invalidate_cache(char* cache);
void bw_screen_put_line(uint8_t row, const char* text, uint8_t width);
void bw_screen_put_cached_line(uint8_t row, const char* text, char* cache, uint8_t width);
void bw_screen_set_reverse(bool enabled);
void bw_screen_set_reverse_cached(uint32_t* cached_state, bool enabled);
void bw_screen_clear_reverse_cache(uint32_t* cached_state);

#endif
