#include "storage.h"

#include "../betawise-sdk/file_store.h"

static const char WOD_MAGIC[4] = {'W', 'O', 'D', '1'};

bool storage_load(WodAppState_t* state) {
    return applet_load_snapshot(1, WOD_MAGIC, state, sizeof(*state)) != 0;
}

bool storage_save(const WodAppState_t* state) {
    return applet_save_snapshot(1, WOD_MAGIC, state, sizeof(*state)) != 0;
}
