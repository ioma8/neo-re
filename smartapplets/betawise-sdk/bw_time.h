#ifndef BETAWISE_BW_TIME_H
#define BETAWISE_BW_TIME_H

#include <stdbool.h>
#include <stdint.h>

#include "os3k.h"

typedef struct {
    uint32_t next_ms;
} BwTimer;

static inline uint32_t bw_now_ms(void) {
    return GetUptimeMilliseconds();
}

static inline uint32_t bw_deadline_after(uint32_t ms) {
    return bw_now_ms() + ms;
}

static inline bool bw_time_due(uint32_t now, uint32_t deadline) {
    return (uint32_t)(now - deadline) < 0x80000000u;
}

static inline void bw_timer_reset(BwTimer* timer, uint32_t interval_ms) {
    timer->next_ms = bw_deadline_after(interval_ms);
}

static inline bool bw_timer_due_and_advance(BwTimer* timer, uint32_t interval_ms) {
    uint32_t now = bw_now_ms();
    if(!bw_time_due(now, timer->next_ms)) {
        return false;
    }
    timer->next_ms = now + interval_ms;
    return true;
}

#endif
