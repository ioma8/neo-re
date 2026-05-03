#ifndef FLOPPY_BIRD_DRAW_TEXT_H
#define FLOPPY_BIRD_DRAW_TEXT_H

#include <stdint.h>

void bird_draw_digit(uint8_t digit, uint16_t x, uint16_t y);
void bird_draw_char(char c, uint16_t x, uint16_t y, uint8_t scale);
void bird_draw_text(const char* text, uint16_t x, uint16_t y, uint8_t scale);

#endif
