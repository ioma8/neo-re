#include "alpha_export.h"

#include "../betawise-sdk/os3k.h"

enum {
    ALPHAWORD_APPLET_ID = 0xA000,
    WRITE_OR_DIE_APPLET_ID = 0xA133,
    CURRENT_APPLET_SLOT_STATE = 0x000035e2,
    CURRENT_APPLET_ID_STATE = 0x000035e6,
    CURRENT_APPLET_SLOT = 0x000035ec,
    APPLET_A5_TABLE = 0x0000355e,
    AW_NS1_RESET = 0x00010001,
    AW_NS1_PAYLOAD = 0x00010002,
    AW_NS2_RESET = 0x00020001,
    AW_NS2_PAYLOAD = 0x00020002,
    AW_NS2_READ_NEXT = 0x00020006,
    AW_STATUS_RESET = 0x11,
    AW_INTERNAL_CONTROL = 0xbc,
    AW_INTERNAL_TERMINATE = 0xbb,
    AW_SELECTOR_REBUILD = 0x83
};

static const uint8_t FILE_KEYS[8] = { 0x2d, 0x2c, 0x04, 0x0f, 0x0e, 0x0a, 0x01, 0x27 };
static const uint8_t FILE_KEY_BY_SLOT[8] = { KEY_FILE_1, KEY_FILE_2, KEY_FILE_3, KEY_FILE_4, KEY_FILE_5, KEY_FILE_6, KEY_FILE_7, KEY_FILE_8 };

typedef struct {
    uint32_t input_len;
    uint32_t input;
    uint32_t output_capacity;
    uint32_t output_count;
    uint32_t output;
} AlphaWordPacket_t;

typedef struct {
    uint32_t input_len;
    uint32_t input;
    uint32_t output_capacity;
    uint32_t output_count;
    uint32_t output;
} AlphaWordNamespace1Packet_t;

static uint32_t ReadLong(uint32_t address) {
    return *(volatile uint32_t*)address;
}

static void WriteLong(uint32_t address, uint32_t value) {
    *(volatile uint32_t*)address = value;
}

static uint32_t ReadBigLong(const uint8_t* bytes) {
    return ((uint32_t)bytes[0] << 24) | ((uint32_t)bytes[1] << 16) | ((uint32_t)bytes[2] << 8) | bytes[3];
}

static uint16_t ReadBigWord(const uint8_t* bytes) {
    return (uint16_t)(((uint16_t)bytes[0] << 8) | bytes[1]);
}

static void WriteBigLong(uint32_t address, uint32_t value) {
    volatile uint8_t* bytes = (volatile uint8_t*)address;
    bytes[0] = (uint8_t)(value >> 24);
    bytes[1] = (uint8_t)(value >> 16);
    bytes[2] = (uint8_t)(value >> 8);
    bytes[3] = (uint8_t)value;
}

static uint32_t AppletA5(uint8_t applet) {
    if(applet == 0) {
        return 0;
    }
    return ReadLong(APPLET_A5_TABLE + ((uint32_t)applet * 4u));
}

static uint32_t CurrentA5(void) {
    register uint32_t a5 __asm__("a5");
    return a5;
}

static void WriteBigLongToBytes(uint8_t* bytes, uint32_t value) {
    bytes[0] = (uint8_t)(value >> 24);
    bytes[1] = (uint8_t)(value >> 16);
    bytes[2] = (uint8_t)(value >> 8);
    bytes[3] = (uint8_t)value;
}

static void SetFirmwareCurrentApplet(uint8_t slot, uint16_t applet_id) {
    WriteLong(CURRENT_APPLET_SLOT_STATE, slot);
    WriteLong(CURRENT_APPLET_ID_STATE, (uint32_t)applet_id << 16);
    WriteLong(CURRENT_APPLET_SLOT, slot);
}

static bool SendCommand(uint8_t applet, uint32_t command, uint32_t param, uint32_t* status) {
    uint32_t local_status = 0;
    (void)AppletSendMessage(applet, (Message_e)command, param, &local_status);
    if(status != 0) {
        *status = local_status;
    }
    return true;
}

static bool SendAlphaWordKey(uint8_t applet, uint8_t key) {
    uint32_t status;
    return SendCommand(applet, MSG_KEY, key, &status);
}

