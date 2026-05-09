#include "game.h"

enum {
    GRAVITY_Q8 = 18,
    FLAP_Q8 = -96,
    MAX_FALL_Q8 = 120,
    BARRIER_WIDTH = 8,
    BARRIER_SPEED_BASE = 3,
    BARRIER_SPEED_MAX = 7,
    GAP_HEIGHT_BASE = 20,
    GAP_HEIGHT_MIN = 14,
    GAP_MARGIN = 6,
    BARRIER_RESET_X = BIRD_COLS - 1,
    BARRIER_SPACING = 140
};

static uint8_t NextGap(BirdGame_t* game, uint8_t gap_h) {
    game->seed = (uint8_t)(game->seed * 33u + 17u + game->score);
    uint8_t range = (uint8_t)(BIRD_ROWS - gap_h - GAP_MARGIN * 2);
    uint8_t offset = (uint8_t)((uint16_t)game->seed % (uint16_t)range);
    return (uint8_t)(GAP_MARGIN + offset);
}

static uint8_t GapHeight(uint8_t score) {
    uint8_t reduce = score / 3u;
    if(reduce > GAP_HEIGHT_BASE - GAP_HEIGHT_MIN) {
        reduce = GAP_HEIGHT_BASE - GAP_HEIGHT_MIN;
    }
    return (uint8_t)(GAP_HEIGHT_BASE - reduce);
}

static void ResetBarrier(BirdGame_t* game, int16_t* bx, uint8_t* gr, uint8_t* gh, bool* passed) {
    *gh = GapHeight(game->score);
    *gr = NextGap(game, *gh);
    *bx = BARRIER_RESET_X;
    *passed = false;
}

void bird_game_init(BirdGame_t* game) {
    game->bird_y_q8 = BIRD_START_Y_Q8;
    game->velocity_q8 = 0;
    game->score = 0;
    game->seed = 7;
    game->game_over = false;
    game->barrier_x = BARRIER_RESET_X;
    game->gap_height = GapHeight(0);
    game->gap_row = NextGap(game, game->gap_height);
    game->passed_barrier = false;
    game->barrier2_x = (int16_t)(BARRIER_RESET_X - BARRIER_SPACING);
    game->gap2_height = GapHeight(0);
    game->gap2_row = NextGap(game, game->gap2_height);
    game->passed_barrier2 = false;
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

static bool PointInBarrier(int16_t bx, uint8_t gap_row, uint8_t gap_h, uint8_t x, uint8_t row) {
    int16_t right = (int16_t)(bx + BARRIER_WIDTH - 1);
    if((int16_t)x < bx || (int16_t)x > right) return false;
    return row < gap_row || row >= (uint8_t)(gap_row + gap_h);
}

bool bird_game_barrier_at(const BirdGame_t* game, uint8_t x, uint8_t row) {
    return PointInBarrier(game->barrier_x, game->gap_row, game->gap_height, x, row)
        || PointInBarrier(game->barrier2_x, game->gap2_row, game->gap2_height, x, row);
}

static void MoveBarrier(BirdGame_t* game, int16_t* bx, uint8_t* gr, uint8_t* gh, bool* passed, uint8_t speed) {
    *bx = (int16_t)(*bx - speed);
    if(!*passed && *bx + BARRIER_WIDTH - 1 < BIRD_X) {
        game->score++;
        *passed = true;
    }
    if(*bx < -BARRIER_WIDTH) {
        ResetBarrier(game, bx, gr, gh, passed);
        /* Place behind the other barrier */
        int16_t other = (bx == &game->barrier_x) ? game->barrier2_x : game->barrier_x;
        *bx = (int16_t)(other + BARRIER_SPACING);
        if(*bx < 0) *bx = BARRIER_RESET_X;
    }
}

void bird_game_tick(BirdGame_t* game) {
    if(game->game_over) return;

    game->velocity_q8 = (int16_t)(game->velocity_q8 + GRAVITY_Q8);
    if(game->velocity_q8 > MAX_FALL_Q8) game->velocity_q8 = MAX_FALL_Q8;
    game->bird_y_q8 = (int16_t)(game->bird_y_q8 + game->velocity_q8);

    if(game->bird_y_q8 < 0 || game->bird_y_q8 > (BIRD_ROWS - 1) * BIRD_Q8) {
        game->game_over = true;
    }

    uint8_t speed = BARRIER_SPEED_BASE + game->score / 5u;
    if(speed > BARRIER_SPEED_MAX) speed = BARRIER_SPEED_MAX;

    MoveBarrier(game, &game->barrier_x, &game->gap_row, &game->gap_height, &game->passed_barrier, speed);
    MoveBarrier(game, &game->barrier2_x, &game->gap2_row, &game->gap2_height, &game->passed_barrier2, speed);

    uint8_t row = bird_game_bird_row(game);
    if(PointInBarrier(game->barrier_x, game->gap_row, game->gap_height, BIRD_X, row) ||
       PointInBarrier(game->barrier2_x, game->gap2_row, game->gap2_height, BIRD_X, row)) {
        game->game_over = true;
    }
}
