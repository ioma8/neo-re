#ifndef WRITE_OR_DIE_UI_H
#define WRITE_OR_DIE_UI_H

#include "app_state.h"

void ui_draw_setup(const WodAppState_t* state);
void ui_draw_challenge(const WodAppState_t* state, WodPressure_t pressure, uint32_t remaining_seconds);
void ui_draw_completed(const WodAppState_t* state);

#endif
