#include "../betawise-sdk/applet.h"
#include "../betawise-sdk/challenge_timer.h"
#include "alpha_export.h"
#include "app_state.h"
#include "challenge.h"
#include "editor.h"
#include "os3k.h"
#include "storage.h"
#include "ui.h"

APPLET_ENTRY(alpha_neo_process_message);
APPLET_STATE(WodAppState_t);

static uint32_t UptimeMilliseconds(void) {
    return GetUptimeMilliseconds();
}

static uint32_t ElapsedMilliseconds(const WodAppState_t* state, uint32_t now_ms) {
    return applet_elapsed_milliseconds(state->start_ms, now_ms);
}

static void SetDefaults(WodAppState_t* state) {
    memset(state, 0, sizeof(*state));
    state->phase = WOD_PHASE_SETUP;
    state->goal_mode = WOD_GOAL_WORDS;
    state->word_goal = WOD_DEFAULT_WORD_GOAL;
    state->time_goal_seconds = WOD_DEFAULT_TIME_SECONDS;
    state->grace_seconds = WOD_DEFAULT_GRACE_SECONDS;
}

static void ClampState(WodAppState_t* state) {
    if(state->phase > WOD_PHASE_EXPORTED) state->phase = WOD_PHASE_SETUP;
    if(state->goal_mode > WOD_GOAL_TIME) state->goal_mode = WOD_GOAL_WORDS;
    if(state->selected_setup_row > 2) state->selected_setup_row = 0;
    if(state->word_goal < WOD_MIN_WORD_GOAL) state->word_goal = WOD_DEFAULT_WORD_GOAL;
    if(state->time_goal_seconds < WOD_MIN_TIME_SECONDS) state->time_goal_seconds = WOD_DEFAULT_TIME_SECONDS;
    if(state->grace_seconds < WOD_MIN_GRACE_SECONDS) state->grace_seconds = WOD_DEFAULT_GRACE_SECONDS;
    if(state->display_pressure > WOD_PRESSURE_PENALTY) state->display_pressure = WOD_PRESSURE_SAFE;
    if(state->editor.len > WOD_MAX_TEXT_BYTES) state->editor.len = WOD_MAX_TEXT_BYTES;
    if(state->editor.cursor > state->editor.len) state->editor.cursor = state->editor.len;
    if(state->export_slot > 8) state->export_slot = 0;
    if(state->export_status > 2) state->export_status = 0;
}

static void MarkDirty(WodAppState_t* state) {
    state->dirty = 1;
}

static void SaveIfDirty(WodAppState_t* state) {
    if(state->dirty != 0) {
        storage_save(state);
        state->dirty = 0;
    }
}

static void StartChallenge(WodAppState_t* state) {
    uint32_t now = UptimeMilliseconds();
    memset(&state->editor, 0, sizeof(state->editor));
    state->phase = WOD_PHASE_RUNNING;
    state->start_ms = now;
    state->last_activity_ms = now;
    state->last_penalty_ms = now;
    state->final_word_count = 0;
    state->export_slot = 0;
    state->export_status = 0;
    MarkDirty(state);
    SaveIfDirty(state);
    state->display_remaining_seconds = state->time_goal_seconds;
    state->display_pressure = WOD_PRESSURE_SAFE;
    ui_draw_challenge(state, WOD_PRESSURE_SAFE, state->time_goal_seconds);
}

static uint32_t RemainingSeconds(const WodAppState_t* state, uint32_t elapsed_ms) {
    if(state->goal_mode == WOD_GOAL_TIME) {
        return applet_remaining_seconds(state->time_goal_seconds, elapsed_ms);
    }
    return state->time_goal_seconds;
}

static void DrawChallenge(WodAppState_t* state, WodPressure_t pressure, uint32_t remaining_seconds, bool text_changed) {
    bool status_changed = text_changed ||
                          remaining_seconds != state->display_remaining_seconds ||
                          pressure != state->display_pressure;
    state->display_remaining_seconds = remaining_seconds;
    state->display_pressure = pressure;
    if(status_changed) {
        ui_update_challenge_status(state, pressure, remaining_seconds);
    }
    if(text_changed) {
        ui_update_challenge_text(state);
    }
}

