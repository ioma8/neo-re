#include "alphaword_export.h"

#include <stdbool.h>

#include "os3k.h"

enum {
    ALPHAWORD_APPLET_ID = 0xA000,
    CURRENT_APPLET_SLOT_STATE = 0x000035e2,
    CURRENT_APPLET_ID_STATE = 0x000035e6,
    CURRENT_APPLET_SLOT = 0x000035ec,
    AW_RECORD_SCAN_START = 0x1000,
    AW_RECORD_SCAN_END = 0x2400,
    AW_RECORD_NAME_OFFSET = 0x16,
    AW_RECORD_SLOT_OFFSET = 0x27,
    AW_RECORD_KEY_OFFSET = 0x29,
    AW_RECORD_POINTER_OFFSET = 0x2a,
    AW_RECORD_USED_OFFSET = 0x2e,
    AW_RECORD_USED_MIRROR_OFFSET = 0x32,
    AW_RECORD_CAPACITY_OFFSET = 0x36,
    AW_RECORD_CAPACITY_MIRROR_OFFSET = 0x3a,
    AW_SMALL_CAPACITY = 512,
    AW_LARGE_CAPACITY = 9216
};

static const uint8_t FILE_KEY_BY_SLOT[8] = {
    KEY_FILE_1, KEY_FILE_2, KEY_FILE_3, KEY_FILE_4, KEY_FILE_5, KEY_FILE_6, KEY_FILE_7, KEY_FILE_8
};

static uint32_t ReadLong(uint32_t address) {
    return *(volatile uint32_t*)address;
}

static void WriteLong(uint32_t address, uint32_t value) {
    *(volatile uint32_t*)address = value;
}

