#ifndef FORTH_COMPILE_H
#define FORTH_COMPILE_H

#include "forth_core.h"

ForthResult forth_compile_text(
    ForthMachine* machine,
    const char* text,
    char* output,
    size_t output_size);

#endif