static bool KeyForChar(char value, uint8_t* key, bool* shifted) {
    *shifted = false;
    if(value >= 'A' && value <= 'Z') {
        value = (char)(value - 'A' + 'a');
        *shifted = true;
    }
    if(value >= 'a' && value <= 'z') {
        static const uint8_t keys[26] = {
            KEY_A, KEY_B, KEY_C, KEY_D, KEY_E, KEY_F, KEY_G, KEY_H, KEY_I, KEY_J, KEY_K, KEY_L, KEY_M,
            KEY_N, KEY_O, KEY_P, KEY_Q, KEY_R, KEY_S, KEY_T, KEY_U, KEY_V, KEY_W, KEY_X, KEY_Y, KEY_Z
        };
        *key = keys[(uint8_t)(value - 'a')];
        return true;
    }
    switch(value) {
        case ' ': *key = KEY_SPACE; return true;
        case '0': *key = KEY_0; return true;
        case '1': *key = KEY_1; return true;
        case '2': *key = KEY_2; return true;
        case '3': *key = KEY_3; return true;
        case '4': *key = KEY_4; return true;
        case '5': *key = KEY_5; return true;
        case '6': *key = KEY_6; return true;
        case '7': *key = KEY_7; return true;
        case '8': *key = KEY_8; return true;
        case '9': *key = KEY_9; return true;
        case '.': *key = KEY_DOT; return true;
        case ',': *key = KEY_COMMA; return true;
        case ';': *key = KEY_SEMICOLON; return true;
        case ':': *key = KEY_SEMICOLON; *shifted = true; return true;
        case '-': *key = KEY_MINUS; return true;
        case '\'': *key = KEY_APOSTROPHE; return true;
        case '/': *key = KEY_SLASH; return true;
        default: return false;
    }
}

static bool SendAlphaWordCharAsKey(uint8_t applet, char value) {
    uint8_t key;
    bool shifted;
    if(value == '\r' || value == '\n') {
        return SendAlphaWordKey(applet, KEY_ENTER);
    }
    if(!KeyForChar(value, &key, &shifted)) {
        return true;
    }
    if(shifted) {
        SetModifierKeys(KEY_MOD_LEFTSHIFT);
    }
    bool ok = SendAlphaWordKey(applet, key);
    if(shifted) {
        SetModifierKeys(KEY_MOD_NONE);
    }
    return ok;
}

static void EnsureAlphaWordInitialized(uint8_t applet) {
    uint32_t status;
    SendCommand(applet, MSG_INIT, 0, &status);
    SendCommand(applet, MSG_SETFOCUS, 0, &status);
}

static bool SendPacketWithCount(uint8_t applet, uint32_t command, const uint8_t* input_data, uint32_t input_len, uint8_t* output, uint32_t output_len, uint32_t* status, uint32_t* output_count) {
    uint32_t scratch = CurrentA5() + 0x800u;
    AlphaWordPacket_t* packet = (AlphaWordPacket_t*)scratch;
    uint8_t* input = (uint8_t*)(scratch + 0x20u);
    uint8_t* packet_output = (uint8_t*)(scratch + 0x30u);
    if(input_len > 4u) {
        input_len = 4u;
    }
    for(uint32_t i = 0; i < 4u; i++) {
        input[i] = i < input_len && input_data != 0 ? input_data[i] : 0;
    }
    for(uint32_t i = 0; i < output_len; i++) {
        output[i] = 0;
        packet_output[i] = 0;
    }
    packet->input_len = input_len;
    packet->input = (uint32_t)input;
    packet->output_capacity = output_len;
    packet->output_count = 0;
    packet->output = (uint32_t)packet_output;
    bool ok = SendCommand(applet, command, (uint32_t)packet, status);
    for(uint32_t i = 0; i < output_len; i++) {
        output[i] = packet_output[i];
    }
    if(output_count != 0) {
        *output_count = packet->output_count;
    }
    return ok;
}

static bool SendPacket(uint8_t applet, uint32_t command, const uint8_t* input_data, uint32_t input_len, uint8_t* output, uint32_t output_len, uint32_t* status) {
    return SendPacketWithCount(applet, command, input_data, input_len, output, output_len, status, 0);
}

