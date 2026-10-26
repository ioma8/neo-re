#include "../betawise-sdk/bw.h"
#include "game.h"
#include "render.h"

#include <stddef.h>

enum {
    DOOM_FB_BYTES = RAY_VIEW_W * (RAY_VIEW_H / 8),
    RAW_KEY_SPACE = 0x79,
    STEP_MS = 90
};

typedef struct {
    RayGame_t game;
    BwTimer step_timer;
    int8_t pending_move;
    int8_t pending_turn;
    uint8_t initialized;
    uint8_t framebuffer[DOOM_FB_BYTES];
    uint16_t depth[RAY_VIEW_W];
    int16_t ray_x[RAY_COLS];
    int16_t ray_y[RAY_COLS];
} RayState_t;

_Static_assert(sizeof(RayGame_t) == 18, "RayGame_t layout changed");
_Static_assert(offsetof(RayState_t, step_timer) == 18, "timer offset changed");
_Static_assert(offsetof(RayState_t, framebuffer) == 25, "framebuffer offset changed");

BW_APP_ENTRY(alpha_neo_process_message);
BW_APP_STATE(RayState_t);

static void Render(RayState_t* state) {
    ray_render_frame(&state->game, state->framebuffer, state->depth, state->ray_x, state->ray_y);
}

static void Reset(RayState_t* state) {
    ray_game_init(&state->game);
    bw_timer_reset(&state->step_timer, STEP_MS);
    state->pending_move = 0;
    state->pending_turn = 0;
    state->initialized = 1;
    SetCursorMode(CURSOR_MODE_HIDE);
    Render(state);
}

static void QueueMove(RayState_t* state, RayMove_t move) {
    state->pending_move = (int8_t)move;
    state->pending_turn = 0;
}

static void QueueTurn(RayState_t* state, int8_t turn) {
    state->pending_turn = turn;
    state->pending_move = 0;
}

static void TickIfDue(RayState_t* state) {
    uint32_t now = bw_now_ms();
    bool redraw = false;
    if(!bw_time_due(now, state->step_timer.next_ms)) return;

    if(state->pending_turn < 0) {
        ray_game_turn_left(&state->game);
        state->pending_turn = 0;
        state->pending_move = 0;
        redraw = true;
    } else if(state->pending_turn > 0) {
        ray_game_turn_right(&state->game);
        state->pending_turn = 0;
        state->pending_move = 0;
        redraw = true;
    } else if(state->pending_move < 0) {
        ray_game_move(&state->game, RAY_MOVE_BACKWARD);
        state->pending_move = 0;
        redraw = true;
    } else if(state->pending_move > 0) {
        ray_game_move(&state->game, RAY_MOVE_FORWARD);
        state->pending_move = 0;
        redraw = true;
    }
    if(redraw || state->game.muzzle_flash != 0) {
        ray_game_tick(&state->game);
        bw_timer_reset(&state->step_timer, STEP_MS);
        Render(state);
    }
}

static void HandleKey(RayState_t* state, uint32_t key, uint32_t* status) {
    if(state->initialized == 0) Reset(state);
    switch(bw_key_id(key)) {
        case KEY_UP:
            QueueMove(state, RAY_MOVE_FORWARD);
            return;
        case KEY_DOWN:
            QueueMove(state, RAY_MOVE_BACKWARD);
            return;
        case KEY_LEFT:
            QueueTurn(state, -1);
            return;
        case KEY_RIGHT:
            QueueTurn(state, 1);
            return;
        case KEY_SPACE:
        case RAW_KEY_SPACE:
            ray_game_shoot(&state->game);
            Render(state);
            return;
        case KEY_R:
            Reset(state);
            return;
        case KEY_ESC:
        case KEY_APPLETS:
            *status = BW_STATUS_EXIT;
            return;
        default:
            return;
    }
}

static void HandleChar(RayState_t* state, uint32_t param) {
    uint8_t c = (uint8_t)bw_key_id(param);
    if(state->initialized == 0) Reset(state);
    if(c == ' ') {
        ray_game_shoot(&state->game);
        Render(state);
    } else if(c == 'r' || c == 'R') {
        Reset(state);
    } else if(c == 'w' || c == 'W') {
        QueueMove(state, RAY_MOVE_FORWARD);
    } else if(c == 's' || c == 'S') {
        QueueMove(state, RAY_MOVE_BACKWARD);
    } else if(c == 'a' || c == 'A') {
        QueueTurn(state, -1);
    } else if(c == 'd' || c == 'D') {
        QueueTurn(state, 1);
    }
}

void alpha_neo_process_message(uint32_t message, uint32_t param, uint32_t* status) {
    RayState_t* state = State();
    uint32_t base_message = bw_message_id(message);
    *status = 0;
    switch(base_message) {
        case MSG_SETFOCUS:
            Reset(state);
            break;
        case MSG_KEY:
            HandleKey(state, param, status);
            break;
        case MSG_CHAR:
            HandleChar(state, param);
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
