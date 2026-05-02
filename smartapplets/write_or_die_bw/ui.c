#include "ui.h"

#include "../betawise-sdk/os3k.h"
#include "../betawise-sdk/screen_lines.h"
#include "editor.h"

void _OS3K_ClearScreen(void);

static void PutLine(uint8_t row, const char* text) {
    applet_screen_put_line(row, text, WOD_SCREEN_COLS);
}

static void SetDisplayReverse(bool enabled) {
    LCD_CMD_REG_LEFT = LCD_CMD_ON(1u);
    LCD_CMD_REG_RIGHT = LCD_CMD_ON(1u);
    LCD_CMD_REG_LEFT = LCD_CMD_REVERSE(enabled ? 1u : 0u);
    LCD_CMD_REG_RIGHT = LCD_CMD_REVERSE(enabled ? 1u : 0u);
}

static void PutCachedLine(uint8_t row, const char* text, char* cache) {
    applet_screen_put_cached_line(row, text, cache, WOD_SCREEN_COLS);
}

static void InvalidateChallengeCache(WodAppState_t* state) {
    SetDisplayReverse(false);
    applet_screen_invalidate_cache(state->display_status_line);
    for(uint8_t row = 0; row < WOD_TEXT_ROWS; row++) {
        applet_screen_invalidate_cache(state->display_text_lines[row]);
    }
    state->display_flash_on = 0;
}

static void PutSelectedLine(uint8_t row, uint32_t selected, const char* text) {
    char line[WOD_SCREEN_COLS + 1];
    applet_screen_clear_line(line, WOD_SCREEN_COLS);
    line[0] = selected ? '>' : ' ';
    for(uint8_t i = 0; i + 2 < WOD_SCREEN_COLS && text[i] != '\0'; i++) {
        line[i + 2] = text[i];
    }
    PutLine(row, line);
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

void ui_draw_setup(WodAppState_t* state) {
    char line[WOD_SCREEN_COLS + 1];
    _OS3K_ClearScreen();
    InvalidateChallengeCache(state);
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

void ui_draw_challenge(WodAppState_t* state, WodPressure_t pressure, uint32_t remaining_seconds) {
    _OS3K_ClearScreen();
    InvalidateChallengeCache(state);
    SetCursorMode(CURSOR_MODE_HIDE);
    ui_update_challenge_status(state, pressure, remaining_seconds);
    ui_update_challenge_text(state);
}

void ui_update_challenge_status(WodAppState_t* state, WodPressure_t pressure, uint32_t remaining_seconds) {
    char line[WOD_SCREEN_COLS + 1];
    if(state->goal_mode == WOD_GOAL_WORDS) {
        sprintf(line, "%lu/%lu %s", (unsigned long)editor_word_count(&state->editor), (unsigned long)state->word_goal, PressureText(pressure));
    } else {
        uint32_t seconds;
        uint32_t minutes = DivMod60(remaining_seconds, &seconds);
        sprintf(line, "%02lu:%02lu %luw %s", (unsigned long)minutes, (unsigned long)seconds, (unsigned long)editor_word_count(&state->editor), PressureText(pressure));
    }
    PutCachedLine(1, line, state->display_status_line);
}

void ui_update_challenge_text(WodAppState_t* state) {
    char line[WOD_SCREEN_COLS + 1];
    bool restore_highlight = state->display_flash_on != 0;
    if(restore_highlight) {
        ui_set_challenge_text_highlight(state, false);
    }
    for(uint8_t row = 0; row < WOD_TEXT_ROWS; row++) {
        editor_render_row(&state->editor, row, line);
        PutCachedLine((uint8_t)(row + 2), line, state->display_text_lines[row]);
    }
    if(restore_highlight) {
        ui_set_challenge_text_highlight(state, true);
    }
}

void ui_set_challenge_text_highlight(WodAppState_t* state, bool enabled) {
    if((state->display_flash_on != 0) == enabled) {
        return;
    }
    SetDisplayReverse(enabled);
    state->display_flash_on = enabled ? 1u : 0u;
}

void ui_draw_completed(WodAppState_t* state) {
    char line[WOD_SCREEN_COLS + 1];
    _OS3K_ClearScreen();
    InvalidateChallengeCache(state);
    SetCursorMode(CURSOR_MODE_HIDE);
    PutLine(1, "Done.");
    sprintf(line, "Words: %lu", (unsigned long)state->final_word_count);
    PutLine(2, line);
    PutLine(3, "Press File 1-8");
    PutLine(4, "append to AlphaWord");
}

void ui_draw_export_result(WodAppState_t* state) {
    char line[WOD_SCREEN_COLS + 1];
    _OS3K_ClearScreen();
    InvalidateChallengeCache(state);
    SetCursorMode(CURSOR_MODE_HIDE);
    if(state->export_status == 1) {
        sprintf(line, "Appended to File %lu", (unsigned long)state->export_slot);
        PutLine(1, line);
        PutLine(2, "WriteOrDie session");
        PutLine(3, "saved in AlphaWord");
    } else {
        PutLine(1, "Append failed");
        PutLine(2, "Open AlphaWord first");
        PutLine(3, "then try again");
    }
    PutLine(4, "Applets exits");
}
