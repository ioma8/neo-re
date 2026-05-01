#ifndef WRITE_OR_DIE_CHALLENGE_H
#define WRITE_OR_DIE_CHALLENGE_H

#include <stdbool.h>
#include <stdint.h>

#include "app_state.h"

WodPressure_t challenge_pressure(uint32_t idle_ms, uint32_t grace_seconds);
bool challenge_words_complete(uint32_t words, uint32_t goal);

#endif
