#include "../betawise-sdk/bw.h"
#include "draw_text.h"
#include "game.h"

enum {
    TICK_MS = 55,
    BARRIER_W = 8
};

typedef struct {
    BirdGame_t game;
    BwTimer tick_timer;
    uint8_t initialized;
} FloppyState_t;

BW_APP_ENTRY(alpha_neo_process_message);
BW_APP_STATE(FloppyState_t);

static void FillSafe(int16_t x, int16_t y, int16_t w, int16_t h, RopCode_e rop) {
    BwRect rect = {x, y, w, h};
    bw_fill_rect_clipped(rect, BIRD_COLS, BIRD_ROWS, rop);
}

static void DrawScore(uint8_t score) {
    FillSafe(0, 0, 24, 12, ROP_WHITENESS);
    bird_draw_digit((uint8_t)((score / 10u) % 10u), 2, 2);
    bird_draw_digit((uint8_t)(score % 10u), 12, 2);
}

static bool TouchesScore(int16_t x, uint8_t gap_row) {
    return x < 24 && x + BARRIER_W > 0 && gap_row > 0;
}

static void DrawBird(uint8_t y, RopCode_e rop) {
    int16_t by = (int16_t)y;
    /* Body - rounder shape */
    FillSafe(BIRD_X - 4, by - 3, 3, 1, rop);   /* top of body */
    FillSafe(BIRD_X - 5, by - 2, 7, 5, rop);   /* main body */
    FillSafe(BIRD_X - 4, by + 3, 5, 1, rop);   /* bottom */
    /* Wing */
    FillSafe(BIRD_X - 2, by - 1, 4, 1, rop);   /* wing top */
    FillSafe(BIRD_X - 1, by, 3, 2, rop);       /* wing body */
    /* Beak */
    FillSafe(BIRD_X + 4, by, 2, 2, rop);       /* beak */
    /* Tail */
    FillSafe(BIRD_X - 7, by - 1, 2, 3, rop);   /* tail */
    if(rop == ROP_BLACKNESS) {
        /* Eye */
        FillSafe(BIRD_X + 1, by - 2, 2, 2, ROP_WHITENESS);
        FillSafe(BIRD_X + 1, by - 2, 1, 1, ROP_BLACKNESS);
    }
}

static void DrawBarrierAt(int16_t x, uint8_t gap_row, int16_t gap_h, RopCode_e rop) {
    int16_t cap_h = 3;
    /* Top barrier */
    FillSafe(x, 0, BARRIER_W, gap_row, rop);
    /* Top cap (wider piece at the gap opening) */
    FillSafe(x - 1, (int16_t)(gap_row - cap_h), BARRIER_W + 2, cap_h, rop);
    /* Bottom barrier */
    FillSafe(x, (int16_t)(gap_row + gap_h), BARRIER_W, (int16_t)(BIRD_ROWS - gap_row - gap_h), rop);
    /* Bottom cap */
    FillSafe(x - 1, (int16_t)(gap_row + gap_h), BARRIER_W + 2, cap_h, rop);
}

static void DrawBarrierLeadingEdge(int16_t old_x, int16_t new_x, uint8_t gap_row, int16_t gap_h) {
    int16_t w = (int16_t)(old_x - new_x);
    if(w <= 0) return;
    int16_t cap_h = 3;
    FillSafe(new_x, 0, w, gap_row, ROP_BLACKNESS);
    FillSafe((int16_t)(new_x - 1), (int16_t)(gap_row - cap_h), w, cap_h, ROP_BLACKNESS);
    FillSafe(new_x, (int16_t)(gap_row + gap_h), w, (int16_t)(BIRD_ROWS - gap_row - gap_h), ROP_BLACKNESS);
    FillSafe((int16_t)(new_x - 1), (int16_t)(gap_row + gap_h), w, cap_h, ROP_BLACKNESS);
}

static void DrawBarrier(const BirdGame_t* game, RopCode_e rop) {
    DrawBarrierAt(game->barrier_x, game->gap_row, (int16_t)game->gap_height, rop);
    DrawBarrierAt(game->barrier2_x, game->gap2_row, (int16_t)game->gap2_height, rop);
}

