#include "os3k.h"

enum { FILE_COUNT = 8 };
enum { MAX_FILE_BYTES = 256 };
enum { SCREEN_ROWS = 4 };
enum { SCREEN_COLS = 28 };

typedef struct {
    uint32_t len;
    uint32_t cursor;
    uint32_t viewport;
    char bytes[MAX_FILE_BYTES];
} Slot_t;

typedef struct {
    uint32_t active_slot;
    Slot_t slots[FILE_COUNT];
} AppState_t;

void _OS3K_ClearScreen();
void _OS3K_SetCursor(uint8_t row, uint8_t col, CursorMode_e cursor_mode);

asm(
    ".section .text.alpha_usb_entry,\"ax\"\n"
    ".global alpha_usb_entry\n"
    "alpha_usb_entry:\n"
    "move.l 12(%sp),-(%sp)\n"
    "move.l 12(%sp),-(%sp)\n"
    "move.l 12(%sp),-(%sp)\n"
    "bsr alpha_neo_process_message\n"
    "lea 12(%sp),%sp\n"
    "rts\n"
    ".text\n"
);

static inline AppState_t* State(void) {
    register char* a5 __asm__("a5");
    return (AppState_t*)(a5 + 0x300);
}

static inline Slot_t* ActiveSlot(AppState_t* state) {
    uint32_t index = state->active_slot > 0 ? state->active_slot - 1 : 0;
    if(index >= FILE_COUNT) {
        index = 0;
    }
    return &state->slots[index];
}

static bool IsSupportedByte(char c) {
    return c == '\n' || (c >= ' ' && c <= '~');
}

static void VisualPosition(const Slot_t* slot, uint32_t cursor, uint32_t* outRow, uint32_t* outCol) {
    uint32_t row = 0;
    uint32_t col = 0;
    uint32_t limit = cursor > slot->len ? slot->len : cursor;
    for(uint32_t i = 0; i < limit; i++) {
        if(slot->bytes[i] == '\n') {
            row++;
            col = 0;
        } else {
            col++;
            if(col == SCREEN_COLS) {
                row++;
                col = 0;
            }
        }
    }
    *outRow = row;
    *outCol = col;
}

static uint32_t CursorForVisualPosition(const Slot_t* slot, uint32_t wantedRow, uint32_t wantedCol) {
    for(uint32_t i = 0; i <= slot->len; i++) {
        uint32_t row, col;
        VisualPosition(slot, i, &row, &col);
        if(row > wantedRow || (row == wantedRow && col >= (wantedCol < SCREEN_COLS ? wantedCol : (SCREEN_COLS - 1)))) {
            return i;
        }
    }
    return slot->len;
}

static void EnsureCursorVisible(Slot_t* slot) {
    uint32_t row, col;
    (void)col;
    VisualPosition(slot, slot->cursor, &row, &col);
    if(row < slot->viewport) {
        slot->viewport = row;
    } else if(row >= slot->viewport + SCREEN_ROWS) {
        slot->viewport = row + 1 - SCREEN_ROWS;
    }
}

static bool InsertByte(Slot_t* slot, char c) {
    if(!IsSupportedByte(c) || slot->len >= MAX_FILE_BYTES) {
        return false;
    }
    for(uint32_t i = slot->len; i > slot->cursor; i--) {
        slot->bytes[i] = slot->bytes[i - 1];
    }
    slot->bytes[slot->cursor] = c;
    slot->len++;
    slot->cursor++;
    EnsureCursorVisible(slot);
    return true;
}

static bool Backspace(Slot_t* slot) {
    if(slot->cursor == 0) {
        return false;
    }
    for(uint32_t i = slot->cursor - 1; i + 1 < slot->len; i++) {
        slot->bytes[i] = slot->bytes[i + 1];
    }
    slot->len--;
    slot->cursor--;
    slot->bytes[slot->len] = 0;
    EnsureCursorVisible(slot);
    return true;
}

static void MoveLeft(Slot_t* slot) {
    if(slot->cursor > 0) {
        slot->cursor--;
        EnsureCursorVisible(slot);
    }
}

static void MoveRight(Slot_t* slot) {
    if(slot->cursor < slot->len) {
        slot->cursor++;
        EnsureCursorVisible(slot);
    }
}

