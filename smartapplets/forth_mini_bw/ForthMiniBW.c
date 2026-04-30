#include "app_state.h"
#include "app_storage.h"
#include "src/forth_util.h"

asm(
    ".section .text.alpha_usb_entry,\"ax\"\n"
    ".global alpha_usb_entry\n"
    "alpha_usb_entry:\n"
    "move.l 12(%sp),-(%sp)\n"
    "move.l 12(%sp),-(%sp)\n"
    "move.l 12(%sp),-(%sp)\n"
    "bsr alpha_neo_process_message\n"
    "lea 12(%sp),%sp\n"
    "rts\n"
    ".text\n"
);

static void accept_printable(AppState* state, uint32_t param) {
    char byte = (char)(param & 0xff);
    if(byte < ' ' || byte > '~' || state->input_len >= INPUT_CAPACITY) return;
    state->input[state->input_len++] = byte;
    state->input[state->input_len] = '\0';
}

static void backspace(AppState* state) {
    if(state->input_len == 0) return;
    state->input[--state->input_len] = '\0';
}

static void reload_from_storage(AppState* state) {
    (void)storage_load_machine(&state->machine);
    state->storage_loaded = 1;
}

static void ensure_storage_loaded(AppState* state) {
    if(!state->storage_loaded) {
        reload_from_storage(state);
    }
}

static void commit_line(AppState* state) {
    ForthResult result;
    char output[128] = {0};
    const char* command;
    if(state->input_len == 0) return;
    command = state->input;
    result = forth_eval_line(&state->machine, command, output, sizeof(output));
    if(result.code == FORTH_OK) {
        if(forth_should_persist_line(&state->machine, command)) {
            result = forth_append_source_line(&state->machine, command);
            if(result.code == FORTH_OK && !storage_save_machine(&state->machine)) {
                result.code = FORTH_SOURCE_FULL;
                forth_strcpy(result.message, "save failed");
            }
        }
    }
    app_push_result(state, command, &result, output);
    state->input_len = 0;
    state->input[0] = '\0';
}

static void handle_char(AppState* state, uint32_t param) {
    char byte = (char)(param & 0xff);
    ensure_storage_loaded(state);
    if(byte == '\r' || byte == '\n') commit_line(state);
    else if(byte == '\b' || byte == 0x7f) backspace(state);
    else accept_printable(state, param);
}

static void handle_key(AppState* state, uint32_t param, uint32_t* status) {
    switch(param & 0xff) {
        case KEY_APPLETS:
            *status = 0x07;
            break;
        case KEY_BACKSPACE:
            backspace(state);
            break;
        default:
            break;
    }
}

void alpha_neo_process_message(uint32_t message, uint32_t param, uint32_t* status) {
    AppState* state = State();
    *status = 0;
    switch(message) {
        case MSG_SETFOCUS:
            app_reset(state);
            app_draw(state);
            break;
        case MSG_CHAR:
            handle_char(state, param);
            app_draw(state);
            break;
        case MSG_KEY:
            handle_key(state, param, status);
            if(*status == 0) app_draw(state);
            break;
        default:
            *status = 0x04;
            break;
    }
}
