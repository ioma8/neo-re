#include "editor.h"

static bool IsWhitespace(char byte) {
    return byte == ' ' || byte == '\t' || byte == '\r' || byte == '\n';
}

static bool IsSoftSpace(char byte) {
    return byte == ' ' || byte == '\t' || byte == '\r';
}

static bool IsSupportedByte(char byte) {
    return byte == '\n' || (byte >= ' ' && byte <= '~');
}

static uint32_t WordLengthAt(const WodEditor_t* editor, uint32_t index) {
    uint32_t length = 0;
    while(index + length < editor->len && !IsWhitespace(editor->bytes[index + length])) {
        length++;
    }
    return length;
}

static void AdvanceRenderedChar(uint32_t* row, uint32_t* col) {
    *col += 1;
    if(*col == WOD_SCREEN_COLS) {
        *row += 1;
        *col = 0;
    }
}

static void WrapBeforeWord(uint32_t wordLength, uint32_t* row, uint32_t* col) {
    if(*col > 0 && wordLength <= WOD_SCREEN_COLS && wordLength > WOD_SCREEN_COLS - *col) {
        *row += 1;
        *col = 0;
    }
}

static void VisualPosition(const WodEditor_t* editor, uint32_t cursor, uint32_t* outRow, uint32_t* outCol) {
    uint32_t row = 0;
    uint32_t col = 0;
    uint32_t limit = cursor > editor->len ? editor->len : cursor;
    uint32_t i = 0;
    while(i < limit) {
        if(editor->bytes[i] == '\n') {
            row++;
            col = 0;
            i++;
        } else if(IsSoftSpace(editor->bytes[i])) {
            uint32_t spaceStart = i;
            while(i < editor->len && IsSoftSpace(editor->bytes[i])) {
                i++;
            }
            if(limit <= i) {
                if(col > 0) {
                    for(uint32_t j = spaceStart; j < limit; j++) {
                        AdvanceRenderedChar(&row, &col);
                    }
                }
                break;
            }
            uint32_t nextWordLength = WordLengthAt(editor, i);
            if(nextWordLength > 0 && col > 0 && nextWordLength <= WOD_SCREEN_COLS &&
               nextWordLength + 1 > WOD_SCREEN_COLS - col) {
                row++;
                col = 0;
            } else if(col > 0) {
                AdvanceRenderedChar(&row, &col);
            }
        } else {
            uint32_t wordLength = WordLengthAt(editor, i);
            WrapBeforeWord(wordLength, &row, &col);
            uint32_t wordEnd = i + wordLength;
            while(i < wordEnd && i < limit) {
                AdvanceRenderedChar(&row, &col);
                i++;
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

bool editor_delete_last_byte(WodEditor_t* editor) {
    if(editor->len == 0) {
        return false;
    }
    editor->len--;
    if(editor->cursor > editor->len) {
        editor->cursor = editor->len;
    }
    editor->bytes[editor->len] = 0;
    EnsureCursorVisible(editor);
    return true;
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
    uint32_t row = 0;
    uint32_t col = 0;
    for(uint8_t i = 0; i < WOD_SCREEN_COLS; i++) {
        output[i] = ' ';
    }
    output[WOD_SCREEN_COLS] = '\0';
    uint32_t i = 0;
    while(i <= editor->len) {
        if(i == editor->cursor && row == wantedRow && col < WOD_SCREEN_COLS) {
            output[col] = '|';
        }
        if(i == editor->len || row > wantedRow) {
            break;
        }
        if(editor->bytes[i] == '\n') {
            row++;
            col = 0;
            i++;
        } else if(IsSoftSpace(editor->bytes[i])) {
            uint32_t spaceStart = i;
            while(i < editor->len && IsSoftSpace(editor->bytes[i])) {
                if(i == editor->cursor && row == wantedRow && col < WOD_SCREEN_COLS) {
                    output[col] = '|';
                }
                i++;
            }
            uint32_t nextWordLength = WordLengthAt(editor, i);
            if(nextWordLength > 0 && col > 0 && nextWordLength <= WOD_SCREEN_COLS &&
               nextWordLength + 1 > WOD_SCREEN_COLS - col) {
                row++;
                col = 0;
            } else if(col > 0 && spaceStart < i) {
                if(row == wantedRow && col < WOD_SCREEN_COLS) {
                    output[col] = ' ';
                }
                AdvanceRenderedChar(&row, &col);
            }
        } else {
            uint32_t wordLength = WordLengthAt(editor, i);
            WrapBeforeWord(wordLength, &row, &col);
            uint32_t wordEnd = i + wordLength;
            while(i < wordEnd) {
                if(i == editor->cursor && row == wantedRow && col < WOD_SCREEN_COLS) {
                    output[col] = '|';
                } else if(row == wantedRow && col < WOD_SCREEN_COLS) {
                    output[col] = editor->bytes[i];
                }
                AdvanceRenderedChar(&row, &col);
                i++;
            }
        }
    }
}
