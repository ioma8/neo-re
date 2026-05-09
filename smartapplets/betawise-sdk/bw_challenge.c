#include "bw_challenge.h"

static uint32_t GraceSeconds(uint32_t grace_seconds) {
    return grace_seconds;
}

uint32_t bw_seconds_to_milliseconds(uint32_t seconds) {
    uint32_t milliseconds = 0;
    while(seconds > 0) {
        milliseconds += 1000u;
        seconds--;
    }
    return milliseconds;
}

uint32_t bw_milliseconds_to_seconds(uint32_t milliseconds) {
    uint32_t seconds = 0;
    while(milliseconds >= 1000u) {
        milliseconds -= 1000u;
        seconds++;
    }
    return seconds;
}

uint32_t bw_elapsed_milliseconds(uint32_t start_ms, uint32_t now_ms) {
    return now_ms - start_ms;
}

uint32_t bw_remaining_seconds(uint32_t goal_seconds, uint32_t elapsed_ms) {
    uint32_t elapsed_seconds = bw_milliseconds_to_seconds(elapsed_ms);
    if(elapsed_seconds >= goal_seconds) {
        return 0;
    }
    return goal_seconds - elapsed_seconds;
}

uint32_t bw_penalty_interval_milliseconds(uint32_t grace_seconds) {
    (void)grace_seconds;
    return 700u;
}

uint32_t bw_pressure_stage(uint32_t idle_ms, uint32_t grace_seconds) {
    uint32_t grace = bw_seconds_to_milliseconds(GraceSeconds(grace_seconds));
    if(idle_ms >= grace + 4000u) {
        return BW_PRESSURE_PENALTY;
    }
    if(idle_ms >= grace) {
        return BW_PRESSURE_WARNING;
    }
    return BW_PRESSURE_SAFE;
}

bool bw_flash_phase(uint32_t elapsed_ms, uint32_t interval_ms) {
    bool phase = false;
    if(interval_ms == 0) {
        return false;
    }
    while(elapsed_ms >= interval_ms) {
        elapsed_ms -= interval_ms;
        phase = !phase;
    }
    return phase;
}
