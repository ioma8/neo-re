#include "app_state.h"
#include "src/forth_util.h"

void _OS3K_ClearScreen();
void _OS3K_SetCursor(uint8_t row, uint8_t col, CursorMode_e cursor_mode);

static void clear_line(char* line) {
    forth_memset(line, ' ', LINE_WIDTH);
    line[LINE_WIDTH] = '\0';
}

static void copy_line(char* line, const char* text) {
    size_t len = forth_strlen(text);
    clear_line(line);
    if(len > LINE_WIDTH) len = LINE_WIDTH;
    forth_memcpy(line, text, len);
}

static void shift_transcript(AppState* state) {
    forth_memcpy(state->transcript[0], state->transcript[1], LINE_WIDTH + 1);
    forth_memcpy(state->transcript[1], state->transcript[2], LINE_WIDTH + 1);
    clear_line(state->transcript[2]);
}

static void draw_line(uint8_t row, const char* line) {
    _OS3K_SetCursor(row, 1, CURSOR_MODE_HIDE);
    PutStringRaw(line);
}

void app_reset(AppState* state) {
    forth_memset(state, 0, sizeof(*state));
    forth_init(&state->machine);
    clear_line(state->transcript[0]);
    clear_line(state->transcript[1]);
    clear_line(state->transcript[2]);
}

void app_draw(const AppState* state) {
    uint8_t row;
    uint8_t visible_len;
    uint8_t cursor_col;
    const char* visible_input;
    char prompt[LINE_WIDTH + 1];
    clear_line(prompt);
    visible_len = state->input_len;
    visible_input = state->input;
    if(visible_len > LINE_WIDTH) {
        visible_input = state->input + (visible_len - LINE_WIDTH);
        visible_len = LINE_WIDTH;
    }
    forth_memcpy(prompt, visible_input, visible_len);
    cursor_col = (uint8_t)(visible_len + 1);
    if(cursor_col > LINE_WIDTH) cursor_col = LINE_WIDTH;
    _OS3K_ClearScreen();
    SetCursorMode(CURSOR_MODE_HIDE);
    for(row = 0; row < OUTPUT_LINES; row++) {
        draw_line((uint8_t)(row + 1), state->transcript[row]);
    }
    draw_line(4, prompt);
    _OS3K_SetCursor(4, cursor_col, CURSOR_MODE_SHOW);
    SetCursorMode(CURSOR_MODE_SHOW);
}

void app_push_result(AppState* state, const char* command, const ForthResult* result, const char* output) {
    char line[96];
    char* cursor = line;
    shift_transcript(state);
    line[0] = '\0';
    forth_strcpy(cursor, command);
    cursor += forth_strlen(cursor);
    if(result->code == FORTH_OK) {
        *cursor++ = ' ';
        *cursor = '\0';
        forth_strcpy(cursor, output[0] != '\0' ? output : "ok");
    } else {
        *cursor++ = ' ';
        *cursor = '\0';
        forth_strcpy(cursor, result->message);
    }
    copy_line(state->transcript[2], line);
}
