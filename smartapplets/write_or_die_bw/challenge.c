#include "challenge.h"

static uint32_t GraceMs(uint32_t grace_seconds) {
    uint32_t seconds = grace_seconds == 0 ? 1 : grace_seconds;
    return seconds * 1000u;
}

WodPressure_t challenge_pressure(uint32_t idle_ms, uint32_t grace_seconds) {
    uint32_t grace = GraceMs(grace_seconds);
    if(idle_ms >= grace * 3u) {
        return WOD_PRESSURE_PENALTY;
    }
    if(idle_ms >= grace * 2u) {
        return WOD_PRESSURE_DANGER;
    }
    if(idle_ms >= grace) {
        return WOD_PRESSURE_WARNING;
    }
    return WOD_PRESSURE_SAFE;
}

bool challenge_words_complete(uint32_t words, uint32_t goal) {
    return goal != 0 && words >= goal;
}
