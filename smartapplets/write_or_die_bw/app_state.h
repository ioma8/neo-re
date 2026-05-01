#ifndef WRITE_OR_DIE_APP_STATE_H
#define WRITE_OR_DIE_APP_STATE_H

#include <stdint.h>

#define WOD_SCREEN_COLS 28
#define WOD_TEXT_ROWS 3
#define WOD_MAX_TEXT_BYTES 768

typedef struct {
    uint32_t len;
    uint32_t cursor;
    uint32_t viewport;
    char bytes[WOD_MAX_TEXT_BYTES];
} WodEditor_t;

typedef enum {
    WOD_PHASE_SETUP = 0,
    WOD_PHASE_RUNNING = 1,
    WOD_PHASE_COMPLETED = 2
} WodPhase_t;

typedef enum {
    WOD_GOAL_WORDS = 0,
    WOD_GOAL_TIME = 1
} WodGoalMode_t;

typedef enum {
    WOD_PRESSURE_SAFE = 0,
    WOD_PRESSURE_WARNING = 1,
    WOD_PRESSURE_DANGER = 2,
    WOD_PRESSURE_PENALTY = 3
} WodPressure_t;

#endif
