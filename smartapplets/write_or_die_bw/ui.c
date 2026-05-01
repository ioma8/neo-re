#include "ui.h"

#include "../betawise-sdk/os3k.h"
#include "editor.h"

void _OS3K_ClearScreen(void);
void _OS3K_SetCursor(uint8_t row, uint8_t col, CursorMode_e cursor_mode);

static void ClearLine(char* line) {
    for(uint8_t i = 0; i < WOD_SCREEN_COLS; i++) {
        line[i] = ' ';
    }
    line[WOD_SCREEN_COLS] = '\0';
}

static void PutLine(uint8_t row, const char* text) {
    char line[WOD_SCREEN_COLS + 1];
    ClearLine(line);
    for(uint8_t i = 0; i < WOD_SCREEN_COLS && text[i] != '\0'; i++) {
        line[i] = text[i];
    }
    _OS3K_SetCursor(row, 1, CURSOR_MODE_HIDE);
    PutStringRaw(line);
}

static void PutSelectedLine(uint8_t row, uint32_t selected, const char* text) {
    char line[WOD_SCREEN_COLS + 1];
    ClearLine(line);
    line[0] = selected ? '>' : ' ';
    for(uint8_t i = 0; i + 2 < WOD_SCREEN_COLS && text[i] != '\0'; i++) {
        line[i + 2] = text[i];
    }
    _OS3K_SetCursor(row, 1, CURSOR_MODE_HIDE);
    PutStringRaw(line);
}

static uint32_t DivMod60(uint32_t value, uint32_t* remainder) {
    uint32_t quotient = 0;
    while(value >= 60u) {
        value -= 60u;
        quotient++;
    }
    *remainder = value;
    return quotient;
}

void ui_draw_setup(const WodAppState_t* state) {
    char line[WOD_SCREEN_COLS + 1];
    _OS3K_ClearScreen();
    SetCursorMode(CURSOR_MODE_HIDE);
    if(state->goal_mode == WOD_GOAL_WORDS) {
        sprintf(line, "Goal: Words %lu", (unsigned long)state->word_goal);
    } else {
        uint32_t unused;
        sprintf(line, "Goal: Time %lum", (unsigned long)DivMod60(state->time_goal_seconds, &unused));
    }
    PutSelectedLine(1, state->selected_setup_row == 0, line);
    sprintf(line, "Grace: %lus", (unsigned long)state->grace_seconds);
    PutSelectedLine(2, state->selected_setup_row == 1, line);
    PutSelectedLine(3, state->selected_setup_row == 2, "Start");
    PutLine(4, state->editor.len == 0 ? "Ready" : "Draft saved");
}

static const char* PressureText(WodPressure_t pressure) {
    switch(pressure) {
        case WOD_PRESSURE_WARNING: return "write";
        case WOD_PRESSURE_DANGER: return "DANGER";
        case WOD_PRESSURE_PENALTY: return "DELETE";
        default: return "safe";
    }
}

void ui_draw_challenge(const WodAppState_t* state, WodPressure_t pressure, uint32_t remaining_seconds) {
    char line[WOD_SCREEN_COLS + 1];
    _OS3K_ClearScreen();
    SetCursorMode(CURSOR_MODE_HIDE);
    if(state->goal_mode == WOD_GOAL_WORDS) {
        sprintf(line, "%lu/%lu %s", (unsigned long)editor_word_count(&state->editor), (unsigned long)state->word_goal, PressureText(pressure));
    } else {
        uint32_t seconds;
        uint32_t minutes = DivMod60(remaining_seconds, &seconds);
        sprintf(line, "%02lu:%02lu %luw %s", (unsigned long)minutes, (unsigned long)seconds, (unsigned long)editor_word_count(&state->editor), PressureText(pressure));
    }
    PutLine(1, line);
    for(uint8_t row = 0; row < WOD_TEXT_ROWS; row++) {
        editor_render_row(&state->editor, row, line);
        PutLine((uint8_t)(row + 2), line);
    }
}

void ui_draw_completed(const WodAppState_t* state) {
    char line[WOD_SCREEN_COLS + 1];
    _OS3K_ClearScreen();
    SetCursorMode(CURSOR_MODE_HIDE);
    PutLine(1, "Done.");
    sprintf(line, "Words: %lu", (unsigned long)state->final_word_count);
    PutLine(2, line);
    PutLine(3, "Saved.");
    PutLine(4, "Applets exits");
}
