#ifndef FLOPPY_BIRD_GAME_H
#define FLOPPY_BIRD_GAME_H

#include <stdbool.h>
#include <stdint.h>

enum {
    BIRD_Q8 = 256,
    BIRD_X = 28,
    BIRD_ROWS = 64,
    BIRD_COLS = 264,
    BIRD_START_Y_Q8 = 32 * BIRD_Q8
};

typedef struct {
    int16_t bird_y_q8;
    int16_t velocity_q8;
    int16_t barrier_x;
    uint8_t gap_row;
    uint8_t score;
    uint8_t seed;
    bool passed_barrier;
    bool game_over;
} BirdGame_t;

void bird_game_init(BirdGame_t* game);
void bird_game_flap(BirdGame_t* game);
void bird_game_tick(BirdGame_t* game);
uint8_t bird_game_bird_row(const BirdGame_t* game);
bool bird_game_barrier_at(const BirdGame_t* game, uint8_t x, uint8_t row);

#endif
