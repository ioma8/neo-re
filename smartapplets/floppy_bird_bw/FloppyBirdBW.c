#include "../betawise-sdk/applet.h"
#include "game.h"
#include "os3k.h"

enum { TICK_MS = 120 };

typedef struct {
    BirdGame_t game;
    uint32_t last_tick_ms;
    uint8_t initialized;
} FloppyState_t;

void _OS3K_ClearScreen();
void _OS3K_SetCursor(uint8_t row, uint8_t col, CursorMode_e cursor_mode);

APPLET_ENTRY(alpha_neo_process_message);
APPLET_STATE(FloppyState_t);

static void PutLine(uint8_t row, const char* text) {
    char line[BIRD_COLS + 1];
    uint8_t i = 0;
    for(; i < BIRD_COLS && text[i] != '\0'; i++) line[i] = text[i];
    for(; i < BIRD_COLS; i++) line[i] = ' ';
    line[BIRD_COLS] = '\0';
    _OS3K_SetCursor(row, 1, CURSOR_MODE_HIDE);
    PutStringRaw(line);
}

static void DrawGame(const FloppyState_t* state) {
    char line[BIRD_COLS + 1];
    uint8_t bird_row = bird_game_bird_row(&state->game);
    SetCursorMode(CURSOR_MODE_HIDE);
    _OS3K_ClearScreen();
    for(uint8_t row = 0; row < BIRD_ROWS; row++) {
        for(uint8_t col = 0; col < BIRD_COLS; col++) {
            char cell = ' ';
            if(row == 0 && col < 7) {
                const char* label = "S:";
                if(col < 2) cell = label[col];
                else if(col == 2) cell = (char)('0' + (state->game.score / 10) % 10);
                else if(col == 3) cell = (char)('0' + state->game.score % 10);
            }
            if(bird_game_barrier_at(&state->game, col, row)) cell = '#';
            if(col == BIRD_X && row == bird_row) cell = state->game.game_over ? 'X' : '>';
            line[col] = cell;
        }
        line[BIRD_COLS] = '\0';
        _OS3K_SetCursor((uint8_t)(row + 1), 1, CURSOR_MODE_HIDE);
        PutStringRaw(line);
    }
    if(state->game.game_over) {
        PutLine(4, "Game over SPACE restarts");
    }
}

static void Reset(FloppyState_t* state) {
    bird_game_init(&state->game);
    state->last_tick_ms = GetUptimeMilliseconds();
    state->initialized = 1;
    DrawGame(state);
}

static void Flap(FloppyState_t* state) {
    bird_game_flap(&state->game);
    DrawGame(state);
}

static void TickIfDue(FloppyState_t* state) {
    uint32_t now = GetUptimeMilliseconds();
    if(now - state->last_tick_ms < TICK_MS) return;
    state->last_tick_ms = now;
    bird_game_tick(&state->game);
    DrawGame(state);
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
            break;
        case MSG_KEY:
            if((param & 0xff) == KEY_SPACE) {
                Flap(state);
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
