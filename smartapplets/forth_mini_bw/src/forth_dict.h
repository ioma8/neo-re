#ifndef FORTH_DICT_H
#define FORTH_DICT_H

#include "forth_core.h"

typedef enum {
    FORTH_OP_PUSH = 1,
    FORTH_OP_CALL,
    FORTH_OP_JUMP_IF_ZERO,
    FORTH_OP_JUMP,
    FORTH_OP_EXIT,
    FORTH_OP_ADD,
    FORTH_OP_SUB,
    FORTH_OP_MUL,
    FORTH_OP_DIV,
    FORTH_OP_MOD,
    FORTH_OP_DUP,
    FORTH_OP_DROP,
    FORTH_OP_SWAP,
    FORTH_OP_OVER,
    FORTH_OP_DOT,
    FORTH_OP_DOTS,
    FORTH_OP_CLEAR,
    FORTH_OP_LESS,
} ForthOpcode;

void forth_seed_builtins(ForthMachine* machine);
int16_t forth_find_word(const ForthMachine* machine, const char* token);
ForthResult forth_add_user_word(ForthMachine* machine, const char* name);

#endif