static void ClearBarrierTrailingEdge(int16_t old_x, int16_t new_x, uint8_t gap_row, int16_t gap_h) {
    int16_t cap_h = 3;
    int16_t clear_w = (int16_t)(old_x - new_x);
    if(clear_w <= 0) return;
    int16_t body_x = (int16_t)(new_x + BARRIER_W);
    int16_t cap_x = (int16_t)(body_x + 1);
    FillSafe(body_x, 0, clear_w, gap_row, ROP_WHITENESS);
    FillSafe(cap_x, (int16_t)(gap_row - cap_h), clear_w, cap_h, ROP_WHITENESS);
    FillSafe(body_x, (int16_t)(gap_row + gap_h), clear_w, (int16_t)(BIRD_ROWS - gap_row - gap_h), ROP_WHITENESS);
    FillSafe(cap_x, (int16_t)(gap_row + gap_h), clear_w, cap_h, ROP_WHITENESS);
}

static void DrawGround(void) {
    /* Ground line at row 60-63 */
    FillSafe(0, 60, BIRD_COLS, 1, ROP_BLACKNESS);
    FillSafe(0, 61, BIRD_COLS, 1, ROP_BLACKNESS);
    /* Dotted pattern on ground */
    for(uint16_t gx = 0; gx < (uint16_t)BIRD_COLS; gx = (uint16_t)(gx + 6)) {
        FillSafe((int16_t)gx, 62, 2, 2, ROP_BLACKNESS);
    }
}

static void DrawGroundRange(int16_t x, int16_t w) {
    if(x < 0) {
        w = (int16_t)(w + x);
        x = 0;
    }
    if(x + w > BIRD_COLS) w = (int16_t)(BIRD_COLS - x);
    if(w <= 0) return;

    FillSafe(x, 60, w, 1, ROP_BLACKNESS);
    FillSafe(x, 61, w, 1, ROP_BLACKNESS);
    int16_t dot_x = (int16_t)((x / 6) * 6);
    if(dot_x < x) dot_x = (int16_t)(dot_x + 6);
    for(; dot_x < x + w; dot_x = (int16_t)(dot_x + 6)) {
        FillSafe(dot_x, 62, 2, 2, ROP_BLACKNESS);
    }
}

static void DrawGroundMovedBarrierRange(int16_t old_x, int16_t new_x) {
    int16_t left = old_x < new_x ? old_x : new_x;
    int16_t right = old_x > new_x ? old_x : new_x;
    DrawGroundRange((int16_t)(left - 1), (int16_t)(right - left + BARRIER_W + 6));
}

static void DrawGame(const FloppyState_t* state) {
    FillSafe(0, 0, BIRD_COLS, BIRD_ROWS, ROP_WHITENESS);
    DrawBarrier(&state->game, ROP_BLACKNESS);
    DrawBird(bird_game_bird_row(&state->game), ROP_BLACKNESS);
    DrawScore(state->game.score);
    DrawGround();
}

static void DrawGameOver(const BirdGame_t* game) {
    FillSafe(0, 0, BIRD_COLS, BIRD_ROWS, ROP_WHITENESS);
    DrawGround();
    FillSafe(34, 4, 196, 50, ROP_BLACKNESS);
    FillSafe(37, 7, 190, 44, ROP_WHITENESS);
    FillSafe(42, 12, 180, 1, ROP_BLACKNESS);
    FillSafe(42, 34, 180, 1, ROP_BLACKNESS);
    bird_draw_text("GAME OVER", 60, 15, 4);
    bird_draw_text("SCORE", 82, 39, 2);
    uint8_t tens = (uint8_t)((game->score / 10u) % 10u);
    uint8_t ones = (uint8_t)(game->score % 10u);
    bird_draw_digit(tens, 145, 38);
    bird_draw_digit(ones, 155, 38);
    bird_draw_text("R RESTART", 50, 54, 1);
    bird_draw_text("ESC EXIT", 158, 54, 1);
}

