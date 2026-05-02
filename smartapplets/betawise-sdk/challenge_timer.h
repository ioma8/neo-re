#ifndef BETAWISE_CHALLENGE_TIMER_H
#define BETAWISE_CHALLENGE_TIMER_H

#include <stdbool.h>
#include <stdint.h>

#define APPLET_PRESSURE_SAFE 0u
#define APPLET_PRESSURE_WARNING 1u
#define APPLET_PRESSURE_DANGER 2u
#define APPLET_PRESSURE_PENALTY 3u

uint32_t applet_seconds_to_milliseconds(uint32_t seconds);
uint32_t applet_milliseconds_to_seconds(uint32_t milliseconds);
uint32_t applet_elapsed_milliseconds(uint32_t start_ms, uint32_t now_ms);
uint32_t applet_remaining_seconds(uint32_t goal_seconds, uint32_t elapsed_ms);
uint32_t applet_penalty_interval_milliseconds(uint32_t grace_seconds);
uint32_t applet_pressure_stage(uint32_t idle_ms, uint32_t grace_seconds);
bool applet_flash_phase(uint32_t elapsed_ms, uint32_t interval_ms);

#endif