static bool SendSentinelWithCount(uint8_t applet, uint8_t sentinel, uint8_t* output, uint32_t output_len, uint32_t* status, uint32_t* output_count) {
    AlphaWordPacket_t packet;
    uint8_t input[4] = { sentinel, 0, 0, 0 };
    for(uint32_t i = 0; i < output_len; i++) {
        output[i] = 0;
    }
    packet.input_len = 1;
    packet.input = (uint32_t)input;
    packet.output_capacity = output_len;
    packet.output_count = 0;
    packet.output = (uint32_t)output;
    bool ok = SendCommand(applet, AW_NS2_PAYLOAD, (uint32_t)&packet, status);
    if(output_count != 0) {
        *output_count = packet.output_count;
    }
    return ok;
}

static bool SendSentinel(uint8_t applet, uint8_t sentinel, uint32_t* status) {
    uint8_t output[8];
    return SendSentinelWithCount(applet, sentinel, output, sizeof(output), status, 0);
}

static bool SendNamespace1Packet(uint8_t applet, uint32_t command, const uint8_t* input_data, uint32_t input_len, uint8_t* output, uint32_t output_len, uint32_t* status) {
    AlphaWordNamespace1Packet_t packet;
    uint8_t input[4];
    for(uint32_t i = 0; i < sizeof(input); i++) {
        input[i] = i < input_len ? input_data[i] : 0;
    }
    for(uint32_t i = 0; i < output_len; i++) {
        output[i] = 0;
    }
    packet.input_len = input_len;
    packet.input = (uint32_t)input;
    packet.output_capacity = output_len;
    packet.output_count = 0;
    packet.output = (uint32_t)output;
    return SendCommand(applet, command, (uint32_t)&packet, status);
}

static bool SendPayload(uint8_t applet, uint8_t byte, uint32_t* status) {
    uint8_t output[8];
    return SendPacket(applet, AW_NS2_PAYLOAD, &byte, 1, output, sizeof(output), status);
}

static bool SendControl(uint8_t applet, uint8_t selector) {
    uint32_t status;
    return SendSentinel(applet, AW_INTERNAL_CONTROL, &status) &&
           SendSentinel(applet, selector, &status);
}

static bool QueryControl(uint8_t applet, uint8_t selector, uint8_t* output, uint32_t output_len, uint32_t* output_count, uint32_t* status) {
    if(!SendSentinel(applet, AW_INTERNAL_CONTROL, status)) {
        return false;
    }
    return SendSentinelWithCount(applet, selector, output, output_len, status, output_count);
}

static bool SelectAlphaWordSlot(uint8_t applet, uint32_t slot) {
    if(slot < 1 || slot > 8) {
        return false;
    }
    if(!SendControl(applet, (uint8_t)slot)) {
        return false;
    }
    return SendControl(applet, AW_SELECTOR_REBUILD);
}

static uint32_t QuerySelectedSlotStatus(uint8_t applet) {
    uint8_t output[4];
    uint32_t count = 0;
    uint32_t status = 0;
    if(!QueryControl(applet, 0x88, output, sizeof(output), &count, &status)) {
        return 0x79010000u | (status & 0xffffu);
    }
    if(count == 0) {
        return 0x79020000u | (status & 0xffffu);
    }
    if(output[0] < 1 || output[0] > 8) {
        return 0x79030000u | ((count & 0xffu) << 8) | output[0];
    }
    return output[0];
}

static uint32_t QueryFileSizeStatus(uint8_t applet) {
    uint8_t output[4];
    uint32_t count = 0;
    uint32_t status = 0;
    if(!QueryControl(applet, 0x91, output, sizeof(output), &count, &status)) {
        return 0x7a010000u | (status & 0xffffu);
    }
    if(count < 2) {
        return 0x7a020000u | ((status & 0xffu) << 8) | (count & 0xffu);
    }
    return ((uint32_t)output[0] << 8) | output[1];
}

static bool OutputHasTerminator(const uint8_t* output, uint32_t output_len) {
    for(uint32_t i = 0; i < output_len; i++) {
        if(output[i] == 0xfe) {
            return true;
        }
    }
    return false;
}

