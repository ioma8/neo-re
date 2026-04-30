#ifndef FORTH_CORE_H
#define FORTH_CORE_H

#include <stddef.h>
#include <stdint.h>

enum {
    FORTH_MAX_STACK = 32,
    FORTH_MAX_RETURN = 32,
    FORTH_MAX_WORDS = 64,
    FORTH_MAX_NAME = 16,
    FORTH_MAX_CODE = 1024,
    FORTH_MAX_PATCH = 32,
    FORTH_MAX_SOURCE = 4032,
    FORTH_MAX_TOKEN = 32,
    FORTH_MAX_MESSAGE = 48,
};

typedef enum {
    FORTH_OK = 0,
    FORTH_DIVIDE_BY_ZERO,
    FORTH_STACK_OVERFLOW,
    FORTH_STACK_UNDERFLOW,
    FORTH_UNKNOWN_WORD,
    FORTH_BAD_DEFINITION,
    FORTH_BAD_CONTROL_FLOW,
    FORTH_CODE_OVERFLOW,
    FORTH_DICTIONARY_FULL,
    FORTH_SOURCE_FULL,
    FORTH_OUTPUT_FULL,
} ForthCode;

typedef struct {
    ForthCode code;
    char message[FORTH_MAX_MESSAGE];
} __attribute__((aligned(2))) ForthResult;

typedef struct {
    uint8_t op;
    int16_t arg;
} ForthInstr;

typedef struct {
    char name[FORTH_MAX_NAME + 1];
    uint8_t kind;
    uint8_t opcode;
    uint16_t entry;
} ForthWord;

typedef struct {
    uint8_t kind;
    uint16_t patch_ip;
    uint16_t begin_ip;
} ForthPatch;

typedef struct {
    int16_t stack[FORTH_MAX_STACK];
    uint16_t depth;
    ForthInstr code[FORTH_MAX_CODE];
    uint16_t code_len;
    ForthWord words[FORTH_MAX_WORDS];
    uint16_t word_count;
    ForthPatch patches[FORTH_MAX_PATCH];
    uint16_t patch_depth;
    uint16_t current_word;
    uint8_t compile_mode;
    uint8_t expect_name;
    char source[FORTH_MAX_SOURCE];
    uint16_t source_len;
} ForthMachine;

void forth_init(ForthMachine* machine);
ForthResult forth_load_source(ForthMachine* machine, const char* source);
ForthResult forth_record_line(ForthMachine* machine, const char* line);
ForthResult forth_append_source_line(ForthMachine* machine, const char* line);
int forth_should_persist_line(const ForthMachine* machine, const char* line);
const char* forth_source(const ForthMachine* machine);
ForthResult forth_eval_line(
    ForthMachine* machine,
    const char* line,
    char* output,
    size_t output_size);

#endif