static bool ShouldFlashText(WodPressure_t pressure) {
    return pressure >= WOD_PRESSURE_DANGER;
}

static bool FlashPhase(uint32_t idle_ms) {
    bool phase = false;
    while(idle_ms >= 500u) {
        idle_ms -= 500u;
        phase = !phase;
    }
    return phase;
}

static void CompleteChallenge(WodAppState_t* state) {
    state->phase = WOD_PHASE_COMPLETED;
    state->final_word_count = editor_word_count(&state->editor);
    state->export_slot = 0;
    state->export_status = 0;
    MarkDirty(state);
    SaveIfDirty(state);
    ui_draw_completed(state);
}

static void CheckCompletion(WodAppState_t* state, uint32_t elapsed_ms) {
    if(state->phase != WOD_PHASE_RUNNING) {
        return;
    }
    if(state->goal_mode == WOD_GOAL_WORDS &&
       challenge_words_complete(editor_word_count(&state->editor), state->word_goal)) {
        CompleteChallenge(state);
    } else if(state->goal_mode == WOD_GOAL_TIME &&
              elapsed_ms >= applet_seconds_to_milliseconds(state->time_goal_seconds)) {
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
                MarkDirty(state);
            } else if(state->selected_setup_row == 2) {
                StartChallenge(state);
                return;
            }
            break;
        case KEY_APPLETS:
        case KEY_ESC:
            SaveIfDirty(state);
            *status = APPLET_EXIT_STATUS;
            break;
        default:
            break;
    }
    if(*status == 0) {
        ui_draw_setup(state);
    }
}

static bool IsEnterChar(uint32_t param) {
    uint8_t byte = (uint8_t)(param & 0xff);
    return byte == '\r' || byte == '\n';
}

static bool IsExitKey(uint32_t key) {
    return key == KEY_APPLETS || key == KEY_ESC || key == 0x46;
}

