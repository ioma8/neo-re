#include "../betawise-sdk/applet.h"
#include "app_state.h"
#include "challenge.h"
#include "editor.h"
#include "os3k.h"
#include "storage.h"
#include "ui.h"

APPLET_ENTRY(alpha_neo_process_message);
APPLET_STATE(WodAppState_t);

static void SetDefaults(WodAppState_t* state) {
    memset(state, 0, sizeof(*state));
    state->phase = WOD_PHASE_SETUP;
    state->goal_mode = WOD_GOAL_WORDS;
    state->word_goal = WOD_DEFAULT_WORD_GOAL;
    state->time_goal_seconds = WOD_DEFAULT_TIME_SECONDS;
    state->grace_seconds = WOD_DEFAULT_GRACE_SECONDS;
}

static void ClampState(WodAppState_t* state) {
    if(state->phase > WOD_PHASE_COMPLETED) state->phase = WOD_PHASE_SETUP;
    if(state->goal_mode > WOD_GOAL_TIME) state->goal_mode = WOD_GOAL_WORDS;
    if(state->selected_setup_row > 2) state->selected_setup_row = 0;
    if(state->word_goal < WOD_MIN_WORD_GOAL) state->word_goal = WOD_DEFAULT_WORD_GOAL;
    if(state->time_goal_seconds < WOD_MIN_TIME_SECONDS) state->time_goal_seconds = WOD_DEFAULT_TIME_SECONDS;
    if(state->grace_seconds < WOD_MIN_GRACE_SECONDS) state->grace_seconds = WOD_DEFAULT_GRACE_SECONDS;
    if(state->display_pressure > WOD_PRESSURE_PENALTY) state->display_pressure = WOD_PRESSURE_SAFE;
    if(state->editor.len > WOD_MAX_TEXT_BYTES) state->editor.len = WOD_MAX_TEXT_BYTES;
    if(state->editor.cursor > state->editor.len) state->editor.cursor = state->editor.len;
}

static void StartChallenge(WodAppState_t* state) {
    uint32_t now = GetUptimeMilliseconds();
    memset(&state->editor, 0, sizeof(state->editor));
    state->phase = WOD_PHASE_RUNNING;
    state->start_ms = now;
    state->last_activity_ms = now;
    state->last_penalty_ms = now;
    state->final_word_count = 0;
    state->dirty = 1;
    storage_save(state);
    state->display_remaining_seconds = state->time_goal_seconds;
    state->display_pressure = WOD_PRESSURE_SAFE;
    ui_draw_challenge(state, WOD_PRESSURE_SAFE, state->time_goal_seconds);
}

static uint32_t RemainingSeconds(const WodAppState_t* state, uint32_t now) {
    if(state->goal_mode == WOD_GOAL_TIME) {
        return challenge_remaining_seconds(now, state->start_ms, state->time_goal_seconds);
    }
    return state->time_goal_seconds;
}

static void DrawChallenge(WodAppState_t* state, WodPressure_t pressure, uint32_t remaining_seconds) {
    state->display_remaining_seconds = remaining_seconds;
    state->display_pressure = pressure;
    ui_draw_challenge(state, pressure, remaining_seconds);
}

static void CompleteChallenge(WodAppState_t* state) {
    state->phase = WOD_PHASE_COMPLETED;
    state->final_word_count = editor_word_count(&state->editor);
    state->dirty = 1;
    storage_save(state);
    ui_draw_completed(state);
}

static void CheckCompletion(WodAppState_t* state, uint32_t now) {
    if(state->phase != WOD_PHASE_RUNNING) {
        return;
    }
    if(state->goal_mode == WOD_GOAL_WORDS &&
       challenge_words_complete(editor_word_count(&state->editor), state->word_goal)) {
        CompleteChallenge(state);
    } else if(state->goal_mode == WOD_GOAL_TIME &&
              challenge_time_complete(now, state->start_ms, state->time_goal_seconds)) {
        CompleteChallenge(state);
    }
}

static void AdjustSetup(WodAppState_t* state, int8_t delta) {
    if(state->selected_setup_row == 0) {
        if(state->goal_mode == WOD_GOAL_WORDS) {
            int32_t next = (int32_t)state->word_goal + (int32_t)delta * 50;
            state->word_goal = next < WOD_MIN_WORD_GOAL ? WOD_MIN_WORD_GOAL : (uint32_t)next;
        } else {
            int32_t next = (int32_t)state->time_goal_seconds + (int32_t)delta * 60;
            state->time_goal_seconds = next < WOD_MIN_TIME_SECONDS ? WOD_MIN_TIME_SECONDS : (uint32_t)next;
        }
    } else if(state->selected_setup_row == 1) {
        int32_t next = (int32_t)state->grace_seconds + delta;
        state->grace_seconds = next < WOD_MIN_GRACE_SECONDS ? WOD_MIN_GRACE_SECONDS : (uint32_t)next;
    }
}

static void HandleSetupKey(WodAppState_t* state, uint32_t key, uint32_t* status) {
    switch(key) {
        case KEY_UP:
            if(state->selected_setup_row > 0) state->selected_setup_row--;
            break;
        case KEY_DOWN:
            if(state->selected_setup_row < 2) state->selected_setup_row++;
            break;
        case KEY_LEFT: AdjustSetup(state, -1); break;
        case KEY_RIGHT: AdjustSetup(state, 1); break;
        case KEY_ENTER:
            if(state->selected_setup_row == 0) {
                state->goal_mode = state->goal_mode == WOD_GOAL_WORDS ? WOD_GOAL_TIME : WOD_GOAL_WORDS;
            } else if(state->selected_setup_row == 2) {
                StartChallenge(state);
                return;
            }
            break;
        case KEY_APPLETS:
        case KEY_ESC:
            storage_save(state);
            *status = APPLET_EXIT_STATUS;
            break;
        default:
            break;
    }
    if(*status == 0) {
        storage_save(state);
        ui_draw_setup(state);
    }
}