static void MoveUp(Slot_t* slot) {
    uint32_t row, col;
    VisualPosition(slot, slot->cursor, &row, &col);
    if(row == 0) {
        return;
    }
    slot->cursor = CursorForVisualPosition(slot, row - 1, col);
    EnsureCursorVisible(slot);
}

static void MoveDown(Slot_t* slot) {
    uint32_t row, col;
    VisualPosition(slot, slot->cursor, &row, &col);
    slot->cursor = CursorForVisualPosition(slot, row + 1, col);
    EnsureCursorVisible(slot);
}

static void RenderRow(const Slot_t* slot, uint8_t screenRow, char* output) {
    uint32_t wantedRow = slot->viewport + screenRow;
    bool cursorMarked = false;

    for(uint8_t i = 0; i < SCREEN_COLS; i++) {
        output[i] = ' ';
    }
    output[SCREEN_COLS] = '\0';

    for(uint32_t i = 0; i <= slot->len; i++) {
        uint32_t row, col;
        VisualPosition(slot, i, &row, &col);
        if(row > wantedRow) {
            break;
        }
        if(row == wantedRow && col < SCREEN_COLS) {
            if(i == slot->cursor) {
                output[col] = '|';
                cursorMarked = true;
            } else if(i < slot->len && slot->bytes[i] != '\n') {
                output[col] = slot->bytes[i];
            }
        }
        if(i == slot->len) {
            break;
        }
    }

    if(!cursorMarked && screenRow == 0) {
        output[0] = '|';
    }
}

static void DrawDocument(AppState_t* state) {
    char line[SCREEN_COLS + 1];
    Slot_t* slot = ActiveSlot(state);
    _OS3K_ClearScreen();
    SetCursorMode(CURSOR_MODE_HIDE);
    for(uint8_t row = 0; row < SCREEN_ROWS; row++) {
        RenderRow(slot, row, line);
        _OS3K_SetCursor(row + 1, 1, CURSOR_MODE_HIDE);
        PutStringRaw(line);
    }
}

static void ZeroState(AppState_t* state) {
    memset(state, 0, sizeof(*state));
    state->active_slot = 1;
}

static void HandleChar(AppState_t* state, uint32_t param) {
    Slot_t* slot = ActiveSlot(state);
    uint8_t byte = param & 0xff;
    if(byte == '\r' || byte == '\n') {
        InsertByte(slot, '\n');
    } else if(byte == 0x08 || byte == 0x7f) {
        Backspace(slot);
    } else if(byte >= ' ' && byte <= '~') {
        InsertByte(slot, (char)byte);
    }
}

static void HandleKey(AppState_t* state, uint32_t param, uint32_t* status) {
    Slot_t* slot = ActiveSlot(state);
    uint32_t key = param & 0xff;
    switch(key) {
        case KEY_LEFT: MoveLeft(slot); break;
        case KEY_RIGHT: MoveRight(slot); break;
        case KEY_UP: MoveUp(slot); break;
        case KEY_DOWN: MoveDown(slot); break;
        case KEY_FILE_1: state->active_slot = 1; break;
        case KEY_FILE_2: state->active_slot = 2; break;
        case KEY_FILE_3: state->active_slot = 3; break;
        case KEY_FILE_4: state->active_slot = 4; break;
        case KEY_FILE_5: state->active_slot = 5; break;
        case KEY_FILE_6: state->active_slot = 6; break;
        case KEY_FILE_7: state->active_slot = 7; break;
        case KEY_FILE_8: state->active_slot = 8; break;
        case KEY_BACKSPACE: Backspace(slot); break;
        case KEY_APPLETS:
        case KEY_ESC:
            *status = 0x07;
            break;
        default:
            break;
    }
}

void alpha_neo_process_message(uint32_t message, uint32_t param, uint32_t* status) {
    AppState_t* state = State();
    *status = 0;
    switch(message) {
        case MSG_SETFOCUS:
            ZeroState(state);
            DrawDocument(state);
            break;
        case MSG_CHAR:
            HandleChar(state, param);
            DrawDocument(state);
            break;
        case MSG_KEY:
            HandleKey(state, param, status);
            if(*status == 0) {
                DrawDocument(state);
            }
            break;
        default:
            *status = 0x04;
            break;
    }
}
