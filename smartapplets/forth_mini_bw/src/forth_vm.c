#include "forth_dict.h"
#include "forth_util.h"
#include "forth_vm.h"

static ForthResult ok_result(void) {
    ForthResult result = {FORTH_OK, {0}};
    return result;
}

static ForthResult error_result(ForthCode code, const char* message) {
    ForthResult result = {code, {0}};
    forth_strncpy(result.message, message, FORTH_MAX_MESSAGE - 1);
    return result;
}

static ForthResult push(ForthMachine* machine, int16_t value) {
    if(machine->depth >= FORTH_MAX_STACK) {
        return error_result(FORTH_STACK_OVERFLOW, "stack overflow");
    }
    machine->stack[machine->depth++] = value;
    return ok_result();
}

static ForthResult pop(ForthMachine* machine, int16_t* value) {
    if(machine->depth == 0) {
        return error_result(FORTH_STACK_UNDERFLOW, "stack underflow");
    }
    *value = machine->stack[--machine->depth];
    return ok_result();
}

static ForthResult append_output(char* output, size_t size, int16_t value) {
    if(!output || size == 0) return ok_result();
    if(forth_append_i32(output, size, value)) return ok_result();
    return error_result(FORTH_OUTPUT_FULL, "output full");
}

static uint16_t udiv16(uint16_t numer, uint16_t denom, uint16_t* rem) {
    uint16_t q = 0;
    uint16_t r = 0;
    uint16_t bit;
    for(bit = 0; bit < 16; bit++) {
        r = (uint16_t)((r << 1) | ((numer >> (15 - bit)) & 1u));
        q <<= 1;
        if(r >= denom) {
            r = (uint16_t)(r - denom);
            q |= 1u;
        }
    }
    if(rem) *rem = r;
    return q;
}

static int16_t div16(int16_t numer, int16_t denom) {
    uint16_t rem;
    int negative = ((numer < 0) ^ (denom < 0)) != 0;
    uint16_t left = (uint16_t)(numer < 0 ? -numer : numer);
    uint16_t right = (uint16_t)(denom < 0 ? -denom : denom);
    int16_t quotient = (int16_t)udiv16(left, right, &rem);
    return negative ? (int16_t)-quotient : quotient;
}

static int16_t mod16(int16_t numer, int16_t denom) {
    uint16_t rem;
    uint16_t left = (uint16_t)(numer < 0 ? -numer : numer);
    uint16_t right = (uint16_t)(denom < 0 ? -denom : denom);
    int16_t result;
    (void)udiv16(left, right, &rem);
    result = (int16_t)rem;
    return numer < 0 ? (int16_t)-result : result;
}

ForthResult forth_exec_builtin(
    ForthMachine* machine,
    uint8_t opcode,
    char* output,
    size_t output_size) {
    int16_t a, b;
    switch(opcode) {
        case FORTH_OP_DUP:
            return machine->depth ? push(machine, machine->stack[machine->depth - 1])
                                  : error_result(FORTH_STACK_UNDERFLOW, "stack underflow");
        case FORTH_OP_DROP: return pop(machine, &a);
        case FORTH_OP_SWAP:
            if(machine->depth < 2) return error_result(FORTH_STACK_UNDERFLOW, "stack underflow");
            a = machine->stack[machine->depth - 1];
            machine->stack[machine->depth - 1] = machine->stack[machine->depth - 2];
            machine->stack[machine->depth - 2] = a;
            return ok_result();
        case FORTH_OP_OVER:
            return machine->depth >= 2 ? push(machine, machine->stack[machine->depth - 2])
                                       : error_result(FORTH_STACK_UNDERFLOW, "stack underflow");
        case FORTH_OP_CLEAR:
            machine->depth = 0;
            return ok_result();
        case FORTH_OP_DOT:
            if(pop(machine, &a).code != FORTH_OK) return error_result(FORTH_STACK_UNDERFLOW, "stack underflow");
            return append_output(output, output_size, a);
        default: break;
    }
    if(pop(machine, &b).code != FORTH_OK || pop(machine, &a).code != FORTH_OK) {
        return error_result(FORTH_STACK_UNDERFLOW, "stack underflow");
    }
    switch(opcode) {
        case FORTH_OP_ADD: return push(machine, a + b);
        case FORTH_OP_SUB: return push(machine, a - b);
        case FORTH_OP_MUL: return push(machine, a * b);
        case FORTH_OP_DIV:
            if(b == 0) return error_result(FORTH_DIVIDE_BY_ZERO, "divide by zero");
            return push(machine, div16(a, b));
        case FORTH_OP_MOD:
            if(b == 0) return error_result(FORTH_DIVIDE_BY_ZERO, "divide by zero");
            return push(machine, mod16(a, b));
        case FORTH_OP_LESS: return push(machine, a < b ? -1 : 0);
        default: return error_result(FORTH_UNKNOWN_WORD, "bad opcode");
    }
}

ForthResult forth_exec_user_word(
    ForthMachine* machine,
    uint16_t word_index,
    char* output,
    size_t output_size) {
    uint16_t call_stack[FORTH_MAX_RETURN];
    uint16_t call_depth = 0;
    uint16_t ip = machine->words[word_index].entry;
    while(ip < machine->code_len) {
        ForthInstr instr = machine->code[ip++];
        ForthResult result = ok_result();
        if(instr.op == FORTH_OP_PUSH) result = push(machine, instr.arg);
        else if(instr.op == FORTH_OP_CALL) {
            if(call_depth >= FORTH_MAX_RETURN) return error_result(FORTH_STACK_OVERFLOW, "return overflow");
            call_stack[call_depth++] = ip;
            ip = machine->words[(uint16_t)instr.arg].entry;
            continue;
        } else if(instr.op == FORTH_OP_JUMP_IF_ZERO) {
            int16_t flag;
            result = pop(machine, &flag);
            if(result.code == FORTH_OK && flag == 0) ip = (uint16_t)instr.arg;
        } else if(instr.op == FORTH_OP_JUMP) {
            ip = (uint16_t)instr.arg;
        } else if(instr.op == FORTH_OP_EXIT) {
            if(call_depth == 0) return ok_result();
            ip = call_stack[--call_depth];
        } else {
            result = forth_exec_builtin(machine, instr.op, output, output_size);
        }
        if(result.code != FORTH_OK) return result;
    }
    return ok_result();
}
