#include "../betawise-sdk/applet.h"
#include "../betawise-sdk/file_store.h"
#include "os3k.h"

enum { FILE_COUNT = 8 };
enum { MAX_FILE_BYTES = 256 };
enum { SCREEN_ROWS = 4 };
enum { SCREEN_COLS = 28 };
enum { BANNER_MS = 3000 };
static const char SLOT_MAGIC[4] = {'B', 'W', '0', '1'};

typedef struct {
    uint32_t len;
    uint32_t cursor;
    uint32_t viewport;
    char bytes[MAX_FILE_BYTES];
} Slot_t;

typedef struct {
    uint32_t active_slot;
    uint32_t banner_slot;
    uint32_t banner_until_ms;
    uint32_t loaded_mask;
    uint32_t dirty_mask;
    Slot_t slots[FILE_COUNT];
} AppState_t;

void _OS3K_ClearScreen();
void _OS3K_SetCursor(uint8_t row, uint8_t col, CursorMode_e cursor_mode);

APPLET_ENTRY(alpha_neo_process_message);

APPLET_STATE(AppState_t);

static inline Slot_t* ActiveSlot(AppState_t* state) {
    uint32_t index = state->active_slot > 0 ? state->active_slot - 1 : 0;
    if(index >= FILE_COUNT) {
        index = 0;
    }
    return &state->slots[index];
}

static void EnsureCursorVisible(Slot_t* slot);

static unsigned long SlotHandle(uint32_t slot) {
    if(slot < 1 || slot > FILE_COUNT) {
        return 1;
    }
    return (unsigned long)slot;
}

static bool BannerActive(const AppState_t* state) {
    return state->banner_slot != 0 && GetUptimeMilliseconds() < state->banner_until_ms;
}

static void ClearExpiredBanner(AppState_t* state) {
    if(state->banner_slot != 0 && !BannerActive(state)) {
        state->banner_slot = 0;
        state->banner_until_ms = 0;
    }
}

static void ShowBanner(AppState_t* state, uint32_t slot) {
    state->banner_slot = slot;
    state->banner_until_ms = GetUptimeMilliseconds() + BANNER_MS;
}

static void LoadSlot(AppState_t* state, uint32_t slot) {
    Slot_t* target = &state->slots[slot - 1];
    memset(target, 0, sizeof(*target));
    (void)applet_load_snapshot(SlotHandle(slot), SLOT_MAGIC, target, sizeof(*target));
    if(target->cursor > target->len) target->cursor = target->len;
    EnsureCursorVisible(target);
    state->loaded_mask |= 1u << (slot - 1);
}

static void SaveSlot(const AppState_t* state, uint32_t slot) {
    (void)applet_save_snapshot(SlotHandle(slot), SLOT_MAGIC, &state->slots[slot - 1], sizeof(state->slots[0]));
}

static void MarkSlotDirty(AppState_t* state, uint32_t slot) {
    state->dirty_mask |= 1u << (slot - 1);
}

static void FlushSlotIfDirty(AppState_t* state, uint32_t slot) {
    uint32_t bit = 1u << (slot - 1);
    if((state->dirty_mask & bit) != 0) {
        SaveSlot(state, slot);
        state->dirty_mask &= ~bit;
    }
}

static void EnsureSlotLoaded(AppState_t* state, uint32_t slot) {
    if((state->loaded_mask & (1u << (slot - 1))) == 0) {
        LoadSlot(state, slot);
    }
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
    uint8_t start_row = 1;
    uint8_t visible_rows = SCREEN_ROWS;
    ClearExpiredBanner(state);
    _OS3K_ClearScreen();
    SetCursorMode(CURSOR_MODE_HIDE);
    if(BannerActive(state)) {
        memset(line, ' ', SCREEN_COLS);
        line[0] = 'F';
        line[1] = 'i';
        line[2] = 'l';
        line[3] = 'e';
        line[4] = ' ';
        line[5] = (char)('0' + state->banner_slot);
        line[SCREEN_COLS] = '\0';
        _OS3K_SetCursor(1, 1, CURSOR_MODE_HIDE);
        PutStringRaw(line);
        start_row = 2;
        visible_rows = 3;
    }
    for(uint8_t row = 0; row < visible_rows; row++) {
        RenderRow(slot, row, line);
        _OS3K_SetCursor((uint8_t)(start_row + row), 1, CURSOR_MODE_HIDE);
        PutStringRaw(line);
    }
}

static void ZeroState(AppState_t* state) {
    memset(state, 0, sizeof(*state));
    state->active_slot = 1;
}

static void HandleChar(AppState_t* state, uint32_t param) {
    EnsureSlotLoaded(state, state->active_slot);
    Slot_t* slot = ActiveSlot(state);
    uint8_t byte = param & 0xff;
    if(byte == '\r' || byte == '\n') {
        if(InsertByte(slot, '\n')) {
            MarkSlotDirty(state, state->active_slot);
        }
    } else if(byte == 0x08 || byte == 0x7f) {
        if(Backspace(slot)) {
            MarkSlotDirty(state, state->active_slot);
        }
    } else if(byte >= ' ' && byte <= '~') {
        if(InsertByte(slot, (char)byte)) {
            MarkSlotDirty(state, state->active_slot);
        }
    }
}

static void HandleKey(AppState_t* state, uint32_t param, uint32_t* status) {
    EnsureSlotLoaded(state, state->active_slot);
    Slot_t* slot = ActiveSlot(state);
    uint32_t key = param & 0xff;
    uint32_t wanted_slot = 0;
    switch(key) {
        case KEY_LEFT: MoveLeft(slot); break;
        case KEY_RIGHT: MoveRight(slot); break;
        case KEY_UP: MoveUp(slot); break;
        case KEY_DOWN: MoveDown(slot); break;
        case KEY_FILE_1: wanted_slot = 1; break;
        case KEY_FILE_2: wanted_slot = 2; break;
        case KEY_FILE_3: wanted_slot = 3; break;
        case KEY_FILE_4: wanted_slot = 4; break;
        case KEY_FILE_5: wanted_slot = 5; break;
        case KEY_FILE_6: wanted_slot = 6; break;
        case KEY_FILE_7: wanted_slot = 7; break;
        case KEY_FILE_8: wanted_slot = 8; break;
        case KEY_BACKSPACE: Backspace(slot); break;
        case KEY_APPLETS:
        case KEY_ESC:
            FlushSlotIfDirty(state, state->active_slot);
            *status = APPLET_EXIT_STATUS;
            break;
        default:
            break;
    }
    if(wanted_slot != 0) {
        if(wanted_slot != state->active_slot) {
            FlushSlotIfDirty(state, state->active_slot);
            state->active_slot = wanted_slot;
        }
        EnsureSlotLoaded(state, wanted_slot);
        state->active_slot = wanted_slot;
        ShowBanner(state, wanted_slot);
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
        case MSG_IDLE:
            if(state->banner_slot != 0 && !BannerActive(state)) {
                ClearExpiredBanner(state);
                DrawDocument(state);
            }
            break;
        default:
            *status = APPLET_UNHANDLED_STATUS;
            break;
    }
}
