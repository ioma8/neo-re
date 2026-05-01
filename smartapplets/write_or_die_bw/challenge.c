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

uint32_t challenge_penalty_interval_ms(uint32_t grace_seconds) {
    uint32_t interval = GraceMs(grace_seconds) >> 1;
    return interval < 2000u ? 2000u : interval;
}

bool challenge_words_complete(uint32_t words, uint32_t goal) {
    return goal != 0 && words >= goal;
}

uint32_t challenge_remaining_seconds(uint32_t now_ms, uint32_t start_ms, uint32_t goal_seconds) {
    uint32_t elapsed_ms = now_ms >= start_ms ? now_ms - start_ms : 0;
    uint32_t elapsed_seconds = 0;
    while(elapsed_ms >= 1000u) {
        elapsed_ms -= 1000u;
        elapsed_seconds++;
    }
    if(elapsed_seconds >= goal_seconds) {
        return 0;
    }
    return goal_seconds - elapsed_seconds;
}

bool challenge_time_complete(uint32_t now_ms, uint32_t start_ms, uint32_t goal_seconds) {
    return goal_seconds != 0 && challenge_remaining_seconds(now_ms, start_ms, goal_seconds) == 0;
}
