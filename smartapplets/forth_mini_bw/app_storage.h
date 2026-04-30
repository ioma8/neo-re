#ifndef FORTH_MINI_APP_STORAGE_H
#define FORTH_MINI_APP_STORAGE_H

#include "src/forth_core.h"

int storage_load_machine(ForthMachine* machine);
int storage_save_machine(const ForthMachine* machine);

#endif