static bool OutputContainsMarker(const uint8_t* output, uint32_t output_len, const char* marker, uint32_t* matched) {
    for(uint32_t i = 0; i < output_len; i++) {
        char value = (char)output[i];
        if(value == marker[*matched]) {
            (*matched)++;
            if(marker[*matched] == '\0') {
                return true;
            }
        } else {
            *matched = value == marker[0] ? 1 : 0;
        }
    }
    return false;
}

static bool BeginReadback(uint8_t applet, uint8_t* output, uint32_t output_len, uint32_t* output_count, uint32_t* status_out) {
    uint32_t status;
    if(!SendSentinel(applet, AW_INTERNAL_CONTROL, &status)) {
        return false;
    }
    bool ok = SendSentinelWithCount(applet, 0x84, output, output_len, &status, output_count);
    if(status_out != 0) {
        *status_out = status;
    }
    return ok;
}

static bool MoveToEnd(uint8_t applet) {
    uint32_t status;
    uint8_t output[32];
    uint32_t output_count = 0;
    if(!BeginReadback(applet, output, sizeof(output), &output_count, &status)) {
        return false;
    }
    for(uint8_t i = 0; i < 64; i++) {
        if(OutputHasTerminator(output, sizeof(output))) {
            return true;
        }
        if(!SendPacket(applet, AW_NS2_READ_NEXT, 0, 0, output, sizeof(output), &status)) {
            return false;
        }
    }
    return false;
}

static uint32_t ReadbackSessionMarkerStatus(uint8_t applet, uint32_t slot) {
    uint32_t status;
    uint8_t output[32];
    uint32_t matched = 0;
    if(!SendCommand(applet, AW_NS2_RESET, 0, &status) || status != AW_STATUS_RESET) {
        return 0x71000000u | (status & 0xffffu);
    }
    if(!SelectAlphaWordSlot(applet, slot)) {
        return 0x72000000u;
    }
    uint32_t output_count = 0;
    if(!BeginReadback(applet, output, sizeof(output), &output_count, &status)) {
        return 0x73000000u | (output_count & 0xffffu);
    }
    for(uint8_t i = 0; i < 64; i++) {
        if(output_count == 0 && i == 0) {
            return 0x77000000u | ((status & 0xffu) << 16) | (output_count & 0xffffu);
        }
        if(OutputContainsMarker(output, sizeof(output), "WriteOrDie", &matched)) {
            return 1;
        }
        if(OutputHasTerminator(output, sizeof(output))) {
            return 0x74000000u | ((uint32_t)output[0] << 16) | ((uint32_t)output[1] << 8) | output[2];
        }
        output_count = 0;
        if(!SendPacketWithCount(applet, AW_NS2_READ_NEXT, 0, 0, output, sizeof(output), &status, &output_count)) {
            return 0x75000000u | (status & 0xffffu);
        }
    }
    return 0x76000000u | ((uint32_t)output[0] << 16) | ((uint32_t)output[1] << 8) | output[2];
}

static bool AppendByte(uint8_t applet, char value) {
    uint8_t byte = value == '\n' ? '\r' : (uint8_t)value;
    uint32_t status;
    return SendPayload(applet, byte, &status);
}

static bool AppendLiteral(uint8_t applet, const char* text) {
    while(*text != '\0') {
        if(!AppendByte(applet, *text)) {
            return false;
        }
        text++;
    }
    return true;
}

static bool AppendEditor(uint8_t applet, const WodEditor_t* editor) {
    for(uint32_t i = 0; i < editor->len; i++) {
        char byte = editor->bytes[i];
        if(byte == '\n' || (byte >= ' ' && byte <= '~')) {
            if(!AppendByte(applet, byte)) {
                return false;
            }
        }
    }
    return true;
}

static uint32_t CountLiteralBytes(const char* text) {
    uint32_t count = 0;
    while(*text != '\0') {
        count++;
        text++;
    }
    return count;
}

static uint32_t CountEditorBytes(const WodEditor_t* editor) {
    uint32_t count = 0;
    for(uint32_t i = 0; i < editor->len; i++) {
        char byte = editor->bytes[i];
        if(byte == '\n' || (byte >= ' ' && byte <= '~')) {
            count++;
        }
    }
    return count;
}

