#include "os3k.h"

enum { INPUT_CAPACITY = 32 };
enum { LINE_WIDTH = 28 };
enum { OUTPUT_LINES = 3 };
enum { STACK_CAPACITY = 16 };

enum {
    EVAL_OK = 0,
    EVAL_DIVIDE_BY_ZERO = 1,
    EVAL_INPUT_FULL = 2,
    EVAL_STACK_OVERFLOW = 3,
    EVAL_STACK_UNDERFLOW = 4,
    EVAL_UNKNOWN_WORD = 5,
};

typedef struct {
    int32_t stack[STACK_CAPACITY];
    uint32_t depth;
    uint8_t input[INPUT_CAPACITY];
    uint32_t input_len;
    uint32_t pending_output_kind;
    int32_t pending_output_value;
    uint8_t transcript_1[LINE_WIDTH];
    uint8_t transcript_2[LINE_WIDTH];
    uint8_t transcript_3[LINE_WIDTH];
} Repl_t;

void _OS3K_ClearScreen();
void _OS3K_SetCursor(uint8_t row, uint8_t col, CursorMode_e cursor_mode);

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

static inline Repl_t* State(void) {
    register char* a5 __asm__("a5");
    return (Repl_t*)(a5 + 0x300);
}

static void ClearLine(uint8_t* line) {
    for(uint32_t i = 0; i < LINE_WIDTH; i++) {
        line[i] = ' ';
    }
}

static void InitRepl(Repl_t* repl) {
    uint8_t* bytes = (uint8_t*)repl;
    uint32_t i;
    for(i = 0; i < sizeof(*repl); i++) {
        bytes[i] = 0;
    }
    ClearLine(repl->transcript_1);
    ClearLine(repl->transcript_2);
    ClearLine(repl->transcript_3);
}

static void ShiftTranscripts(Repl_t* repl) {
    uint8_t temp[LINE_WIDTH];
    uint32_t i;
    for(i = 0; i < LINE_WIDTH; i++) {
        temp[i] = repl->transcript_2[i];
    }
    for(i = 0; i < LINE_WIDTH; i++) {
        repl->transcript_1[i] = temp[i];
    }
    for(i = 0; i < LINE_WIDTH; i++) {
        repl->transcript_2[i] = repl->transcript_3[i];
    }
}

static int IsBlank(uint8_t byte) {
    return byte == ' ' || byte == 0;
}

static uint32_t TrimmedLen(const uint8_t* line) {
    uint32_t len = LINE_WIDTH;
    while(len > 0 && IsBlank(line[len - 1])) {
        len--;
    }
    return len;
}

static uint32_t InputTrimmedLen(const Repl_t* repl) {
    uint32_t len = repl->input_len < LINE_WIDTH ? repl->input_len : LINE_WIDTH;
    while(len > 0 && repl->input[len - 1] == ' ') {
        len--;
    }
    return len;
}

static uint32_t UDiv32(uint32_t numerator, uint32_t denominator);
static uint32_t UMod32(uint32_t numerator, uint32_t denominator);

static void DrawLine(uint8_t row, const uint8_t* line) {
    char text[LINE_WIDTH + 1];
    for(uint32_t i = 0; i < LINE_WIDTH; i++) {
        text[i] = (char)line[i];
    }
    text[LINE_WIDTH] = '\0';
    _OS3K_SetCursor(row, 1, CURSOR_MODE_HIDE);
    PutStringRaw(text);
}

static void Draw(Repl_t* repl) {
    uint8_t prompt[LINE_WIDTH];
    ClearLine(prompt);
    for(uint32_t i = 0; i < repl->input_len && i < LINE_WIDTH; i++) {
        prompt[i] = repl->input[i];
    }
    _OS3K_ClearScreen();
    SetCursorMode(CURSOR_MODE_HIDE);
    DrawLine(1, repl->transcript_1);
    DrawLine(2, repl->transcript_2);
    DrawLine(3, repl->transcript_3);
    DrawLine(4, prompt);
}