static bool IsEnterChar(uint32_t param) {
    uint8_t byte = (uint8_t)(param & 0xff);
    return byte == '\r' || byte == '\n';
}

static bool ApplyChar(WodAppState_t* state, uint8_t byte) {
    if(byte == '\r' || byte == '\n') {
        return editor_insert_byte(&state->editor, '\n');
    }
    if(byte == 0x08 || byte == 0x7f) {
        return editor_backspace(&state->editor);
    }
    if(byte >= ' ' && byte <= '~') {
        return editor_insert_byte(&state->editor, (char)byte);
    }
    return false;
}

static void MarkActivity(WodAppState_t* state, uint32_t now) {
    state->last_activity_ms = now;
    state->last_penalty_ms = now;
    state->dirty = 1;
    storage_save(state);
}

static void HandleRunningChar(WodAppState_t* state, uint32_t param) {
    uint32_t now = GetUptimeMilliseconds();
    if(ApplyChar(state, param & 0xff)) {
        MarkActivity(state, now);
    }
    CheckCompletion(state, now);
    if(state->phase == WOD_PHASE_RUNNING) {
        DrawChallenge(state, WOD_PRESSURE_SAFE, RemainingSeconds(state, now));
    }
}

static void HandleRunningKey(WodAppState_t* state, uint32_t key, uint32_t* status) {
    switch(key) {
        case KEY_LEFT: editor_move_left(&state->editor); break;
        case KEY_RIGHT: editor_move_right(&state->editor); break;
        case KEY_UP: editor_move_up(&state->editor); break;
        case KEY_DOWN: editor_move_down(&state->editor); break;
        case KEY_BACKSPACE:
            if(editor_backspace(&state->editor)) {
                MarkActivity(state, GetUptimeMilliseconds());
            }
            break;
        case KEY_APPLETS:
        case KEY_ESC:
            storage_save(state);
            *status = APPLET_EXIT_STATUS;
            break;
        default:
            break;
    }
    if(*status == 0) {
        uint32_t now = GetUptimeMilliseconds();
        CheckCompletion(state, now);
        if(state->phase == WOD_PHASE_RUNNING) {
            uint32_t idle = now - state->last_activity_ms;
            DrawChallenge(state, challenge_pressure(idle, state->grace_seconds), RemainingSeconds(state, now));
        }
    }
}

static void HandleRunningIdle(WodAppState_t* state) {
    uint32_t now = GetUptimeMilliseconds();
    uint32_t idle = now - state->last_activity_ms;
    WodPressure_t pressure = challenge_pressure(idle, state->grace_seconds);
    bool changed = false;
    if(pressure == WOD_PRESSURE_PENALTY &&
       now - state->last_penalty_ms >= challenge_penalty_interval_ms(state->grace_seconds)) {
        if(editor_delete_last_word(&state->editor)) {
            state->dirty = 1;
            storage_save(state);
            changed = true;
        }
        state->last_penalty_ms = now;
    }
    CheckCompletion(state, now);
    if(state->phase == WOD_PHASE_RUNNING) {
        uint32_t remaining = RemainingSeconds(state, now);
        if(changed || pressure != state->display_pressure || remaining != state->display_remaining_seconds) {
            DrawChallenge(state, pressure, remaining);
        }
    }
}

void alpha_neo_process_message(uint32_t message, uint32_t param, uint32_t* status) {
    WodAppState_t* state = State();
    *status = 0;
    switch(message) {
        case MSG_SETFOCUS:
            SetDefaults(state);
            if(storage_load(state)) {
                ClampState(state);
            }
            if(state->phase == WOD_PHASE_COMPLETED) {
                ui_draw_completed(state);
            } else {
                state->phase = WOD_PHASE_SETUP;
                state->selected_setup_row = 0;
                ui_draw_setup(state);
            }
            break;
        case MSG_CHAR:
            if(state->phase == WOD_PHASE_RUNNING) {
                HandleRunningChar(state, param);
            } else if(state->phase == WOD_PHASE_SETUP && IsEnterChar(param)) {
                HandleSetupKey(state, KEY_ENTER, status);
            } else if(state->phase == WOD_PHASE_COMPLETED && IsEnterChar(param)) {
                state->phase = WOD_PHASE_SETUP;
                state->selected_setup_row = 0;
                ui_draw_setup(state);
            }
            break;
        case MSG_KEY:
            if(state->phase == WOD_PHASE_SETUP) {
                HandleSetupKey(state, param & 0xff, status);
            } else if(state->phase == WOD_PHASE_RUNNING) {
                HandleRunningKey(state, param & 0xff, status);
            } else if((param & 0xff) == KEY_ENTER) {
                state->phase = WOD_PHASE_SETUP;
                state->selected_setup_row = 0;
                ui_draw_setup(state);
            } else if((param & 0xff) == KEY_APPLETS || (param & 0xff) == KEY_ESC) {
                storage_save(state);
                *status = APPLET_EXIT_STATUS;
            }
            break;
        case MSG_IDLE:
            if(state->phase == WOD_PHASE_RUNNING) {
                HandleRunningIdle(state);
            }
            break;
        case MSG_KILLFOCUS:
            storage_save(state);
            break;
        default:
            *status = APPLET_UNHANDLED_STATUS;
            break;
    }
}