static bool PrepareAlphaWordImportSpan(uint8_t applet, uint32_t byte_count) {
    uint32_t a5 = AppletA5(applet);
    if(a5 < 0x1000u || a5 >= 0x80000u || byte_count == 0) {
        return false;
    }
    uint32_t transfer = a5 + 0x1d6u;
    *(volatile uint8_t*)(transfer + 0x2cu) = 0;
    WriteBigLong(transfer + 0x32u, byte_count);
    WriteBigLong(transfer + 0x36u, 0);
    WriteBigLong(transfer + 0x3au, 0);
    return true;
}

static bool AppendLiteralAsKeys(uint8_t applet, const char* text) {
    while(*text != '\0') {
        if(!SendAlphaWordCharAsKey(applet, *text)) {
            return false;
        }
        text++;
    }
    return true;
}

static bool AppendEditorAsKeys(uint8_t applet, const WodEditor_t* editor) {
    for(uint32_t i = 0; i < editor->len; i++) {
        char value = editor->bytes[i];
        if(value == '\n' || (value >= ' ' && value <= '~')) {
            if(!SendAlphaWordCharAsKey(applet, value)) {
                return false;
            }
        }
    }
    return true;
}

static bool PointerAlreadySeen(const uint32_t* pointers, uint8_t count, uint32_t pointer) {
    for(uint8_t i = 0; i < count; i++) {
        if(pointers[i] == pointer) return true;
    }
    return false;
}

static bool AppendBufferByte(uint8_t* buffer, uint16_t* offset, uint16_t limit, uint8_t value) {
    if(*offset + 1 >= limit) {
        return false;
    }
    buffer[*offset] = value;
    (*offset)++;
    return true;
}

static bool AppendBufferLiteral(uint8_t* buffer, uint16_t* offset, uint16_t limit, const char* text) {
    while(*text != '\0') {
        if(!AppendBufferByte(buffer, offset, limit, (uint8_t)*text)) {
            return false;
        }
        text++;
    }
    return true;
}

static bool AppendBufferEditor(uint8_t* buffer, uint16_t* offset, uint16_t limit, const WodEditor_t* editor) {
    for(uint32_t i = 0; i < editor->len; i++) {
        char byte = editor->bytes[i];
        if(byte == '\n') {
            if(!AppendBufferByte(buffer, offset, limit, '\r')) return false;
        } else if(byte >= ' ' && byte <= '~') {
            if(!AppendBufferByte(buffer, offset, limit, (uint8_t)byte)) return false;
        }
    }
    return true;
}

static bool AppendToBuffer(uint8_t* buffer, uint16_t* offset, uint16_t limit, const WodEditor_t* editor) {
    if(*offset >= limit - 32u) {
        return false;
    }
    if(*offset > 0 && !AppendBufferByte(buffer, offset, limit, '\r')) {
        return false;
    }
    if(!AppendBufferLiteral(buffer, offset, limit, "WriteOrDie session\r")) {
        return false;
    }
    return AppendBufferEditor(buffer, offset, limit, editor);
}

static bool AppendToAlphaWordBuffer(uint8_t* record, uint32_t pointer, uint16_t used, uint16_t capacity, const WodEditor_t* editor) {
    if(pointer < 0x2000u || pointer >= 0x80000u) {
        return false;
    }
    if((capacity != 512u && capacity != 9216u) || used > capacity) {
        return false;
    }
    uint8_t* buffer = (uint8_t*)pointer;
    if(used == capacity && (buffer[0] == 0 || buffer[0] == 0xa7)) {
        used = 0;
    }
    uint16_t offset = used;
    if(offset == 0) {
        while(offset < capacity && buffer[offset] != 0 && buffer[offset] != 0xa7) {
            offset++;
        }
    }
    while(offset < capacity && buffer[offset] != 0 && buffer[offset] != 0xa7) {
        offset++;
    }
    if(!AppendToBuffer(buffer, &offset, capacity, editor)) {
        return false;
    }
    WriteBigLongToBytes(record + 0x2eu, offset);
    WriteBigLongToBytes(record + 0x32u, offset);
    while(offset < capacity) {
        buffer[offset] = 0xa7;
        offset++;
    }
    return true;
}

