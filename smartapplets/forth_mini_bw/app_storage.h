#ifndef FORTH_MINI_APP_STORAGE_H
#define FORTH_MINI_APP_STORAGE_H

#include <stddef.h>

int storage_load(char* buffer, size_t capacity);
int storage_save(const char* buffer);

#endif
