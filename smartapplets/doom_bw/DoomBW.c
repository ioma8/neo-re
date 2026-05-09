#include "../betawise-sdk/bw.h"
#include "game.h"
#include "render.h"

typedef struct {
    RayGame_t game;
    BwTimer step_timer;
    int8_t pending_move;
    int8_t pending_turn;
    uint8_t initialized;
} RayState_t;

enum {
    RAW_KEY_SPACE = 0x79,
    STEP_MS = 70
};

BW_APP_ENTRY(alpha_neo_process_message);
BW_APP_STATE(RayState_t);

static void Reset(RayState_t* state) {
    ray_game_init(&state->game);
    bw_timer_reset(&state->step_timer, STEP_MS);
    state->pending_move = 0;
    state->pending_turn = 0;
    state->initialized = 1;
    SetCursorMode(CURSOR_MODE_HIDE);
    ray_render_frame(&state->game);
}

static void Redraw(RayState_t* state) {
    ray_render_frame(&state->game);
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
    if(!bw_timer_due_and_advance(&state->step_timer, STEP_MS)) return;

    if(state->pending_turn < 0) {
        ray_game_turn_left(&state->game);
    } else if(state->pending_turn > 0) {
        ray_game_turn_right(&state->game);
    } else if(state->pending_move < 0) {
        ray_game_move(&state->game, RAY_MOVE_BACKWARD);
    } else if(state->pending_move > 0) {
        ray_game_move(&state->game, RAY_MOVE_FORWARD);
    } else {
        return;
    }

    state->pending_move = 0;
    state->pending_turn = 0;
    Redraw(state);
}

static void HandleKey(RayState_t* state, uint32_t key, uint32_t* status) {
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
            break;
        case KEY_R:
            ray_game_init(&state->game);
            bw_timer_reset(&state->step_timer, STEP_MS);
            state->pending_move = 0;
            state->pending_turn = 0;
            break;
        case KEY_ESC:
        case KEY_APPLETS:
            *status = BW_STATUS_EXIT;
            return;
        default:
            return;
    }
    Redraw(state);
}

static void HandleChar(RayState_t* state, uint32_t param) {
    uint8_t c = (uint8_t)bw_key_id(param);
    if(c == ' ') {
        ray_game_shoot(&state->game);
        Redraw(state);
    } else if(c == 'r' || c == 'R') {
        ray_game_init(&state->game);
        Redraw(state);
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
