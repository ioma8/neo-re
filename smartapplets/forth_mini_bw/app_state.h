#ifndef FORTH_MINI_APP_STATE_H
#define FORTH_MINI_APP_STATE_H

#include "os3k.h"
#include "src/forth_core.h"

enum { INPUT_CAPACITY = 96, LINE_WIDTH = 28, OUTPUT_LINES = 3 };

typedef struct {
    ForthMachine machine;
    char input[INPUT_CAPACITY + 1];
    uint8_t input_len;
    uint8_t storage_loaded;
    char transcript[OUTPUT_LINES][LINE_WIDTH + 1];
} AppState;

static inline AppState* State(void) {
    register char* a5 __asm__("a5");
    return (AppState*)(a5 + 0x300);
}

void app_reset(AppState* state);
void app_draw(const AppState* state);
void app_push_result(AppState* state, const char* command, const ForthResult* result, const char* output);

#endif
