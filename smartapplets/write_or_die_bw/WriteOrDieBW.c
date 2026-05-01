#include "../betawise-sdk/applet.h"
#include "app_state.h"
#include "os3k.h"
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

static void StartChallenge(WodAppState_t* state) {
    uint32_t now = GetUptimeMilliseconds();
    state->phase = WOD_PHASE_RUNNING;
    state->start_ms = now;
    state->last_activity_ms = now;
    state->last_penalty_ms = now;
    ui_draw_challenge(state, WOD_PRESSURE_SAFE, state->time_goal_seconds);
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
            *status = APPLET_EXIT_STATUS;
            break;
        default:
            break;
    }
    if(*status == 0) {
        ui_draw_setup(state);
    }
}

void alpha_neo_process_message(uint32_t message, uint32_t param, uint32_t* status) {
    WodAppState_t* state = State();
    *status = 0;
    switch(message) {
        case MSG_SETFOCUS:
            SetDefaults(state);
            ui_draw_setup(state);
            break;
        case MSG_KEY:
            HandleSetupKey(state, param & 0xff, status);
            break;
        default:
            *status = APPLET_UNHANDLED_STATUS;
            break;
    }
}
