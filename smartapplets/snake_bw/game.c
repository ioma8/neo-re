#include "game.h"

static bool Opposite(SnakeDirection_t a, SnakeDirection_t b) {
    return ((uint8_t)a + 2u) % 4u == (uint8_t)b;
}

bool snake_game_contains(const SnakeGame_t* game, uint8_t x, uint8_t y) {
    for(uint8_t i = 0; i < game->length; i++) {
        if(game->body_x[i] == x && game->body_y[i] == y) return true;
    }
    return false;
}

static uint8_t NextRandom(SnakeGame_t* game) {
    game->seed = (uint8_t)(game->seed * 37u + 23u + game->score);
    return game->seed;
}

static void PlaceFood(SnakeGame_t* game) {
    for(uint8_t tries = 0; tries < 96; tries++) {
        uint8_t x = (uint8_t)(NextRandom(game) % SNAKE_COLS);
        uint8_t y = (uint8_t)(NextRandom(game) % SNAKE_ROWS);
        if(!snake_game_contains(game, x, y)) {
            game->food_x = x;
            game->food_y = y;
            return;
        }
    }
    game->food_x = 0;
    game->food_y = 0;
}

void snake_game_init_seeded(SnakeGame_t* game, uint8_t seed) {
    for(uint8_t i = 0; i < SNAKE_MAX_LEN; i++) {
        game->body_x[i] = 0;
        game->body_y[i] = 0;
    }
    game->length = 3;
    game->head_x = SNAKE_COLS / 2;
    game->head_y = SNAKE_ROWS / 2;
    game->body_x[0] = game->head_x;
    game->body_y[0] = game->head_y;
    game->body_x[1] = (uint8_t)(game->head_x - 1);
    game->body_y[1] = game->head_y;
    game->body_x[2] = (uint8_t)(game->head_x - 2);
    game->body_y[2] = game->head_y;
    game->score = 0;
    game->seed = seed == 0 ? 31 : seed;
    game->direction = SNAKE_RIGHT;
    game->pending_direction = SNAKE_RIGHT;
    game->paused = false;
    game->game_over = false;
    PlaceFood(game);
}

void snake_game_init(SnakeGame_t* game) {
    snake_game_init_seeded(game, 31);
}

void snake_game_restart_seeded(SnakeGame_t* game, uint8_t seed) {
    snake_game_init_seeded(game, seed);
}

void snake_game_restart(SnakeGame_t* game) {
    snake_game_restart_seeded(game, 31);
}

void snake_game_turn(SnakeGame_t* game, SnakeDirection_t direction) {
    if(!Opposite(game->direction, direction)) {
        game->pending_direction = direction;
    }
}

void snake_game_toggle_pause(SnakeGame_t* game) {
    if(!game->game_over) {
        game->paused = !game->paused;
    }
}

static bool BodyCollision(const SnakeGame_t* game, uint8_t x, uint8_t y, bool growing) {
    uint8_t limit = growing ? game->length : (uint8_t)(game->length - 1);
    for(uint8_t i = 0; i < limit; i++) {
        if(game->body_x[i] == x && game->body_y[i] == y) return true;
    }
    return false;
}

void snake_game_tick(SnakeGame_t* game) {
    if(game->paused || game->game_over) return;

    int16_t next_x = (int16_t)game->head_x;
    int16_t next_y = (int16_t)game->head_y;
    game->direction = game->pending_direction;
    if(game->direction == SNAKE_UP) next_y--;
    else if(game->direction == SNAKE_DOWN) next_y++;
    else if(game->direction == SNAKE_LEFT) next_x--;
    else next_x++;

    if(next_x < 0) next_x = SNAKE_COLS - 1;
    else if(next_x >= SNAKE_COLS) next_x = 0;
    if(next_y < 0) next_y = SNAKE_ROWS - 1;
    else if(next_y >= SNAKE_ROWS) next_y = 0;

    bool growing = (uint8_t)next_x == game->food_x && (uint8_t)next_y == game->food_y;
    if(BodyCollision(game, (uint8_t)next_x, (uint8_t)next_y, growing)) {
        game->game_over = true;
        return;
    }

    uint8_t new_length = game->length;
    if(growing && new_length < SNAKE_MAX_LEN) {
        new_length++;
        game->score++;
    }
    for(uint8_t i = new_length - 1; i > 0; i--) {
        game->body_x[i] = game->body_x[i - 1];
        game->body_y[i] = game->body_y[i - 1];
    }
    game->head_x = (uint8_t)next_x;
    game->head_y = (uint8_t)next_y;
    game->body_x[0] = game->head_x;
    game->body_y[0] = game->head_y;
    game->length = new_length;
    if(growing) PlaceFood(game);
}
