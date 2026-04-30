#ifndef FORTH_VM_H
#define FORTH_VM_H

#include "forth_core.h"

ForthResult forth_exec_builtin(
    ForthMachine* machine,
    uint8_t opcode,
    char* output,
    size_t output_size);
ForthResult forth_exec_user_word(
    ForthMachine* machine,
    uint16_t word_index,
    char* output,
    size_t output_size);

#endif
