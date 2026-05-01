#include "editor.h"

static bool IsWhitespace(char byte) {
    return byte == ' ' || byte == '\t' || byte == '\r' || byte == '\n';
}

static bool IsSupportedByte(char byte) {
    return byte == '\n' || (byte >= ' ' && byte <= '~');
}

static void VisualPosition(const WodEditor_t* editor, uint32_t cursor, uint32_t* outRow, uint32_t* outCol) {
    uint32_t row = 0;
    uint32_t col = 0;
    uint32_t limit = cursor > editor->len ? editor->len : cursor;
    for(uint32_t i = 0; i < limit; i++) {
        if(editor->bytes[i] == '\n') {
            row++;
            col = 0;
        } else {
            col++;
            if(col == WOD_SCREEN_COLS) {
                row++;
                col = 0;
            }
        }
    }
    *outRow = row;
    *outCol = col;
}

static uint32_t CursorForVisualPosition(const WodEditor_t* editor, uint32_t wantedRow, uint32_t wantedCol) {
    uint32_t targetCol = wantedCol < WOD_SCREEN_COLS ? wantedCol : WOD_SCREEN_COLS - 1;
    for(uint32_t i = 0; i <= editor->len; i++) {
        uint32_t row, col;
        VisualPosition(editor, i, &row, &col);
        if(row > wantedRow || (row == wantedRow && col >= targetCol)) {
            return i;
        }
    }
    return editor->len;
}

static void EnsureCursorVisible(WodEditor_t* editor) {
    uint32_t row, col;
    (void)col;
    VisualPosition(editor, editor->cursor, &row, &col);
    if(row < editor->viewport) {
        editor->viewport = row;
    } else if(row >= editor->viewport + WOD_TEXT_ROWS) {
        editor->viewport = row + 1 - WOD_TEXT_ROWS;
    }
}

bool editor_insert_byte(WodEditor_t* editor, char byte) {
    if(!IsSupportedByte(byte) || editor->len >= WOD_MAX_TEXT_BYTES) {
        return false;
    }
    for(uint32_t i = editor->len; i > editor->cursor; i--) {
        editor->bytes[i] = editor->bytes[i - 1];
    }
    editor->bytes[editor->cursor] = byte;
    editor->len++;
    editor->cursor++;
    EnsureCursorVisible(editor);
    return true;
}

bool editor_backspace(WodEditor_t* editor) {
    if(editor->cursor == 0) {
        return false;
    }
    for(uint32_t i = editor->cursor - 1; i + 1 < editor->len; i++) {
        editor->bytes[i] = editor->bytes[i + 1];
    }
    editor->len--;
    editor->cursor--;
    editor->bytes[editor->len] = 0;
    EnsureCursorVisible(editor);
    return true;
}

void editor_move_left(WodEditor_t* editor) {
    if(editor->cursor > 0) {
        editor->cursor--;
        EnsureCursorVisible(editor);
    }
}

void editor_move_right(WodEditor_t* editor) {
    if(editor->cursor < editor->len) {
        editor->cursor++;
        EnsureCursorVisible(editor);
    }
}

void editor_move_up(WodEditor_t* editor) {
    uint32_t row, col;
    VisualPosition(editor, editor->cursor, &row, &col);
    if(row > 0) {
        editor->cursor = CursorForVisualPosition(editor, row - 1, col);
        EnsureCursorVisible(editor);
    }
}

void editor_move_down(WodEditor_t* editor) {
    uint32_t row, col;
    VisualPosition(editor, editor->cursor, &row, &col);
    editor->cursor = CursorForVisualPosition(editor, row + 1, col);
    EnsureCursorVisible(editor);
}

uint32_t editor_word_count(const WodEditor_t* editor) {
    uint32_t count = 0;
    bool inWord = false;
    for(uint32_t i = 0; i < editor->len; i++) {
        if(IsWhitespace(editor->bytes[i])) {
            inWord = false;
        } else if(!inWord) {
            count++;
            inWord = true;
        }
    }
    return count;
}

bool editor_delete_last_word(WodEditor_t* editor) {
    uint32_t end = editor->len;
    while(end > 0 && IsWhitespace(editor->bytes[end - 1])) {
        end--;
    }
    if(end == 0) {
        return false;
    }
    uint32_t start = end;
    while(start > 0 && !IsWhitespace(editor->bytes[start - 1])) {
        start--;
    }
    for(uint32_t i = start; i + (end - start) < editor->len; i++) {
        editor->bytes[i] = editor->bytes[i + (end - start)];
    }
    editor->len -= end - start;
    if(editor->cursor > editor->len) {
        editor->cursor = editor->len;
    }
    editor->bytes[editor->len] = 0;
    EnsureCursorVisible(editor);
    return true;
}

void editor_render_row(const WodEditor_t* editor, uint8_t screenRow, char* output) {
    uint32_t wantedRow = editor->viewport + screenRow;
    bool cursorMarked = false;
    for(uint8_t i = 0; i < WOD_SCREEN_COLS; i++) {
        output[i] = ' ';
    }
    output[WOD_SCREEN_COLS] = '\0';
    for(uint32_t i = 0; i <= editor->len; i++) {
        uint32_t row, col;
        VisualPosition(editor, i, &row, &col);
        if(row > wantedRow) {
            break;
        }
        if(row == wantedRow && col < WOD_SCREEN_COLS) {
            if(i == editor->cursor) {
                output[col] = '|';
                cursorMarked = true;
            } else if(i < editor->len && editor->bytes[i] != '\n') {
                output[col] = editor->bytes[i];
            }
        }
        if(i == editor->len) {
            break;
        }
    }
    if(!cursorMarked && screenRow == 0) {
        output[0] = '|';
    }
}
