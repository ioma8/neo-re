#include "draw_text.h"

#include "../betawise-sdk/bw.h"

void bird_draw_digit(uint8_t digit, uint16_t x, uint16_t y) {
    bw_draw_digit_3x5(digit, x, y, 2);
}

void bird_draw_char(char c, uint16_t x, uint16_t y, uint8_t scale) {
    bw_draw_glyph_3x5(c, x, y, scale);
}

void bird_draw_text(const char* text, uint16_t x, uint16_t y, uint8_t scale) {
    bw_draw_text_3x5(text, x, y, scale);
}
