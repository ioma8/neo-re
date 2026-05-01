#include "app_storage.h"
#include "../betawise-sdk/file_store.h"

enum { CURRENT_FILE = 1 };
static const char SNAPSHOT_MAGIC[4] = {'F', 'M', 'N', '1'};

int storage_load_machine(ForthMachine* machine) {
    return applet_load_snapshot(CURRENT_FILE, SNAPSHOT_MAGIC, machine, sizeof(*machine));
}

int storage_save_machine(const ForthMachine* machine) {
    return applet_save_snapshot(CURRENT_FILE, SNAPSHOT_MAGIC, machine, sizeof(*machine));
}
