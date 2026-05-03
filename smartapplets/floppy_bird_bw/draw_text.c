#include "draw_text.h"

#include "os3k.h"

static uint8_t DigitBits(uint8_t digit, uint8_t row) {
    static const uint8_t digits[10][5] = {
        {7, 5, 5, 5, 7}, {2, 6, 2, 2, 7}, {7, 1, 7, 4, 7}, {7, 1, 7, 1, 7}, {5, 5, 7, 1, 1},
        {7, 4, 7, 1, 7}, {7, 4, 7, 5, 7}, {7, 1, 1, 1, 1}, {7, 5, 7, 5, 7}, {7, 5, 7, 1, 7}
    };
    return digits[digit][row];
}

static uint8_t LetterBits(char c, uint8_t row) {
    switch(c) {
        case 'A': { static const uint8_t v[5] = {2, 5, 7, 5, 5}; return v[row]; }
        case 'C': { static const uint8_t v[5] = {7, 4, 4, 4, 7}; return v[row]; }
        case 'E': { static const uint8_t v[5] = {7, 4, 6, 4, 7}; return v[row]; }
        case 'F': { static const uint8_t v[5] = {7, 4, 6, 4, 4}; return v[row]; }
        case 'G': { static const uint8_t v[5] = {7, 4, 5, 5, 7}; return v[row]; }
        case 'L': { static const uint8_t v[5] = {4, 4, 4, 4, 7}; return v[row]; }
        case 'M': { static const uint8_t v[5] = {5, 7, 7, 5, 5}; return v[row]; }
        case 'O': { static const uint8_t v[5] = {7, 5, 5, 5, 7}; return v[row]; }
        case 'R': { static const uint8_t v[5] = {6, 5, 6, 5, 5}; return v[row]; }
        case 'S': { static const uint8_t v[5] = {7, 4, 7, 1, 7}; return v[row]; }
        case 'T': { static const uint8_t v[5] = {7, 2, 2, 2, 2}; return v[row]; }
        case 'V': { static const uint8_t v[5] = {5, 5, 5, 5, 2}; return v[row]; }
        default: return 0;
    }
}

static uint8_t GlyphBits(char c, uint8_t row) {
    if(c >= '0' && c <= '9') return DigitBits((uint8_t)(c - '0'), row);
    return LetterBits(c, row);
}

void bird_draw_digit(uint8_t digit, uint16_t x, uint16_t y) {
    for(uint8_t row = 0; row < 5; row++) {
        uint8_t bits = DigitBits(digit, row);
        for(uint8_t col = 0; col < 3; col++) {
            if((bits & (1u << (2u - col))) != 0) {
                RasterOp((uint16_t)(x + col * 2), (uint16_t)(y + row * 2), 2, 2, 0, ROP_BLACKNESS);
            }
        }
    }
}

void bird_draw_char(char c, uint16_t x, uint16_t y, uint8_t scale) {
    for(uint8_t row = 0; row < 5; row++) {
        uint8_t bits = GlyphBits(c, row);
        for(uint8_t col = 0; col < 3; col++) {
            if((bits & (1u << (2u - col))) != 0) {
                RasterOp((uint16_t)(x + col * scale), (uint16_t)(y + row * scale), scale, scale, 0, ROP_BLACKNESS);
            }
        }
    }
}

void bird_draw_text(const char* text, uint16_t x, uint16_t y, uint8_t scale) {
    while(*text != '\0') {
        bird_draw_char(*text, x, y, scale);
        x = (uint16_t)(x + 4u * scale);
        text++;
    }
}
