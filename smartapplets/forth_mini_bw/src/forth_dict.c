#include "forth_dict.h"
#include "forth_util.h"

enum { WORD_BUILTIN = 1, WORD_USER = 2 };

static ForthResult ok_result(void) {
    ForthResult result = {FORTH_OK, {0}};
    return result;
}

void forth_seed_builtins(ForthMachine* machine) {
    ForthWord* word;
#define SEED_WORD(NAME, OPCODE) \
    word = &machine->words[machine->word_count++]; \
    forth_memset(word, 0, sizeof(*word)); \
    forth_strncpy(word->name, NAME, FORTH_MAX_NAME); \
    word->kind = WORD_BUILTIN; \
    word->opcode = OPCODE;
    SEED_WORD("+", FORTH_OP_ADD)
    SEED_WORD("-", FORTH_OP_SUB)
    SEED_WORD("*", FORTH_OP_MUL)
    SEED_WORD("/", FORTH_OP_DIV)
    SEED_WORD("mod", FORTH_OP_MOD)
    SEED_WORD("dup", FORTH_OP_DUP)
    SEED_WORD("drop", FORTH_OP_DROP)
    SEED_WORD("swap", FORTH_OP_SWAP)
    SEED_WORD("over", FORTH_OP_OVER)
    SEED_WORD(".", FORTH_OP_DOT)
    SEED_WORD(".s", FORTH_OP_DOTS)
    SEED_WORD("clear", FORTH_OP_CLEAR)
    SEED_WORD("<", FORTH_OP_LESS)
#undef SEED_WORD
}

int16_t forth_find_word(const ForthMachine* machine, const char* token) {
    int16_t i;
    for(i = (int16_t)machine->word_count - 1; i >= 0; i--) {
        if(forth_strcmp(machine->words[i].name, token) == 0) {
            return i;
        }
    }
    return -1;
}

ForthResult forth_add_user_word(ForthMachine* machine, const char* name) {
    ForthWord* word;
    if(name[0] == '\0' || forth_find_word(machine, name) >= 0) {
        ForthResult result = {FORTH_BAD_DEFINITION, "bad word name"};
        return result;
    }
    if(machine->word_count >= FORTH_MAX_WORDS) {
        ForthResult result = {FORTH_DICTIONARY_FULL, "dictionary full"};
        return result;
    }
    word = &machine->words[machine->word_count];
    forth_memset(word, 0, sizeof(*word));
    forth_strncpy(word->name, name, FORTH_MAX_NAME);
    word->kind = WORD_USER;
    word->entry = machine->code_len;
    machine->current_word = machine->word_count++;
    return ok_result();
}
