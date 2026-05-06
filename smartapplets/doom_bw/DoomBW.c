#include "../betawise-sdk/applet.h"
#include "game.h"
#include "os3k.h"
#include "render.h"

typedef struct {
    RayGame_t game;
    uint8_t initialized;
} RayState_t;

APPLET_ENTRY(alpha_neo_process_message);
APPLET_STATE(RayState_t);

static void Reset(RayState_t* state) {
    ray_game_init(&state->game);
    state->initialized = 1;
    SetCursorMode(CURSOR_MODE_HIDE);
    ray_render_frame(&state->game);
}

static void Redraw(RayState_t* state) {
    ray_render_frame(&state->game);
}

static void HandleKey(RayState_t* state, uint32_t key, uint32_t* status) {
    switch(key & 0xffu) {
        case KEY_UP:
            ray_game_move(&state->game, RAY_MOVE_FORWARD);
            break;
        case KEY_DOWN:
            ray_game_move(&state->game, RAY_MOVE_BACKWARD);
            break;
        case KEY_LEFT:
            ray_game_turn_left(&state->game);
            break;
        case KEY_RIGHT:
            ray_game_turn_right(&state->game);
            break;
        case KEY_SPACE:
            ray_game_shoot(&state->game);
            break;
        case KEY_R:
            ray_game_init(&state->game);
            break;
        case KEY_ESC:
        case KEY_APPLETS:
            *status = APPLET_EXIT_STATUS;
            return;
        default:
            return;
    }
    Redraw(state);
}

static void HandleChar(RayState_t* state, uint32_t param) {
    uint8_t c = (uint8_t)(param & 0xffu);
    if(c == 'r' || c == 'R') {
        ray_game_init(&state->game);
        Redraw(state);
    }
}

void alpha_neo_process_message(uint32_t message, uint32_t param, uint32_t* status) {
    RayState_t* state = State();
    *status = 0;
    switch(message) {
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
            break;
        default:
            *status = APPLET_UNHANDLED_STATUS;
            break;
    }
}