static uint32_t WriteU32(uint8_t* line, uint32_t offset, uint32_t value) {
    uint8_t digits[10];
    uint32_t count = 0;
    if(offset >= LINE_WIDTH) {
        return offset;
    }
    if(value == 0) {
        line[offset] = '0';
        return offset + 1;
    }
    while(value != 0 && count < sizeof(digits)) {
        digits[count++] = (uint8_t)('0' + UMod32(value, 10u));
        value = UDiv32(value, 10u);
    }
    while(count > 0 && offset < LINE_WIDTH) {
        line[offset++] = digits[--count];
    }
    return offset;
}

static uint32_t I32MagnitudeU32(int32_t value) {
    if(value >= 0) {
        return (uint32_t)value;
    }
    return (uint32_t)(-(value + 1)) + 1;
}

static int32_t MulI32Wrapping(int32_t lhs, int32_t rhs) {
    uint32_t a = (uint32_t)lhs;
    uint32_t b = (uint32_t)rhs;
    uint32_t result = 0;
    while(b != 0) {
        if((b & 1u) != 0) {
            result = result + a;
        }
        a <<= 1;
        b >>= 1;
    }
    return (int32_t)result;
}

static void UDivMod32(uint32_t numerator, uint32_t denominator, uint32_t* quotient, uint32_t* remainder) {
    uint32_t q = 0;
    uint32_t r = 0;
    uint32_t bit = 32;
    if(denominator == 0) {
        *quotient = 0;
        *remainder = numerator;
        return;
    }
    while(bit != 0) {
        bit--;
        r <<= 1;
        r |= (numerator >> bit) & 1u;
        if(r >= denominator) {
            r -= denominator;
            q |= 1u << bit;
        }
    }
    *quotient = q;
    *remainder = r;
}

static uint32_t UDiv32(uint32_t numerator, uint32_t denominator) {
    uint32_t quotient;
    uint32_t remainder;
    UDivMod32(numerator, denominator, &quotient, &remainder);
    (void)remainder;
    return quotient;
}

static uint32_t UMod32(uint32_t numerator, uint32_t denominator) {
    uint32_t quotient;
    uint32_t remainder;
    UDivMod32(numerator, denominator, &quotient, &remainder);
    (void)quotient;
    return remainder;
}

static int32_t DivI32Wrapping(int32_t lhs, int32_t rhs) {
    uint32_t quotient, remainder;
    int negative = ((lhs < 0) ^ (rhs < 0));
    (void)remainder;
    UDivMod32(I32MagnitudeU32(lhs), I32MagnitudeU32(rhs), &quotient, &remainder);
    if(negative) {
        return (int32_t)(0u - quotient);
    }
    return (int32_t)quotient;
}

static int32_t RemI32Wrapping(int32_t lhs, int32_t rhs) {
    uint32_t quotient, remainder;
    int negative = lhs < 0;
    (void)quotient;
    UDivMod32(I32MagnitudeU32(lhs), I32MagnitudeU32(rhs), &quotient, &remainder);
    if(negative) {
        return (int32_t)(0u - remainder);
    }
    return (int32_t)remainder;
}

static uint32_t WriteI32(uint8_t* line, uint32_t offset, int32_t value) {
    uint32_t divisor = 1000000000u;
    uint32_t remainder = I32MagnitudeU32(value);
    int started = 0;
    if(offset >= LINE_WIDTH) {
        return offset;
    }
    if(value == 0) {
        line[offset] = '0';
        return offset + 1;
    }
    if(value < 0 && offset < LINE_WIDTH) {
        line[offset++] = '-';
    }
    while(divisor != 0 && offset < LINE_WIDTH) {
        uint8_t digit = 0;
        while(remainder >= divisor) {
            remainder -= divisor;
            digit++;
        }
        if(digit != 0 || started || divisor == 1) {
            line[offset++] = (uint8_t)('0' + digit);
            started = 1;
        }
        divisor = UDiv32(divisor, 10u);
    }
    return offset;
}

static uint32_t WriteStackSummaryFrom(const Repl_t* repl, uint8_t* line, uint32_t offset) {
    uint32_t next = offset;
    uint32_t start;
    if(next < LINE_WIDTH) {
        line[next++] = '<';
    }
    next = WriteU32(line, next, repl->depth);
    next = TrimmedLen(line);
    if(next < LINE_WIDTH) {
        line[next++] = '>';
    }
    if(next < LINE_WIDTH) {
        line[next++] = ' ';
    }
    start = repl->depth > 3 ? repl->depth - 3 : 0;
    while(start < repl->depth && next < LINE_WIDTH) {
        next = WriteI32(line, next, repl->stack[start]);
        next = TrimmedLen(line);
        if(next < LINE_WIDTH && start + 1 < repl->depth) {
            line[next++] = ' ';
        }
        start++;
    }
    return next;
}

