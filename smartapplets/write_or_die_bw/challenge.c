#include "challenge.h"

#include "../betawise-sdk/challenge_timer.h"

WodPressure_t challenge_pressure(uint32_t idle_ms, uint32_t grace_seconds) {
    return (WodPressure_t)applet_pressure_stage(idle_ms, grace_seconds);
}

bool challenge_words_complete(uint32_t words, uint32_t goal) {
    return goal != 0 && words >= goal;
}
