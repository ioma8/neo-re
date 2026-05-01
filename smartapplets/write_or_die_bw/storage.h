#ifndef WRITE_OR_DIE_STORAGE_H
#define WRITE_OR_DIE_STORAGE_H

#include <stdbool.h>

#include "app_state.h"

bool storage_load(WodAppState_t* state);
bool storage_save(const WodAppState_t* state);

#endif
