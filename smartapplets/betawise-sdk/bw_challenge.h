#ifndef BETAWISE_BW_CHALLENGE_H
#define BETAWISE_BW_CHALLENGE_H

#include <stdbool.h>
#include <stdint.h>

#define BW_PRESSURE_SAFE 0u
#define BW_PRESSURE_WARNING 1u
#define BW_PRESSURE_DANGER 2u
#define BW_PRESSURE_PENALTY 3u

uint32_t bw_seconds_to_milliseconds(uint32_t seconds);
uint32_t bw_milliseconds_to_seconds(uint32_t milliseconds);
uint32_t bw_elapsed_milliseconds(uint32_t start_ms, uint32_t now_ms);
uint32_t bw_remaining_seconds(uint32_t goal_seconds, uint32_t elapsed_ms);
uint32_t bw_penalty_interval_milliseconds(uint32_t grace_seconds);
uint32_t bw_pressure_stage(uint32_t idle_ms, uint32_t grace_seconds);
bool bw_flash_phase(uint32_t elapsed_ms, uint32_t interval_ms);

#endif
