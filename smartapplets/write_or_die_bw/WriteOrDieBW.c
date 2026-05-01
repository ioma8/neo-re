#include "../betawise-sdk/applet.h"
#include "os3k.h"

typedef struct {
    uint32_t marker;
} AppState_t;

APPLET_ENTRY(alpha_neo_process_message);
APPLET_STATE(AppState_t);

void _OS3K_ClearScreen(void);

void alpha_neo_process_message(uint32_t message, uint32_t param, uint32_t* status) {
    AppState_t* state = State();
    *status = 0;
    if(message == MSG_SETFOCUS) {
        state->marker = 0x574F4431;
        _OS3K_ClearScreen();
        PutStringRaw("WriteOrDie");
    } else if(message == MSG_KEY && ((param & 0xff) == KEY_APPLETS || (param & 0xff) == KEY_ESC)) {
        *status = APPLET_EXIT_STATUS;
    } else {
        *status = APPLET_UNHANDLED_STATUS;
    }
}
