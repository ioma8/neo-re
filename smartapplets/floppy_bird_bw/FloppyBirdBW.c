#include "../betawise-sdk/applet.h"
#include "draw_text.h"
#include "game.h"
#include "os3k.h"

enum {
    TICK_MS = 55,
    BARRIER_W = 8,
    GAP_H = 20
};

typedef struct {
    BirdGame_t game;
    uint32_t last_tick_ms;
    uint8_t initialized;
} FloppyState_t;

APPLET_ENTRY(alpha_neo_process_message);
APPLET_STATE(FloppyState_t);

static void FillSafe(int16_t x, int16_t y, int16_t w, int16_t h, RopCode_e rop) {
    if(x < 0) {
        w = (int16_t)(w + x);
        x = 0;
    }
    if(y < 0) {
        h = (int16_t)(h + y);
        y = 0;
    }
    if(x + w > BIRD_COLS) w = (int16_t)(BIRD_COLS - x);
    if(y + h > BIRD_ROWS) h = (int16_t)(BIRD_ROWS - y);
    if(w > 0 && h > 0) RasterOp((uint16_t)x, (uint16_t)y, (uint16_t)w, (uint16_t)h, 0, rop);
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
    FillSafe(BIRD_X - 4, (int16_t)y - 3, 8, 7, rop);
    FillSafe(BIRD_X + 4, (int16_t)y - 1, 3, 3, rop);
    if(rop == ROP_BLACKNESS) FillSafe(BIRD_X + 1, (int16_t)y - 2, 1, 1, ROP_WHITENESS);
}

static void DrawBarrierAt(int16_t x, uint8_t gap_row, RopCode_e rop) {
    FillSafe(x, 0, BARRIER_W, gap_row, rop);
    FillSafe(
        x,
        (int16_t)(gap_row + GAP_H),
        BARRIER_W,
        (int16_t)(BIRD_ROWS - gap_row - GAP_H),
        rop
    );
}

static void DrawBarrier(const BirdGame_t* game, RopCode_e rop) {
    DrawBarrierAt(game->barrier_x, game->gap_row, rop);
}

static void ClearBarrierTrailingEdge(int16_t old_x, int16_t new_x, uint8_t gap_row) {
    int16_t clear_x = (int16_t)(new_x + BARRIER_W);
    int16_t clear_w = (int16_t)(old_x + BARRIER_W - clear_x);
    if(clear_w <= 0) return;
    FillSafe(clear_x, 0, clear_w, gap_row, ROP_WHITENESS);
    FillSafe(clear_x, (int16_t)(gap_row + GAP_H), clear_w, (int16_t)(BIRD_ROWS - gap_row - GAP_H), ROP_WHITENESS);
}

static void DrawGame(const FloppyState_t* state) {
    FillSafe(0, 0, BIRD_COLS, BIRD_ROWS, ROP_WHITENESS);
    DrawBarrier(&state->game, ROP_BLACKNESS);
    DrawBird(bird_game_bird_row(&state->game), ROP_BLACKNESS);
    DrawScore(state->game.score);
}

static void DrawGameOver(const BirdGame_t* game) {
    FillSafe(0, 0, BIRD_COLS, BIRD_ROWS, ROP_WHITENESS);
    bird_draw_text("GAME OVER", 60, 8, 4);
    bird_draw_text("SCORE", 94, 34, 2);
    bird_draw_char((char)('0' + (game->score / 10u) % 10u), 146, 34, 2);
    bird_draw_char((char)('0' + game->score % 10u), 154, 34, 2);
    bird_draw_text("R FOR RESTART", 58, 50, 1);
    bird_draw_text("ESC FOR CLOSE", 158, 50, 1);
}

static void DrawStep(const BirdGame_t* game, uint8_t old_y, int16_t old_barrier_x, uint8_t old_gap, uint8_t old_score) {
    if(game->game_over) {
        DrawGameOver(game);
        return;
    }
    DrawBird(old_y, ROP_WHITENESS);
    if(game->barrier_x > old_barrier_x || old_gap != game->gap_row) {
        DrawBarrierAt(old_barrier_x, old_gap, ROP_WHITENESS);
    } else {
        ClearBarrierTrailingEdge(old_barrier_x, game->barrier_x, old_gap);
    }
    DrawBarrier(game, ROP_BLACKNESS);
    DrawBird(bird_game_bird_row(game), ROP_BLACKNESS);
    if(game->score != old_score || TouchesScore(old_barrier_x, old_gap) || TouchesScore(game->barrier_x, game->gap_row)) {
        DrawScore(game->score);
    }
}

static void Reset(FloppyState_t* state) {
    bird_game_init(&state->game);
    state->last_tick_ms = GetUptimeMilliseconds();
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
    state->last_tick_ms = GetUptimeMilliseconds();
    DrawGame(state);
}

static void TickIfDue(FloppyState_t* state) {
    uint32_t now = GetUptimeMilliseconds();
    if(now - state->last_tick_ms < TICK_MS) return;
    state->last_tick_ms = now;
    if(state->game.game_over) return;

    uint8_t old_y = bird_game_bird_row(&state->game);
    int16_t old_barrier_x = state->game.barrier_x;
    uint8_t old_gap = state->game.gap_row;
    uint8_t old_score = state->game.score;
    bird_game_tick(&state->game);
    DrawStep(&state->game, old_y, old_barrier_x, old_gap, old_score);
}

void alpha_neo_process_message(uint32_t message, uint32_t param, uint32_t* status) {
    FloppyState_t* state = State();
    *status = 0;
    switch(message) {
        case MSG_SETFOCUS:
            Reset(state);
            break;
        case MSG_CHAR:
            if((param & 0xff) == ' ') Flap(state);
            else if((param & 0xff) == 'r' || (param & 0xff) == 'R') Restart(state);
            break;
        case MSG_KEY:
            if((param & 0xff) == KEY_SPACE) {
                Flap(state);
            } else if((param & 0xff) == KEY_R) {
                Restart(state);
            } else if((param & 0xff) == KEY_ESC || (param & 0xff) == KEY_APPLETS) {
                *status = APPLET_EXIT_STATUS;
            }
            break;
        case MSG_IDLE:
            if(state->initialized == 0) Reset(state);
            TickIfDue(state);
            break;
        default:
            *status = APPLET_UNHANDLED_STATUS;
            break;
    }
}