static int TokenEq(const Repl_t* repl, uint32_t start, uint32_t end, const char* expected) {
    uint32_t i = 0;
    while(expected[i] != '\0') {
        if(start + i >= end || repl->input[start + i] != (uint8_t)expected[i]) {
            return 0;
        }
        i++;
    }
    return start + i == end;
}

static int ParseI32Slice(const Repl_t* repl, uint32_t start, uint32_t end, int32_t* value) {
    uint32_t index;
    uint32_t magnitude = 0;
    uint32_t limit;
    int negative;
    if(start >= end) {
        return 0;
    }
    negative = repl->input[start] == '-';
    index = start + (uint32_t)negative;
    if(index == end) {
        return 0;
    }
    limit = negative ? ((uint32_t)INT32_MAX + 1u) : (uint32_t)INT32_MAX;
    while(index < end) {
        uint8_t byte = repl->input[index];
        uint32_t digit;
        if(byte < '0' || byte > '9') {
            return 0;
        }
        digit = (uint32_t)(byte - '0');
        if(magnitude > UDiv32(limit, 10u)
                || (magnitude == UDiv32(limit, 10u) && digit > UMod32(limit, 10u))) {
            return 0;
        }
        magnitude = magnitude * 10u + digit;
        index++;
    }
    if(negative) {
        if(magnitude == (uint32_t)INT32_MAX + 1u) {
            *value = INT32_MIN;
        } else {
            *value = -(int32_t)magnitude;
        }
    } else {
        *value = (int32_t)magnitude;
    }
    return 1;
}

static int NextTokenBounds(const Repl_t* repl, uint32_t* cursor, uint32_t* start_out, uint32_t* end_out) {
    while(*cursor < repl->input_len && repl->input[*cursor] == ' ') {
        (*cursor)++;
    }
    *start_out = *cursor;
    while(*cursor < repl->input_len && repl->input[*cursor] != ' ') {
        (*cursor)++;
    }
    *end_out = *cursor;
    return *start_out != *end_out;
}

static uint32_t Push(Repl_t* repl, int32_t value) {
    if(repl->depth >= STACK_CAPACITY) {
        return EVAL_STACK_OVERFLOW;
    }
    repl->stack[repl->depth++] = value;
    return EVAL_OK;
}

static uint32_t Pop(Repl_t* repl, int32_t* value) {
    if(repl->depth == 0) {
        return EVAL_STACK_UNDERFLOW;
    }
    repl->depth--;
    *value = repl->stack[repl->depth];
    return EVAL_OK;
}

static uint32_t Add(Repl_t* repl) {
    int32_t lhs, rhs;
    uint32_t error = Pop(repl, &rhs);
    if(error != EVAL_OK) {
        return error;
    }
    error = Pop(repl, &lhs);
    if(error != EVAL_OK) {
        return error;
    }
    return Push(repl, lhs + rhs);
}

static uint32_t Sub(Repl_t* repl) {
    int32_t lhs, rhs;
    uint32_t error = Pop(repl, &rhs);
    if(error != EVAL_OK) {
        return error;
    }
    error = Pop(repl, &lhs);
    if(error != EVAL_OK) {
        return error;
    }
    return Push(repl, lhs - rhs);
}

static uint32_t Mul(Repl_t* repl) {
    int32_t lhs, rhs;
    uint32_t error = Pop(repl, &rhs);
    if(error != EVAL_OK) {
        return error;
    }
    error = Pop(repl, &lhs);
    if(error != EVAL_OK) {
        return error;
    }
    return Push(repl, MulI32Wrapping(lhs, rhs));
}

static uint32_t CheckedDiv(Repl_t* repl) {
    int32_t lhs, rhs;
    uint32_t error = Pop(repl, &rhs);
    if(error != EVAL_OK) {
        return error;
    }
    error = Pop(repl, &lhs);
    if(error != EVAL_OK) {
        return error;
    }
    if(rhs == 0) {
        return EVAL_DIVIDE_BY_ZERO;
    }
    return Push(repl, DivI32Wrapping(lhs, rhs));
}