static uint32_t FileSlotForKey(uint32_t key) {
    switch(key) {
        case KEY_FILE_1: return 1;
        case KEY_FILE_2: return 2;
        case KEY_FILE_3: return 3;
        case KEY_FILE_4: return 4;
        case KEY_FILE_5: return 5;
        case KEY_FILE_6: return 6;
        case KEY_FILE_7: return 7;
        case KEY_FILE_8: return 8;
        default: return 0;
    }
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

static void MarkActivity(WodAppState_t* state, uint32_t now_ms) {
    state->last_activity_ms = now_ms;
    state->last_penalty_ms = now_ms;
    MarkDirty(state);
}

static void HandleRunningChar(WodAppState_t* state, uint32_t param) {
    uint32_t now = UptimeMilliseconds();
    uint32_t elapsed = ElapsedMilliseconds(state, now);
    bool changed = ApplyChar(state, param & 0xff);
    if(changed) {
        MarkActivity(state, now);
        ui_set_challenge_text_highlight(state, false);
    }
    CheckCompletion(state, elapsed);
    if(state->phase == WOD_PHASE_RUNNING) {
        DrawChallenge(state, WOD_PRESSURE_SAFE, RemainingSeconds(state, elapsed), changed);
    }
}

static void HandleRunningKey(WodAppState_t* state, uint32_t key, uint32_t* status) {
    bool text_changed = false;
    switch(key) {
        case KEY_LEFT: editor_move_left(&state->editor); text_changed = true; break;
        case KEY_RIGHT: editor_move_right(&state->editor); text_changed = true; break;
        case KEY_UP: editor_move_up(&state->editor); text_changed = true; break;
        case KEY_DOWN: editor_move_down(&state->editor); text_changed = true; break;
        case KEY_BACKSPACE:
            if(editor_backspace(&state->editor)) {
                MarkActivity(state, UptimeMilliseconds());
                text_changed = true;
            }
            break;
        case KEY_APPLETS:
        case KEY_ESC:
            SaveIfDirty(state);
            *status = APPLET_EXIT_STATUS;
            break;
        default:
            break;
    }
    if(*status == 0) {
        uint32_t now = UptimeMilliseconds();
        uint32_t elapsed = ElapsedMilliseconds(state, now);
        CheckCompletion(state, elapsed);
        if(state->phase == WOD_PHASE_RUNNING) {
            uint32_t idle = now - state->last_activity_ms;
            WodPressure_t pressure = challenge_pressure(idle, state->grace_seconds);
            DrawChallenge(state, pressure, RemainingSeconds(state, elapsed), text_changed);
            ui_set_challenge_text_highlight(state, ShouldFlashText(pressure) && FlashPhase(idle));
        }
    }
}

static void HandleRunningIdle(WodAppState_t* state) {
    uint32_t now = UptimeMilliseconds();
    uint32_t elapsed = ElapsedMilliseconds(state, now);
    uint32_t idle = now - state->last_activity_ms;
    WodPressure_t pressure = challenge_pressure(idle, state->grace_seconds);
    bool changed = false;
    if(pressure == WOD_PRESSURE_PENALTY &&
       now - state->last_penalty_ms >= applet_penalty_interval_milliseconds(state->grace_seconds)) {
        if(editor_delete_last_word(&state->editor)) {
            MarkDirty(state);
            changed = true;
        }
        state->last_penalty_ms = now;
    }
    CheckCompletion(state, elapsed);
    if(state->phase == WOD_PHASE_RUNNING) {
        uint32_t remaining = RemainingSeconds(state, elapsed);
        if(changed || pressure != state->display_pressure || remaining != state->display_remaining_seconds) {
            DrawChallenge(state, pressure, remaining, changed);
        }
        ui_set_challenge_text_highlight(state, ShouldFlashText(pressure) && FlashPhase(idle));
    }
}

static void ResetToSetup(WodAppState_t* state) {
    state->phase = WOD_PHASE_SETUP;
    state->selected_setup_row = 0;
    MarkDirty(state);
    SaveIfDirty(state);
    ui_draw_setup(state);
}

static void ExportCompletedSession(WodAppState_t* state, uint32_t slot) {
    uint32_t export_status = alpha_export_append_session(&state->editor, slot);
    bool ok = export_status == 1;
    state->export_slot = slot;
    state->export_status = export_status;
    if(ok) {
        state->phase = WOD_PHASE_EXPORTED;
    }
    MarkDirty(state);
    SaveIfDirty(state);
    ui_draw_export_result(state);
}

static void HandleCompletedKey(WodAppState_t* state, uint32_t key, uint32_t* status) {
    uint32_t slot = FileSlotForKey(key);
    if(slot != 0) {
        ExportCompletedSession(state, slot);
    } else if(key == KEY_ENTER) {
        ResetToSetup(state);
    } else if(IsExitKey(key)) {
        SaveIfDirty(state);
        *status = APPLET_EXIT_STATUS;
    }
}

static void HandleExportedKey(WodAppState_t* state, uint32_t key, uint32_t* status) {
    if(key == KEY_ENTER) {
        ResetToSetup(state);
    } else if(IsExitKey(key)) {
        SaveIfDirty(state);
        *status = APPLET_EXIT_STATUS;
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
            } else if(state->phase == WOD_PHASE_EXPORTED) {
                ui_draw_export_result(state);
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
            } else if((state->phase == WOD_PHASE_COMPLETED || state->phase == WOD_PHASE_EXPORTED) && IsEnterChar(param)) {
                ResetToSetup(state);
            }
            break;
        case MSG_KEY:
            if(state->phase == WOD_PHASE_SETUP) {
                HandleSetupKey(state, param & 0xff, status);
            } else if(state->phase == WOD_PHASE_RUNNING) {
                HandleRunningKey(state, param & 0xff, status);
            } else if(state->phase == WOD_PHASE_COMPLETED) {
                HandleCompletedKey(state, param & 0xff, status);
            } else if(state->phase == WOD_PHASE_EXPORTED) {
                HandleExportedKey(state, param & 0xff, status);
            }
            break;
        case MSG_IDLE:
            if(state->phase == WOD_PHASE_RUNNING) {
                HandleRunningIdle(state);
            }
            break;
        case MSG_KILLFOCUS:
            SaveIfDirty(state);
            break;
        default:
            *status = APPLET_UNHANDLED_STATUS;
            break;
    }
}
