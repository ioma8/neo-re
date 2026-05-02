#ifndef WRITE_OR_DIE_UI_H
#define WRITE_OR_DIE_UI_H

#include "app_state.h"

void ui_draw_setup(WodAppState_t* state);
void ui_draw_challenge(WodAppState_t* state, WodPressure_t pressure, uint32_t remaining_seconds);
void ui_update_challenge_status(WodAppState_t* state, WodPressure_t pressure, uint32_t remaining_seconds);
void ui_update_challenge_text(WodAppState_t* state);
void ui_set_challenge_text_highlight(WodAppState_t* state, bool enabled);
void ui_draw_completed(WodAppState_t* state);
void ui_draw_export_result(WodAppState_t* state);

#endif