static uint32_t CheckedMod(Repl_t* repl) {
    int32_t lhs, rhs;
    uint32_t error = Pop(repl, &rhs);
    if(error != EVAL_OK) {
        return error;
    }
    error = Pop(repl, &lhs);
    if(error != EVAL_OK) {
        return error;
    }
    if(rhs == 0) {
        return EVAL_DIVIDE_BY_ZERO;
    }
    return Push(repl, RemI32Wrapping(lhs, rhs));
}

static uint32_t Dup(Repl_t* repl) {
    if(repl->depth == 0) {
        return EVAL_STACK_UNDERFLOW;
    }
    return Push(repl, repl->stack[repl->depth - 1]);
}

static uint32_t DropTop(Repl_t* repl) {
    int32_t value;
    return Pop(repl, &value);
}

static uint32_t Swap(Repl_t* repl) {
    int32_t top, next;
    if(repl->depth < 2) {
        return EVAL_STACK_UNDERFLOW;
    }
    top = repl->stack[repl->depth - 1];
    next = repl->stack[repl->depth - 2];
    repl->stack[repl->depth - 1] = next;
    repl->stack[repl->depth - 2] = top;
    return EVAL_OK;
}

static uint32_t Over(Repl_t* repl) {
    if(repl->depth < 2) {
        return EVAL_STACK_UNDERFLOW;
    }
    return Push(repl, repl->stack[repl->depth - 2]);
}

static uint32_t PopToPendingOutput(Repl_t* repl) {
    uint32_t error;
    if(repl->depth == 0) {
        return EVAL_STACK_UNDERFLOW;
    }
    error = Pop(repl, &repl->pending_output_value);
    if(error != EVAL_OK) {
        return error;
    }
    repl->pending_output_kind = 1;
    return EVAL_OK;
}

static void WriteLiteral(uint8_t* line, uint32_t offset, const char* text) {
    uint32_t i = 0;
    while(text[i] != '\0' && offset < LINE_WIDTH) {
        line[offset++] = (uint8_t)text[i++];
    }
}

static void WriteErrorTranscript(Repl_t* repl, uint32_t command_len, uint32_t error) {
    uint32_t offset = 0;
    ClearLine(repl->transcript_3);
    while(offset < command_len && offset < LINE_WIDTH) {
        repl->transcript_3[offset] = repl->input[offset];
        offset++;
    }
    if(offset < LINE_WIDTH) {
        repl->transcript_3[offset++] = ' ';
    }
    switch(error) {
        case EVAL_DIVIDE_BY_ZERO: WriteLiteral(repl->transcript_3, offset, "divide by zero"); break;
        case EVAL_INPUT_FULL: WriteLiteral(repl->transcript_3, offset, "input full"); break;
        case EVAL_STACK_OVERFLOW: WriteLiteral(repl->transcript_3, offset, "stack overflow"); break;
        case EVAL_STACK_UNDERFLOW: WriteLiteral(repl->transcript_3, offset, "stack underflow"); break;
        default: WriteLiteral(repl->transcript_3, offset, "unknown word"); break;
    }
}

static void WriteSuccessTranscript(Repl_t* repl, uint32_t command_len) {
    uint32_t offset = 0;
    ClearLine(repl->transcript_3);
    while(offset < command_len && offset < LINE_WIDTH) {
        repl->transcript_3[offset] = repl->input[offset];
        offset++;
    }
    switch(repl->pending_output_kind) {
        case 1:
            if(offset < LINE_WIDTH) {
                repl->transcript_3[offset++] = ' ';
            }
            WriteI32(repl->transcript_3, offset, repl->pending_output_value);
            offset = TrimmedLen(repl->transcript_3);
            break;
        case 2:
            if(offset < LINE_WIDTH) {
                repl->transcript_3[offset++] = ' ';
            }
            WriteStackSummaryFrom(repl, repl->transcript_3, offset);
            offset = TrimmedLen(repl->transcript_3);
            break;
        default:
            break;
    }
    if(offset < LINE_WIDTH) {
        repl->transcript_3[offset++] = ' ';
    }
    if(offset < LINE_WIDTH) {
        repl->transcript_3[offset++] = ' ';
    }
    if(offset < LINE_WIDTH) {
        repl->transcript_3[offset++] = 'o';
    }
    if(offset < LINE_WIDTH) {
        repl->transcript_3[offset++] = 'k';
    }
}

