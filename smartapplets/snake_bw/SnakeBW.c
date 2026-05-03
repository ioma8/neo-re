#include "../betawise-sdk/applet.h"
#include "game.h"
#include "os3k.h"

enum {
    CELL_W = 4,
    CELL_H = 4,
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

static void Fill(uint16_t x, uint16_t y, uint16_t w, uint16_t h, RopCode_e rop) {
    RasterOp(x, y, w, h, 0, rop);
}

static void DrawCell(uint8_t x, uint8_t y, RopCode_e rop) {
    Fill((uint16_t)x * CELL_W, (uint16_t)y * CELL_H, CELL_W, CELL_H, rop);
}

static void DrawDigit(uint8_t digit, uint16_t x, uint16_t y) {
    static const uint8_t digits[10][5] = {
        {7, 5, 5, 5, 7}, {2, 6, 2, 2, 7}, {7, 1, 7, 4, 7}, {7, 1, 7, 1, 7}, {5, 5, 7, 1, 1},
        {7, 4, 7, 1, 7}, {7, 4, 7, 5, 7}, {7, 1, 1, 1, 1}, {7, 5, 7, 5, 7}, {7, 5, 7, 1, 7}
    };
    for(uint8_t row = 0; row < 5; row++) {
        for(uint8_t col = 0; col < 3; col++) {
            if((digits[digit][row] & (1u << (2u - col))) != 0) {
                Fill((uint16_t)(x + col * 2), (uint16_t)(y + row * 2), 2, 2, ROP_BLACKNESS);
            }
        }
    }
}

static void DrawScore(uint8_t score) {
    DrawDigit((uint8_t)((score / 10u) % 10u), 2, 2);
    DrawDigit((uint8_t)(score % 10u), 10, 2);
}

static void DrawPauseMark(void) {
    Fill(124, 22, 5, 20, ROP_BLACKNESS);
    Fill(136, 22, 5, 20, ROP_BLACKNESS);
}

static void DrawGameOverMark(void) {
    for(uint8_t i = 0; i < 28; i++) {
        Fill((uint16_t)(118 + i), (uint16_t)(18 + i), 3, 3, ROP_BLACKNESS);
        Fill((uint16_t)(146 - i), (uint16_t)(18 + i), 3, 3, ROP_BLACKNESS);
    }
}

static void Draw(const SnakeState_t* state) {
    Fill(0, 0, 264, 64, ROP_WHITENESS);
    DrawCell(state->game.food_x, state->game.food_y, ROP_BLACKNESS);
    for(uint8_t i = 0; i < state->game.length; i++) {
        DrawCell(state->game.body_x[i], state->game.body_y[i], ROP_BLACKNESS);
    }
    if(state->game.paused) DrawPauseMark();
    if(state->game.game_over) {
        DrawGameOverMark();
        DrawScore(state->game.score);
    }
}

static void Reset(SnakeState_t* state) {
    snake_game_init(&state->game);
    state->last_tick_ms = GetUptimeMilliseconds();
    state->initialized = 1;
    SetCursorMode(CURSOR_MODE_HIDE);
    Draw(state);
}

static void TickIfDue(SnakeState_t* state) {
    uint32_t now = GetUptimeMilliseconds();
    if(now - state->last_tick_ms < TickMs(&state->game)) return;
    state->last_tick_ms = now;
    snake_game_tick(&state->game);
    Draw(state);
}

static void HandleKey(SnakeState_t* state, uint32_t key, uint32_t* status) {
    switch(key & 0xffu) {
        case KEY_UP: snake_game_turn(&state->game, SNAKE_UP); break;
        case KEY_RIGHT: snake_game_turn(&state->game, SNAKE_RIGHT); break;
        case KEY_DOWN: snake_game_turn(&state->game, SNAKE_DOWN); break;
        case KEY_LEFT: snake_game_turn(&state->game, SNAKE_LEFT); break;
        case KEY_P: snake_game_toggle_pause(&state->game); break;
        case KEY_R: snake_game_restart(&state->game); break;
        case KEY_ESC:
        case KEY_APPLETS:
            *status = APPLET_EXIT_STATUS;
            return;
        default: return;
    }
    Draw(state);
}

static void HandleChar(SnakeState_t* state, uint32_t param) {
    uint8_t byte = (uint8_t)(param & 0xffu);
    if(byte == 'p' || byte == 'P') snake_game_toggle_pause(&state->game);
    else if(byte == 'r' || byte == 'R') snake_game_restart(&state->game);
    else return;
    Draw(state);
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
