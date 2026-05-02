#ifndef WRITE_OR_DIE_APP_STATE_H
#define WRITE_OR_DIE_APP_STATE_H

#include <stdint.h>

#define WOD_SCREEN_COLS 41
#define WOD_TEXT_ROWS 3
#define WOD_MAX_TEXT_BYTES 768
#define WOD_DEFAULT_WORD_GOAL 500
#define WOD_DEFAULT_TIME_SECONDS 600
#define WOD_DEFAULT_GRACE_SECONDS 10
#define WOD_MIN_WORD_GOAL 5
#define WOD_MIN_TIME_SECONDS 60
#define WOD_MIN_GRACE_SECONDS 0
#define WOD_MAX_GRACE_SECONDS 30

typedef struct {
    uint32_t len;
    uint32_t cursor;
    uint32_t viewport;
    char bytes[WOD_MAX_TEXT_BYTES];
} WodEditor_t;

typedef enum {
    WOD_PHASE_SETUP = 0,
    WOD_PHASE_RUNNING = 1,
    WOD_PHASE_COMPLETED = 2,
    WOD_PHASE_EXPORTED = 3
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

typedef struct {
    uint32_t phase;
    uint32_t selected_setup_row;
    uint32_t goal_mode;
    uint32_t word_goal;
    uint32_t time_goal_seconds;
    uint32_t grace_seconds;
    WodEditor_t editor;
    uint32_t start_ms;
    uint32_t last_activity_ms;
    uint32_t last_penalty_ms;
    uint32_t final_word_count;
    uint32_t dirty;
    uint32_t display_remaining_seconds;
    uint32_t display_pressure;
    uint32_t export_slot;
    uint32_t export_status;
    uint32_t display_flash_on;
    char display_status_line[WOD_SCREEN_COLS + 1];
    char display_text_lines[WOD_TEXT_ROWS][WOD_SCREEN_COLS + 1];
} WodAppState_t;

#endif
