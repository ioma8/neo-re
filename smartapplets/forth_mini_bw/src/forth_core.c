#include "forth_compile.h"
#include "forth_dict.h"
#include "forth_util.h"

static ForthResult source_full(void) {
    ForthResult result = {FORTH_SOURCE_FULL, "source full"};
    return result;
}

static ForthResult load_error(ForthCode code, const char* message) {
    ForthResult result = {code, {0}};
    forth_strncpy(result.message, message, FORTH_MAX_MESSAGE - 1);
    return result;
}

typedef union {
    ForthResult value;
    uint32_t align;
} ForthResultSlot;

int forth_should_persist_line(const ForthMachine* machine, const char* line) {
    const char* cursor = line;
    if(machine->compile_mode || machine->expect_name) {
        return 1;
    }
    while(*cursor) {
        if(*cursor == ':') {
            return 1;
        }
        cursor++;
    }
    return 0;
}

ForthResult forth_append_source_line(ForthMachine* machine, const char* line) {
    size_t len = forth_strlen(line);
    if(machine->source_len + len + 2 > FORTH_MAX_SOURCE) {
        return source_full();
    }
    forth_memcpy(machine->source + machine->source_len, line, len);
    machine->source_len += (uint16_t)len;
    machine->source[machine->source_len++] = '\n';
    machine->source[machine->source_len] = '\0';
    return (ForthResult){FORTH_OK, {0}};
}

void forth_init(ForthMachine* machine) {
    forth_memset(machine, 0, sizeof(*machine));
    forth_seed_builtins(machine);
}

ForthResult forth_load_source(ForthMachine* machine, const char* source) {
    char line[128];
    ForthResultSlot result;
    size_t line_len = 0;
    size_t len = forth_strlen(source);
    const char* cursor = source;
    if(len + 1 > FORTH_MAX_SOURCE) {
        return source_full();
    }
    while(*cursor) {
        char ch = *cursor++;
        if(ch == '\n') {
            if(line_len != 0) {
                line[line_len] = '\0';
                result.value = forth_compile_text(machine, line, 0, 0);
                if(result.value.code != FORTH_OK) {
                    return result.value;
                }
                line_len = 0;
            }
            continue;
        }
        if(line_len + 1 >= sizeof(line)) {
            return load_error(FORTH_BAD_DEFINITION, "line too long");
        }
        line[line_len++] = ch;
    }
    if(line_len != 0) {
        line[line_len] = '\0';
        result.value = forth_compile_text(machine, line, 0, 0);
        if(result.value.code != FORTH_OK) {
            return result.value;
        }
    }
    forth_memcpy(machine->source, source, len + 1);
    machine->source_len = (uint16_t)len;
    if(machine->expect_name || machine->patch_depth != 0) {
        return load_error(FORTH_BAD_CONTROL_FLOW, "incomplete compile");
    }
    return (ForthResult){FORTH_OK, {0}};
}

ForthResult forth_record_line(ForthMachine* machine, const char* line) {
    ForthResult result = forth_compile_text(machine, line, 0, 0);
    if(result.code != FORTH_OK) {
        return result;
    }
    return forth_append_source_line(machine, line);
}

const char* forth_source(const ForthMachine* machine) {
    return machine->source;
}

ForthResult forth_eval_line(
    ForthMachine* machine,
    const char* line,
    char* output,
    size_t output_size) {
    if(output_size != 0) {
        output[0] = '\0';
    }
    return forth_compile_text(machine, line, output, output_size);
}