static uint32_t EvalToken(Repl_t* repl, uint32_t start, uint32_t end) {
    int32_t value;
    if(ParseI32Slice(repl, start, end, &value)) {
        return Push(repl, value);
    }
    if(TokenEq(repl, start, end, "+") || TokenEq(repl, start, end, "=")) {
        return Add(repl);
    }
    if(TokenEq(repl, start, end, "-")) {
        return Sub(repl);
    }
    if(TokenEq(repl, start, end, "*")) {
        return Mul(repl);
    }
    if(TokenEq(repl, start, end, "/")) {
        return CheckedDiv(repl);
    }
    if(TokenEq(repl, start, end, "mod")) {
        return CheckedMod(repl);
    }
    if(TokenEq(repl, start, end, "dup")) {
        return Dup(repl);
    }
    if(TokenEq(repl, start, end, "drop")) {
        return DropTop(repl);
    }
    if(TokenEq(repl, start, end, "swap")) {
        return Swap(repl);
    }
    if(TokenEq(repl, start, end, "over")) {
        return Over(repl);
    }
    if(TokenEq(repl, start, end, ".")) {
        return PopToPendingOutput(repl);
    }
    if(TokenEq(repl, start, end, ".s")) {
        repl->pending_output_kind = 2;
        return EVAL_OK;
    }
    if(TokenEq(repl, start, end, "clear")) {
        repl->depth = 0;
        return EVAL_OK;
    }
    return EVAL_UNKNOWN_WORD;
}

static void Enter(Repl_t* repl) {
    uint32_t cursor = 0;
    uint32_t start, end;
    uint32_t error;
    uint32_t command_len;
    if(repl->input_len == 0) {
        return;
    }
    ShiftTranscripts(repl);
    ClearLine(repl->transcript_3);
    repl->pending_output_kind = 0;
    repl->pending_output_value = 0;
    while(NextTokenBounds(repl, &cursor, &start, &end)) {
        error = EvalToken(repl, start, end);
        if(error != EVAL_OK) {
            command_len = InputTrimmedLen(repl);
            WriteErrorTranscript(repl, command_len, error);
            repl->input_len = 0;
            return;
        }
    }
    command_len = InputTrimmedLen(repl);
    WriteSuccessTranscript(repl, command_len);
    repl->input_len = 0;
}

static void AcceptPrintable(Repl_t* repl, uint8_t byte) {
    if(byte < ' ' || byte > '~') {
        return;
    }
    if(repl->input_len >= INPUT_CAPACITY) {
        return;
    }
    repl->input[repl->input_len++] = byte;
}

static void Backspace(Repl_t* repl) {
    if(repl->input_len > 0) {
        repl->input_len--;
    }
}

static void HandleChar(Repl_t* repl, uint32_t param) {
    uint8_t byte = (uint8_t)(param & 0xff);
    if(byte == '\r' || byte == '\n') {
        Enter(repl);
    } else if(byte == 0x08 || byte == 0x7f) {
        Backspace(repl);
    } else {
        AcceptPrintable(repl, byte);
    }
}

static void HandleKey(Repl_t* repl, uint32_t param, uint32_t* status) {
    uint32_t key = param & 0xff;
    switch(key) {
        case KEY_APPLETS:
            *status = 0x07;
            break;
        case KEY_BACKSPACE:
            Backspace(repl);
            break;
        default:
            break;
    }
}

void alpha_neo_process_message(uint32_t message, uint32_t param, uint32_t* status) {
    Repl_t* repl = State();
    *status = 0;
    switch(message) {
        case MSG_SETFOCUS:
            InitRepl(repl);
            Draw(repl);
            break;
        case MSG_CHAR:
            HandleChar(repl, param);
            Draw(repl);
            break;
        case MSG_KEY:
            HandleKey(repl, param, status);
            if(*status == 0) {
                Draw(repl);
            }
            break;
        default:
            *status = 0x04;
            break;
    }
}
