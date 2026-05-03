#ifndef SNAKE_DRAW_H
#define SNAKE_DRAW_H

#include <stdint.h>

#include "game.h"

void snake_draw_full(const SnakeGame_t* game);
void snake_draw_step(
    SnakeGame_t* game,
    uint8_t old_tail_x,
    uint8_t old_tail_y,
    uint8_t old_food_x,
    uint8_t old_food_y,
    uint8_t old_length
);

#endif
