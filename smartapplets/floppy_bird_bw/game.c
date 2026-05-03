#include "game.h"

enum {
    GRAVITY_Q8 = 45,
    FLAP_Q8 = -150,
    MAX_FALL_Q8 = 150,
    BARRIER_RESET_X = BIRD_COLS - 1
};

static uint8_t NextGap(BirdGame_t* game) {
    game->seed = (uint8_t)(game->seed * 33u + 17u + game->score);
    return (uint8_t)(game->seed % (BIRD_ROWS - 1));
}

void bird_game_init(BirdGame_t* game) {
    game->bird_y_q8 = BIRD_START_Y_Q8;
    game->velocity_q8 = 0;
    game->barrier_x = BIRD_COLS - 1;
    game->gap_row = 1;
    game->score = 0;
    game->seed = 7;
    game->passed_barrier = false;
    game->game_over = false;
}

void bird_game_flap(BirdGame_t* game) {
    if(game->game_over) {
        bird_game_init(game);
        return;
    }
    game->velocity_q8 = FLAP_Q8;
}

uint8_t bird_game_bird_row(const BirdGame_t* game) {
    int16_t row = (int16_t)((game->bird_y_q8 + BIRD_Q8 / 2) / BIRD_Q8);
    if(row < 0) return 0;
    if(row >= BIRD_ROWS) return BIRD_ROWS - 1;
    return (uint8_t)row;
}

bool bird_game_barrier_at(const BirdGame_t* game, uint8_t x, uint8_t row) {
    if((int16_t)x != game->barrier_x) return false;
    return row != game->gap_row && row != (uint8_t)(game->gap_row + 1);
}

void bird_game_tick(BirdGame_t* game) {
    if(game->game_over) return;

    game->velocity_q8 = (int16_t)(game->velocity_q8 + GRAVITY_Q8);
    if(game->velocity_q8 > MAX_FALL_Q8) game->velocity_q8 = MAX_FALL_Q8;
    game->bird_y_q8 = (int16_t)(game->bird_y_q8 + game->velocity_q8);

    if(game->bird_y_q8 < 0 || game->bird_y_q8 > (BIRD_ROWS - 1) * BIRD_Q8) {
        game->game_over = true;
    }

    game->barrier_x--;
    if(!game->passed_barrier && game->barrier_x < BIRD_X) {
        game->score++;
        game->passed_barrier = true;
    }
    if(game->barrier_x < 0) {
        game->barrier_x = BARRIER_RESET_X;
        game->gap_row = NextGap(game);
        game->passed_barrier = false;
    }

    if(game->barrier_x == BIRD_X && bird_game_barrier_at(game, BIRD_X, bird_game_bird_row(game))) {
        game->game_over = true;
    }
}