static void DrawStep(const BirdGame_t* game, uint8_t old_y, int16_t old_barrier_x, uint8_t old_gap, uint8_t old_score, int16_t old_gap_h, int16_t old_barrier2_x, uint8_t old_gap2, int16_t old_gap2_h) {
    if(game->game_over) {
        DrawGameOver(game);
        return;
    }
    DrawBird(old_y, ROP_WHITENESS);
    /* Full redraw approach: erase and redraw both barriers */
    int16_t gap_h = (int16_t)game->gap_height;
    if(game->barrier_x > old_barrier_x || old_gap != game->gap_row || gap_h != old_gap_h) {
        DrawBarrierAt(old_barrier_x, old_gap, old_gap_h, ROP_WHITENESS);
        DrawBarrierAt(game->barrier_x, game->gap_row, gap_h, ROP_BLACKNESS);
    } else {
        ClearBarrierTrailingEdge(old_barrier_x, game->barrier_x, old_gap, gap_h);
        DrawBarrierLeadingEdge(old_barrier_x, game->barrier_x, game->gap_row, gap_h);
    }
    int16_t gap2_h = (int16_t)game->gap2_height;
    if(game->barrier2_x > old_barrier2_x || old_gap2 != game->gap2_row || gap2_h != old_gap2_h) {
        DrawBarrierAt(old_barrier2_x, old_gap2, old_gap2_h, ROP_WHITENESS);
        DrawBarrierAt(game->barrier2_x, game->gap2_row, gap2_h, ROP_BLACKNESS);
    } else {
        ClearBarrierTrailingEdge(old_barrier2_x, game->barrier2_x, old_gap2, gap2_h);
        DrawBarrierLeadingEdge(old_barrier2_x, game->barrier2_x, game->gap2_row, gap2_h);
    }
    DrawBird(bird_game_bird_row(game), ROP_BLACKNESS);
    if(game->score != old_score ||
       TouchesScore(old_barrier_x, old_gap) ||
       TouchesScore(game->barrier_x, game->gap_row) ||
       TouchesScore(old_barrier2_x, old_gap2) ||
       TouchesScore(game->barrier2_x, game->gap2_row)) {
        DrawScore(game->score);
    }
    DrawGroundMovedBarrierRange(old_barrier_x, game->barrier_x);
    DrawGroundMovedBarrierRange(old_barrier2_x, game->barrier2_x);
    DrawGroundRange((int16_t)(BIRD_X - 8), 16);
}

static void Reset(FloppyState_t* state) {
    bird_game_init(&state->game);
    bw_timer_reset(&state->tick_timer, TICK_MS);
    state->initialized = 1;
    DrawGame(state);
}

static void Flap(FloppyState_t* state) {
    bool was_over = state->game.game_over;
    bird_game_flap(&state->game);
    if(was_over) DrawGame(state);
}

static void Restart(FloppyState_t* state) {
    bird_game_init(&state->game);
    bw_timer_reset(&state->tick_timer, TICK_MS);
    DrawGame(state);
}

static void TickIfDue(FloppyState_t* state) {
    if(!bw_timer_due_and_advance(&state->tick_timer, TICK_MS)) return;
    if(state->game.game_over) return;

    uint8_t old_y = bird_game_bird_row(&state->game);
    int16_t old_barrier_x = state->game.barrier_x;
    uint8_t old_gap = state->game.gap_row;
    uint8_t old_score = state->game.score;
    int16_t old_gap_h = (int16_t)state->game.gap_height;
    int16_t old_barrier2_x = state->game.barrier2_x;
    uint8_t old_gap2 = state->game.gap2_row;
    int16_t old_gap2_h = (int16_t)state->game.gap2_height;
    bird_game_tick(&state->game);
    DrawStep(&state->game, old_y, old_barrier_x, old_gap, old_score, old_gap_h, old_barrier2_x, old_gap2, old_gap2_h);
}

void alpha_neo_process_message(uint32_t message, uint32_t param, uint32_t* status) {
    FloppyState_t* state = State();
    uint32_t base_message = bw_message_id(message);
    *status = 0;
    switch(base_message) {
        case MSG_SETFOCUS:
            Reset(state);
            break;
        case MSG_CHAR:
            if(bw_key_id(param) == ' ') Flap(state);
            else if(bw_key_id(param) == 'r' || bw_key_id(param) == 'R') Restart(state);
            else *status = BW_STATUS_UNHANDLED;
            break;
        case MSG_KEY:
            if(bw_key_id(param) == KEY_SPACE) {
                Flap(state);
            } else if(bw_key_id(param) == KEY_R) {
                Restart(state);
            } else if(bw_key_id(param) == KEY_ESC || bw_key_id(param) == KEY_APPLETS) {
                *status = BW_STATUS_EXIT;
            } else {
                *status = BW_STATUS_UNHANDLED;
            }
            break;
        case MSG_IDLE:
            if(state->initialized == 0) Reset(state);
            TickIfDue(state);
            break;
        default:
            *status = BW_STATUS_UNHANDLED;
            break;
    }
}
