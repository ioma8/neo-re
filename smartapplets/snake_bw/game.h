#ifndef SNAKE_GAME_H
#define SNAKE_GAME_H

#include <stdbool.h>
#include <stdint.h>

enum {
    SNAKE_COLS = 50,
    SNAKE_ROWS = 16,
    SNAKE_MAX_LEN = 192
};

typedef enum {
    SNAKE_UP = 0,
    SNAKE_RIGHT = 1,
    SNAKE_DOWN = 2,
    SNAKE_LEFT = 3
} SnakeDirection_t;

typedef struct {
    uint8_t body_x[SNAKE_MAX_LEN];
    uint8_t body_y[SNAKE_MAX_LEN];
    uint8_t length;
    uint8_t head_x;
    uint8_t head_y;
    uint8_t food_x;
    uint8_t food_y;
    uint8_t score;
    uint8_t seed;
    SnakeDirection_t direction;
    SnakeDirection_t pending_direction;
    bool paused;
    bool game_over;
} SnakeGame_t;

void snake_game_init(SnakeGame_t* game);
void snake_game_init_seeded(SnakeGame_t* game, uint8_t seed);
void snake_game_restart(SnakeGame_t* game);
void snake_game_restart_seeded(SnakeGame_t* game, uint8_t seed);
void snake_game_turn(SnakeGame_t* game, SnakeDirection_t direction);
void snake_game_toggle_pause(SnakeGame_t* game);
void snake_game_tick(SnakeGame_t* game);
bool snake_game_contains(const SnakeGame_t* game, uint8_t x, uint8_t y);

#endif
