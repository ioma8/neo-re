#include "forth_compile.h"
#include "forth_dict.h"
#include "forth_util.h"

static ForthResult source_full(void) {
    ForthResult result = {FORTH_SOURCE_FULL, "source full"};
    return result;
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
    size_t len = forth_strlen(source);
    ForthResult result;
    if(len + 1 > FORTH_MAX_SOURCE) {
        return source_full();
    }
    result = forth_compile_text(machine, source, 0, 0);
    if(result.code == FORTH_OK) {
        forth_memcpy(machine->source, source, len + 1);
        machine->source_len = (uint16_t)len;
    }
    return result;
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
