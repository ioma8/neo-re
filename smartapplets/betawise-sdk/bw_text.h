#ifndef BETAWISE_BW_TEXT_H
#define BETAWISE_BW_TEXT_H

#include <stdint.h>

#include "bw_draw.h"

static inline uint8_t bw_digit_3x5_bits(uint8_t digit, uint8_t row) {
    static const uint8_t digits[10][5] = {
        {7, 5, 5, 5, 7}, {2, 6, 2, 2, 7}, {7, 1, 7, 4, 7}, {7, 1, 7, 1, 7}, {5, 5, 7, 1, 1},
        {7, 4, 7, 1, 7}, {7, 4, 7, 5, 7}, {7, 1, 1, 1, 1}, {7, 5, 7, 5, 7}, {7, 5, 7, 1, 7}
    };
    return digits[digit][row];
}

static inline uint8_t bw_letter_3x5_bits(char c, uint8_t row) {
    switch(c) {
        case 'A': { static const uint8_t v[5] = {2, 5, 7, 5, 5}; return v[row]; }
        case 'C': { static const uint8_t v[5] = {7, 4, 4, 4, 7}; return v[row]; }
        case 'E': { static const uint8_t v[5] = {7, 4, 6, 4, 7}; return v[row]; }
        case 'F': { static const uint8_t v[5] = {7, 4, 6, 4, 4}; return v[row]; }
        case 'G': { static const uint8_t v[5] = {7, 4, 5, 5, 7}; return v[row]; }
        case 'K': { static const uint8_t v[5] = {5, 6, 4, 6, 5}; return v[row]; }
        case 'L': { static const uint8_t v[5] = {4, 4, 4, 4, 7}; return v[row]; }
        case 'M': { static const uint8_t v[5] = {5, 7, 7, 5, 5}; return v[row]; }
        case 'N': { static const uint8_t v[5] = {5, 7, 7, 7, 5}; return v[row]; }
        case 'O': { static const uint8_t v[5] = {7, 5, 5, 5, 7}; return v[row]; }
        case 'P': { static const uint8_t v[5] = {6, 5, 6, 4, 4}; return v[row]; }
        case 'R': { static const uint8_t v[5] = {6, 5, 6, 5, 5}; return v[row]; }
        case 'S': { static const uint8_t v[5] = {7, 4, 7, 1, 7}; return v[row]; }
        case 'T': { static const uint8_t v[5] = {7, 2, 2, 2, 2}; return v[row]; }
        case 'U': { static const uint8_t v[5] = {5, 5, 5, 5, 7}; return v[row]; }
        case 'V': { static const uint8_t v[5] = {5, 5, 5, 5, 2}; return v[row]; }
        default: return 0;
    }
}

static inline uint8_t bw_glyph_3x5_bits(char c, uint8_t row) {
    if(c >= '0' && c <= '9') return bw_digit_3x5_bits((uint8_t)(c - '0'), row);
    return bw_letter_3x5_bits(c, row);
}

static inline void bw_draw_glyph_3x5(char c, uint16_t x, uint16_t y, uint8_t scale) {
    for(uint8_t row = 0; row < 5; row++) {
        uint8_t bits = bw_glyph_3x5_bits(c, row);
        for(uint8_t col = 0; col < 3; col++) {
            if((bits & (1u << (2u - col))) != 0) {
                bw_fill_rect((uint16_t)(x + col * scale), (uint16_t)(y + row * scale), scale, scale, ROP_BLACKNESS);
            }
        }
    }
}

static inline void bw_draw_digit_3x5(uint8_t digit, uint16_t x, uint16_t y, uint8_t scale) {
    bw_draw_glyph_3x5((char)('0' + digit), x, y, scale);
}

static inline void bw_draw_text_3x5(const char* text, uint16_t x, uint16_t y, uint8_t scale) {
    while(*text != '\0') {
        bw_draw_glyph_3x5(*text, x, y, scale);
        x = (uint16_t)(x + 4u * scale);
        text++;
    }
}

static inline void bw_draw_u8_3digits(uint8_t value, uint16_t x, uint16_t y, uint8_t scale) {
    bw_draw_digit_3x5((uint8_t)((value / 100u) % 10u), x, y, scale);
    bw_draw_digit_3x5((uint8_t)((value / 10u) % 10u), (uint16_t)(x + 4u * scale), y, scale);
    bw_draw_digit_3x5((uint8_t)(value % 10u), (uint16_t)(x + 8u * scale), y, scale);
}

static inline void bw_draw_u8_2digits(uint8_t value, uint16_t x, uint16_t y, uint8_t scale) {
    bw_draw_digit_3x5((uint8_t)((value / 10u) % 10u), x, y, scale);
    bw_draw_digit_3x5((uint8_t)(value % 10u), (uint16_t)(x + 4u * scale), y, scale);
}

#endif
