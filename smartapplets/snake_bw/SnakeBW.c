#include "../betawise-sdk/bw.h"
#include "draw.h"
#include "game.h"

enum {
    TICK_BASE_MS = 90,
    TICK_MIN_MS = 50
};

typedef struct {
    SnakeGame_t game;
    BwTimer tick_timer;
    uint8_t initialized;
} SnakeState_t;

BW_APP_ENTRY(alpha_neo_process_message);
BW_APP_STATE(SnakeState_t);

static uint32_t TickMs(const SnakeGame_t* game) {
    uint32_t speedup = (uint32_t)(game->score / 4u) * 8u;
    if(speedup >= TICK_BASE_MS - TICK_MIN_MS) return TICK_MIN_MS;
    return TICK_BASE_MS - speedup;
}

static uint8_t NewSeed(void) {
    uint32_t now = bw_now_ms();
    uint8_t seed = (uint8_t)(now ^ (now >> 8) ^ (now >> 16) ^ (now >> 24));
    return seed == 0 ? 31 : seed;
}

static void ScheduleNextTick(SnakeState_t* state) {
    bw_timer_reset(&state->tick_timer, TickMs(&state->game));
}

static void Reset(SnakeState_t* state) {
    snake_game_init_seeded(&state->game, NewSeed());
    state->initialized = 1;
    ScheduleNextTick(state);
    SetCursorMode(CURSOR_MODE_HIDE);
    snake_draw_full(&state->game);
}

static void TickIfDue(SnakeState_t* state) {
    if(!bw_timer_due_and_advance(&state->tick_timer, TickMs(&state->game))) return;
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

static bool HandleDirection(SnakeState_t* state, SnakeDirection_t dir) {
    if(state->game.direction == dir) return true; /* same direction: handled but no movement */
    if(snake_game_turn(&state->game, dir)) {
        return true;
    }
    return false;
}

static void HandleKey(SnakeState_t* state, uint32_t key, uint32_t* status) {
    switch(bw_key_id(key)) {
        case KEY_UP: if(!HandleDirection(state, SNAKE_UP)) *status = BW_STATUS_UNHANDLED; return;
        case KEY_RIGHT: if(!HandleDirection(state, SNAKE_RIGHT)) *status = BW_STATUS_UNHANDLED; return;
        case KEY_DOWN: if(!HandleDirection(state, SNAKE_DOWN)) *status = BW_STATUS_UNHANDLED; return;
        case KEY_LEFT: if(!HandleDirection(state, SNAKE_LEFT)) *status = BW_STATUS_UNHANDLED; return;
        case KEY_P:
            snake_game_toggle_pause(&state->game);
            ScheduleNextTick(state);
            break;
        case KEY_R:
            snake_game_restart_seeded(&state->game, NewSeed());
            ScheduleNextTick(state);
            break;
        case KEY_ESC:
        case KEY_APPLETS:
            *status = BW_STATUS_EXIT;
            return;
        default:
            *status = BW_STATUS_UNHANDLED;
            return;
    }
    snake_draw_full(&state->game);
}

static void HandleChar(SnakeState_t* state, uint32_t param, uint32_t* status) {
    uint8_t byte = (uint8_t)bw_key_id(param);
    /* Arrow keys sometimes arrive as MSG_CHAR on some firmware paths */
    switch(byte) {
        case 'w': case 'W': if(!HandleDirection(state, SNAKE_UP)) *status = BW_STATUS_UNHANDLED; return;
        case 'd': case 'D': if(!HandleDirection(state, SNAKE_RIGHT)) *status = BW_STATUS_UNHANDLED; return;
        case 's': case 'S': if(!HandleDirection(state, SNAKE_DOWN)) *status = BW_STATUS_UNHANDLED; return;
        case 'a': case 'A': if(!HandleDirection(state, SNAKE_LEFT)) *status = BW_STATUS_UNHANDLED; return;
        default: break;
    }
    if(byte == 'p' || byte == 'P') snake_game_toggle_pause(&state->game);
    else if(byte == 'r' || byte == 'R') {
        snake_game_restart_seeded(&state->game, NewSeed());
        ScheduleNextTick(state);
    }
    else {
        *status = BW_STATUS_UNHANDLED;
        return;
    }
    snake_draw_full(&state->game);
}



void alpha_neo_process_message(uint32_t message, uint32_t param, uint32_t* status) {
    SnakeState_t* state = State();
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
            HandleChar(state, param, status);
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
