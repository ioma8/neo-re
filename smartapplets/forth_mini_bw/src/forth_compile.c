#include "forth_compile.h"
#include "forth_dict.h"
#include "forth_util.h"
#include "forth_vm.h"

enum { PATCH_IF = 1, PATCH_ELSE = 2, PATCH_BEGIN = 3, PATCH_WHILE = 4 };

static ForthResult ok_result(void) {
    ForthResult result = {FORTH_OK, {0}};
    return result;
}

static ForthResult error_result(ForthCode code, const char* message) {
    ForthResult result = {code, {0}};
    forth_strncpy(result.message, message, FORTH_MAX_MESSAGE - 1);
    return result;
}

static int next_token(const char** cursor, char* token) {
    size_t len = 0;
    while(**cursor && forth_isspace(**cursor)) (*cursor)++;
    while(**cursor && !forth_isspace(**cursor) && len < FORTH_MAX_TOKEN) {
        token[len++] = *(*cursor)++;
    }
    token[len] = '\0';
    return len != 0;
}

static ForthResult emit(ForthMachine* machine, uint8_t op, int16_t arg) {
    if(machine->code_len >= FORTH_MAX_CODE) return error_result(FORTH_CODE_OVERFLOW, "code full");
    machine->code[machine->code_len].op = op;
    machine->code[machine->code_len].arg = arg;
    machine->code_len++;
    return ok_result();
}

static ForthResult patch_push(ForthMachine* machine, uint8_t kind, uint16_t patch_ip, uint16_t begin_ip) {
    if(machine->patch_depth >= FORTH_MAX_PATCH) return error_result(FORTH_BAD_CONTROL_FLOW, "patch full");
    machine->patches[machine->patch_depth++] = (ForthPatch){kind, patch_ip, begin_ip};
    return ok_result();
}

static int is_control_word(const char* token) {
    return forth_strcmp(token, "if") == 0 || forth_strcmp(token, "else") == 0 ||
           forth_strcmp(token, "then") == 0 || forth_strcmp(token, "begin") == 0 ||
           forth_strcmp(token, "until") == 0 || forth_strcmp(token, "while") == 0 ||
           forth_strcmp(token, "repeat") == 0;
}

static ForthResult handle_control(ForthMachine* machine, const char* token) {
    ForthPatch patch;
    if(forth_strcmp(token, "if") == 0) {
        return emit(machine, FORTH_OP_JUMP_IF_ZERO, 0).code == FORTH_OK
                   ? patch_push(machine, PATCH_IF, machine->code_len - 1, 0)
                   : error_result(FORTH_CODE_OVERFLOW, "code full");
    }
    if(forth_strcmp(token, "begin") == 0) {
        return patch_push(machine, PATCH_BEGIN, machine->code_len, machine->code_len);
    }
    if(machine->patch_depth == 0) return error_result(FORTH_BAD_CONTROL_FLOW, "bad control flow");
    patch = machine->patches[--machine->patch_depth];
    if(forth_strcmp(token, "else") == 0 && patch.kind == PATCH_IF) {
        machine->code[patch.patch_ip].arg = (int16_t)(machine->code_len + 1);
        return emit(machine, FORTH_OP_JUMP, 0).code == FORTH_OK
                   ? patch_push(machine, PATCH_ELSE, machine->code_len - 1, 0)
                   : error_result(FORTH_CODE_OVERFLOW, "code full");
    }
    if(forth_strcmp(token, "then") == 0 && (patch.kind == PATCH_IF || patch.kind == PATCH_ELSE)) {
        machine->code[patch.patch_ip].arg = (int16_t)machine->code_len;
        return ok_result();
    }
    if(forth_strcmp(token, "until") == 0 && patch.kind == PATCH_BEGIN) {
        return emit(machine, FORTH_OP_JUMP_IF_ZERO, (int16_t)patch.begin_ip);
    }
    if(forth_strcmp(token, "while") == 0 && patch.kind == PATCH_BEGIN) {
        machine->patches[machine->patch_depth++] = patch;
        return emit(machine, FORTH_OP_JUMP_IF_ZERO, 0).code == FORTH_OK
                   ? patch_push(machine, PATCH_WHILE, machine->code_len - 1, patch.begin_ip)
                   : error_result(FORTH_CODE_OVERFLOW, "code full");
    }
    if(forth_strcmp(token, "repeat") == 0 && patch.kind == PATCH_WHILE && machine->patch_depth != 0) {
        ForthPatch begin = machine->patches[--machine->patch_depth];
        if(begin.kind != PATCH_BEGIN) return error_result(FORTH_BAD_CONTROL_FLOW, "bad control flow");
        machine->code[patch.patch_ip].arg = (int16_t)(machine->code_len + 1);
        return emit(machine, FORTH_OP_JUMP, (int16_t)begin.begin_ip);
    }
    return error_result(FORTH_BAD_CONTROL_FLOW, "bad control flow");
}

static ForthResult run_token(ForthMachine* machine, const char* token, char* output, size_t output_size) {
    int16_t value;
    int16_t word_index;
    if(machine->expect_name) {
        machine->expect_name = 0;
        machine->compile_mode = 1;
        return forth_add_user_word(machine, token);
    }
    if(forth_strcmp(token, ":") == 0) {
        machine->expect_name = 1;
        return ok_result();
    }
    if(forth_strcmp(token, ";") == 0) {
        machine->compile_mode = 0;
        return emit(machine, FORTH_OP_EXIT, 0);
    }
    if(machine->compile_mode && is_control_word(token)) return handle_control(machine, token);
    if(forth_parse_i32(token, &value)) {
        if(machine->compile_mode) {
            return emit(machine, FORTH_OP_PUSH, (int16_t)value);
        }
        if(machine->depth >= FORTH_MAX_STACK) {
            return error_result(FORTH_STACK_OVERFLOW, "stack overflow");
        }
        machine->stack[machine->depth++] = value;
        return ok_result();
    }
    word_index = forth_find_word(machine, token);
    if(word_index < 0) return error_result(FORTH_UNKNOWN_WORD, "unknown word");
    if(machine->compile_mode) {
        ForthWord* word = &machine->words[word_index];
        return emit(machine, word->kind == 1 ? word->opcode : FORTH_OP_CALL, (int16_t)word_index);
    }
    if(machine->words[word_index].kind == 1) {
        return forth_exec_builtin(machine, machine->words[word_index].opcode, output, output_size);
    }
    return forth_exec_user_word(machine, (uint16_t)word_index, output, output_size);
}

ForthResult forth_compile_text(
    ForthMachine* machine,
    const char* text,
    char* output,
    size_t output_size) {
    char token[FORTH_MAX_TOKEN + 1];
    char result_storage[sizeof(ForthResult) + 1];
    ForthResult* result =
        (ForthResult*)((((unsigned long)result_storage) + 1UL) & ~1UL);
    const char* cursor = text;
    while(next_token(&cursor, token)) {
        *result = run_token(machine, token, output, output_size);
        if(result->code != FORTH_OK) return *result;
    }
    if(machine->expect_name || machine->patch_depth != 0) {
        return error_result(FORTH_BAD_CONTROL_FLOW, "incomplete compile");
    }
    return ok_result();
}
