#include "alphaword_export.h"

#include <stdbool.h>

#include "os3k.h"

enum {
    ALPHAWORD_APPLET_ID = 0xA000,
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
    AW_MIN_CAPACITY = 256
};

static const uint8_t FILE_KEY_BY_SLOT[8] = {
    KEY_FILE_1, KEY_FILE_2, KEY_FILE_3, KEY_FILE_4, KEY_FILE_5, KEY_FILE_6, KEY_FILE_7, KEY_FILE_8
};

static uint32_t ReadBigLong(const uint8_t* bytes) {
    return ((uint32_t)bytes[0] << 24) | ((uint32_t)bytes[1] << 16) | ((uint32_t)bytes[2] << 8) | bytes[3];
}

static void WriteBigLongToBytes(uint8_t* bytes, uint32_t value) {
    bytes[0] = (uint8_t)(value >> 24);
    bytes[1] = (uint8_t)(value >> 16);
    bytes[2] = (uint8_t)(value >> 8);
    bytes[3] = (uint8_t)value;
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
    if(pointer < 0x2000u || pointer >= 0x80000u || capacity < AW_MIN_CAPACITY || used > capacity) {
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
    if(offset < capacity) {
        buffer[offset] = 0xa7;
    }
    return true;
}

static uint32_t BackingSlot(uint32_t visible_slot) {
    return visible_slot == 1u ? 8u : visible_slot - 1u;
}

static bool LooksLikeFileRecord(uint8_t* memory, uint32_t name, uint32_t slot) {
    if(memory[name] != 'F' || memory[name + 1] != 'i' || memory[name + 2] != 'l' ||
       memory[name + 3] != 'e' || memory[name + 4] != ' ') {
        return false;
    }
    if(memory[name + 5] != (uint8_t)('0' + slot)) {
        return false;
    }
    uint32_t record = name - AW_RECORD_NAME_OFFSET;
    return memory[record + AW_RECORD_SLOT_OFFSET] == slot && memory[record + AW_RECORD_KEY_OFFSET] == FILE_KEY_BY_SLOT[slot - 1];
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
        if(!LooksLikeFileRecord(memory, name, backing_slot)) {
            continue;
        }
        uint32_t record = name - AW_RECORD_NAME_OFFSET;
        uint32_t pointer = ReadBigLong(&memory[record + AW_RECORD_POINTER_OFFSET]);
        uint32_t used = ReadBigLong(&memory[record + AW_RECORD_USED_OFFSET]);
        uint32_t used_mirror = ReadBigLong(&memory[record + AW_RECORD_USED_MIRROR_OFFSET]);
        uint32_t capacity = ReadBigLong(&memory[record + AW_RECORD_CAPACITY_OFFSET]);
        uint32_t capacity_mirror = ReadBigLong(&memory[record + AW_RECORD_CAPACITY_MIRROR_OFFSET]);
        if(pointer < 0x2000u || pointer >= 0x80000u) continue;
        if(pointer < AW_RECORD_SCAN_END && pointer + capacity > AW_RECORD_SCAN_START) continue;
        if(used != used_mirror) continue;
        if(capacity != capacity_mirror) continue;
        if(used > capacity || capacity < AW_MIN_CAPACITY) continue;
        candidates++;
        if(best_record == 0 || capacity > best_capacity || (capacity == best_capacity && record > best_record)) {
            best_record = record;
            best_pointer = pointer;
            best_used = used;
            best_capacity = capacity;
        }
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
    if(slot < 1 || slot > 8) {
        return 3;
    }
    uint32_t status = AppendThroughAlphaWordBuffers(slot, title, text, text_len);
    return status < 0x10000u ? 1 : status;
}