static uint32_t AppendThroughAlphaWordBuffers(const WodEditor_t* editor, uint32_t slot) {
    uint8_t* memory = (uint8_t*)0;
    uint32_t pointers[8];
    uint8_t pointer_count = 0;
    uint32_t candidates = 0;
    uint32_t appended = 0;
    if(slot < 1 || slot > 8) {
        return 0;
    }
    for(uint32_t name = 0x1000u; name < 0x2400u; name++) {
        uint8_t digit = (uint8_t)('0' + slot);
        if(memory[name] != 'F' || memory[name + 1] != 'i' || memory[name + 2] != 'l' ||
           memory[name + 3] != 'e' || memory[name + 4] != ' ' || memory[name + 5] != digit) {
            continue;
        }
        uint32_t record = name - 0x16u;
        if(memory[record + 0x27u] != slot || memory[record + 0x29u] != FILE_KEY_BY_SLOT[slot - 1]) {
            continue;
        }
        uint32_t pointer = ReadBigLong(&memory[record + 0x2au]);
        uint32_t used_long = ReadBigLong(&memory[record + 0x2eu]);
        uint32_t used_mirror = ReadBigLong(&memory[record + 0x32u]);
        uint32_t capacity_long = ReadBigLong(&memory[record + 0x36u]);
        uint32_t capacity_mirror = ReadBigLong(&memory[record + 0x3au]);
        if(used_long != used_mirror || capacity_long != capacity_mirror) {
            continue;
        }
        if(used_long > capacity_long || (capacity_long != 512u && capacity_long != 9216u)) {
            continue;
        }
        if(PointerAlreadySeen(pointers, pointer_count, pointer)) {
            continue;
        }
        candidates++;
        if(pointer_count < 8) {
            pointers[pointer_count] = pointer;
            pointer_count++;
        }
        if(AppendToAlphaWordBuffer(&memory[record], pointer, (uint16_t)used_long, (uint16_t)capacity_long, editor)) {
            appended++;
        }
    }
    return appended != 0 ? appended : (0x73000000u | (candidates & 0xffffu));
}

static bool AppendThroughAlphaWordMessages(uint8_t applet, const WodEditor_t* editor, uint32_t slot) {
    uint32_t status;
    uint8_t write_or_die = AppletFindById(WRITE_OR_DIE_APPLET_ID);
    uint32_t saved_slot_state = ReadLong(CURRENT_APPLET_SLOT_STATE);
    uint32_t saved_id_state = ReadLong(CURRENT_APPLET_ID_STATE);
    uint32_t saved_slot = ReadLong(CURRENT_APPLET_SLOT);
    bool ok;
    if(slot < 1 || slot > 8) {
        return false;
    }
    SetFirmwareCurrentApplet(applet, ALPHAWORD_APPLET_ID);
    SendCommand(applet, MSG_SETFOCUS, 0, &status);
    ok = SendAlphaWordKey(applet, FILE_KEYS[slot - 1]) &&
         SendAlphaWordKey(applet, 0x3e) &&
         SendAlphaWordKey(applet, 0x40) &&
         AppendLiteralAsKeys(applet, "WriteOrDie session\n") &&
         AppendEditorAsKeys(applet, editor);
    SendCommand(applet, MSG_KILLFOCUS, 0, &status);
    if(write_or_die != 0) {
        SetFirmwareCurrentApplet(write_or_die, WRITE_OR_DIE_APPLET_ID);
    } else {
        WriteLong(CURRENT_APPLET_SLOT_STATE, saved_slot_state);
        WriteLong(CURRENT_APPLET_ID_STATE, saved_id_state);
        WriteLong(CURRENT_APPLET_SLOT, saved_slot);
    }
    return ok;
}

uint32_t alpha_export_append_session(const WodEditor_t* editor, uint32_t slot) {
    uint8_t applet = AppletFindById(ALPHAWORD_APPLET_ID);
    if(applet == 0) {
        return 2;
    }
    uint32_t append_status = AppendThroughAlphaWordBuffers(editor, slot);
    if(append_status == 0) {
        return 3;
    }
    return append_status < 0x10000u ? 1 : append_status;
}