static uint32_t ReadBigLong(const uint8_t* bytes) {
    return ((uint32_t)bytes[0] << 24) | ((uint32_t)bytes[1] << 16) | ((uint32_t)bytes[2] << 8) | bytes[3];
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

static bool SendCommand(uint8_t applet, uint32_t command, uint32_t param) {
    uint32_t status = 0;
    (void)AppletSendMessage(applet, (Message_e)command, param, &status);
    return true;
}

static bool PreselectAlphaWordFile(uint8_t applet, uint32_t slot) {
    uint32_t saved_slot_state = ReadLong(CURRENT_APPLET_SLOT_STATE);
    uint32_t saved_id_state = ReadLong(CURRENT_APPLET_ID_STATE);
    uint32_t saved_slot = ReadLong(CURRENT_APPLET_SLOT);
    bool ok;
    if(slot < 1 || slot > 8) {
        return false;
    }
    SetFirmwareCurrentApplet(applet, ALPHAWORD_APPLET_ID);
    SendCommand(applet, MSG_SETFOCUS, 0);
    ok = SendCommand(applet, MSG_KEY, FILE_KEY_BY_SLOT[slot - 1]);
    SendCommand(applet, MSG_KILLFOCUS, 0);
    WriteLong(CURRENT_APPLET_SLOT_STATE, saved_slot_state);
    WriteLong(CURRENT_APPLET_ID_STATE, saved_id_state);
    WriteLong(CURRENT_APPLET_SLOT, saved_slot);
    return ok;
}

static bool AppendByte(uint8_t* buffer, uint16_t* offset, uint16_t limit, uint8_t value) {
    if(*offset + 1 >= limit) {
        return false;
    }
    buffer[*offset] = value;
    (*offset)++;
    return true;
}

static bool AppendLiteral(uint8_t* buffer, uint16_t* offset, uint16_t limit, const char* text) {
    while(*text != '\0') {
        if(!AppendByte(buffer, offset, limit, (uint8_t)*text)) {
            return false;
        }
        text++;
    }
    return true;
}

static bool AppendText(uint8_t* buffer, uint16_t* offset, uint16_t limit, const char* text, uint32_t text_len) {
    for(uint32_t i = 0; i < text_len; i++) {
        char byte = text[i];
        if(byte == '\n') {
            if(!AppendByte(buffer, offset, limit, '\r')) return false;
        } else if(byte >= ' ' && byte <= '~') {
            if(!AppendByte(buffer, offset, limit, (uint8_t)byte)) return false;
        }
    }
    return true;
}

static bool AppendBlock(uint8_t* buffer, uint16_t* offset, uint16_t limit, const char* title, const char* text, uint32_t text_len) {
    if(*offset >= limit - 32u) {
        return false;
    }
    if(*offset > 0 && !AppendByte(buffer, offset, limit, '\r')) {
        return false;
    }
    if(!AppendLiteral(buffer, offset, limit, title) || !AppendByte(buffer, offset, limit, '\r')) {
        return false;
    }
    return AppendText(buffer, offset, limit, text, text_len);
}

static bool AppendToRecord(uint8_t* record, uint32_t pointer, uint16_t used, uint16_t capacity, const char* title, const char* text, uint32_t text_len) {
    uint8_t* buffer = (uint8_t*)pointer;
    uint16_t offset = used == capacity ? 0 : used;
    if(pointer < 0x2000u || pointer >= 0x80000u || (capacity != AW_SMALL_CAPACITY && capacity != AW_LARGE_CAPACITY) || used > capacity) {
        return false;
    }
    while(offset < capacity && buffer[offset] != 0 && buffer[offset] != 0xa7) {
        offset++;
    }
    if(!AppendBlock(buffer, &offset, capacity, title, text, text_len)) {
        return false;
    }
    WriteBigLongToBytes(record + AW_RECORD_USED_OFFSET, offset);
    WriteBigLongToBytes(record + AW_RECORD_USED_MIRROR_OFFSET, offset);
    while(offset < capacity) {
        buffer[offset] = 0xa7;
        offset++;
    }
    return true;
}

static uint32_t BackingSlot(uint32_t visible_slot) {
    return visible_slot == 1u ? 8u : visible_slot - 1u;
}

static uint32_t AppendThroughAlphaWordBuffers(uint32_t visible_slot, const char* title, const char* text, uint32_t text_len) {
    uint8_t* memory = (uint8_t*)0;
    uint32_t backing_slot = BackingSlot(visible_slot);
    uint32_t candidates = 0;
    uint32_t best_record = 0;
    uint32_t best_pointer = 0;
    uint32_t best_used = 0;
    uint32_t best_capacity = 0;
    for(uint32_t name = AW_RECORD_SCAN_START; name < AW_RECORD_SCAN_END; name++) {
        uint8_t digit = (uint8_t)('0' + backing_slot);
        if(memory[name] != 'F' || memory[name + 1] != 'i' || memory[name + 2] != 'l' ||
           memory[name + 3] != 'e' || memory[name + 4] != ' ' || memory[name + 5] != digit) {
            continue;
        }
        uint32_t record = name - AW_RECORD_NAME_OFFSET;
        if(memory[record + AW_RECORD_SLOT_OFFSET] != backing_slot || memory[record + AW_RECORD_KEY_OFFSET] != FILE_KEY_BY_SLOT[backing_slot - 1]) {
            continue;
        }
        uint32_t pointer = ReadBigLong(&memory[record + AW_RECORD_POINTER_OFFSET]);
        uint32_t used = ReadBigLong(&memory[record + AW_RECORD_USED_OFFSET]);
        uint32_t used_mirror = ReadBigLong(&memory[record + AW_RECORD_USED_MIRROR_OFFSET]);
        uint32_t capacity = ReadBigLong(&memory[record + AW_RECORD_CAPACITY_OFFSET]);
        uint32_t capacity_mirror = ReadBigLong(&memory[record + AW_RECORD_CAPACITY_MIRROR_OFFSET]);
        if(used != used_mirror) continue;
        if(capacity != AW_SMALL_CAPACITY && capacity != AW_LARGE_CAPACITY) capacity = capacity_mirror;
        if(used > capacity || (capacity != AW_SMALL_CAPACITY && capacity != AW_LARGE_CAPACITY)) continue;
        candidates++;
        best_record = record;
        best_pointer = pointer;
        best_used = used;
        best_capacity = capacity;
    }
    if(best_record != 0 && AppendToRecord(&memory[best_record], best_pointer, (uint16_t)best_used, (uint16_t)best_capacity, title, text, text_len)) {
        return 1;
    }
    return 0x73000000u | (candidates & 0xffffu);
}

uint32_t alphaword_append_text_block(uint32_t slot, const char* title, const char* text, uint32_t text_len) {
    uint8_t applet = AppletFindById(ALPHAWORD_APPLET_ID);
    if(applet == 0) {
        return 2;
    }
    if(slot < 1 || slot > 8 || !PreselectAlphaWordFile(applet, slot)) {
        return 3;
    }
    uint32_t status = AppendThroughAlphaWordBuffers(slot, title, text, text_len);
    return status < 0x10000u ? 1 : status;
}
