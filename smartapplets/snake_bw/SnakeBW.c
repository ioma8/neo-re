#include "../betawise-sdk/applet.h"
#include "draw.h"
#include "game.h"
#include "os3k.h"

enum {
    TICK_BASE_MS = 145,
    TICK_MIN_MS = 75
};

typedef struct {
    SnakeGame_t game;
    uint32_t last_tick_ms;
    uint8_t initialized;
} SnakeState_t;

APPLET_ENTRY(alpha_neo_process_message);
APPLET_STATE(SnakeState_t);

static uint32_t TickMs(const SnakeGame_t* game) {
    uint32_t speedup = (uint32_t)(game->score / 4u) * 8u;
    if(speedup >= TICK_BASE_MS - TICK_MIN_MS) return TICK_MIN_MS;
    return TICK_BASE_MS - speedup;
}

static uint8_t NewSeed(void) {
    uint32_t now = GetUptimeMilliseconds();
    uint8_t seed = (uint8_t)(now ^ (now >> 8) ^ (now >> 16) ^ (now >> 24));
    return seed == 0 ? 31 : seed;
}

static void Reset(SnakeState_t* state) {
    snake_game_init_seeded(&state->game, NewSeed());
    state->last_tick_ms = GetUptimeMilliseconds();
    state->initialized = 1;
    SetCursorMode(CURSOR_MODE_HIDE);
    snake_draw_full(&state->game);
}

static void TickIfDue(SnakeState_t* state) {
    uint32_t now = GetUptimeMilliseconds();
    if(now - state->last_tick_ms < TickMs(&state->game)) return;
    state->last_tick_ms = now;
    if(state->game.paused || state->game.game_over) return;

    uint8_t old_tail = (uint8_t)(state->game.length - 1);
    uint8_t old_tail_x = state->game.body_x[old_tail];
    uint8_t old_tail_y = state->game.body_y[old_tail];
    uint8_t old_food_x = state->game.food_x;
    uint8_t old_food_y = state->game.food_y;
    uint8_t old_length = state->game.length;
    snake_game_tick(&state->game);
    snake_draw_step(&state->game, old_tail_x, old_tail_y, old_food_x, old_food_y, old_length);
}

static void HandleKey(SnakeState_t* state, uint32_t key, uint32_t* status) {
    switch(key & 0xffu) {
        case KEY_UP: snake_game_turn(&state->game, SNAKE_UP); return;
        case KEY_RIGHT: snake_game_turn(&state->game, SNAKE_RIGHT); return;
        case KEY_DOWN: snake_game_turn(&state->game, SNAKE_DOWN); return;
        case KEY_LEFT: snake_game_turn(&state->game, SNAKE_LEFT); return;
        case KEY_P:
            snake_game_toggle_pause(&state->game);
            break;
        case KEY_R:
            snake_game_restart_seeded(&state->game, NewSeed());
            state->last_tick_ms = GetUptimeMilliseconds();
            break;
        case KEY_ESC:
        case KEY_APPLETS:
            *status = APPLET_EXIT_STATUS;
            return;
        default: return;
    }
    snake_draw_full(&state->game);
}

static void HandleChar(SnakeState_t* state, uint32_t param) {
    uint8_t byte = (uint8_t)(param & 0xffu);
    if(byte == 'p' || byte == 'P') snake_game_toggle_pause(&state->game);
    else if(byte == 'r' || byte == 'R') {
        snake_game_restart_seeded(&state->game, NewSeed());
        state->last_tick_ms = GetUptimeMilliseconds();
    }
    else return;
    snake_draw_full(&state->game);
}

void alpha_neo_process_message(uint32_t message, uint32_t param, uint32_t* status) {
    SnakeState_t* state = State();
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
            TickIfDue(state);
            break;
        default:
            *status = APPLET_UNHANDLED_STATUS;
            break;
    }
}
