#include "forth_util.h"

static char* append_char(char* out, char ch) {
    *out++ = ch;
    *out = '\0';
    return out;
}

void* forth_memcpy(void* dst, const void* src, size_t len) {
    size_t i;
    char* out = (char*)dst;
    const char* in = (const char*)src;
    for(i = 0; i < len; i++) out[i] = in[i];
    return dst;
}

void* forth_memset(void* dst, int value, size_t len) {
    size_t i;
    char* out = (char*)dst;
    for(i = 0; i < len; i++) out[i] = (char)value;
    return dst;
}

size_t forth_strlen(const char* text) {
    size_t len = 0;
    while(text[len] != '\0') len++;
    return len;
}

int forth_strcmp(const char* left, const char* right) {
    while(*left && *left == *right) {
        left++;
        right++;
    }
    return ((unsigned char)*left) - ((unsigned char)*right);
}

void forth_strncpy(char* dst, const char* src, size_t len) {
    size_t i;
    for(i = 0; i < len && src[i] != '\0'; i++) dst[i] = src[i];
    for(; i < len; i++) dst[i] = '\0';
}

void forth_strcpy(char* dst, const char* src) {
    while(*src) *dst++ = *src++;
    *dst = '\0';
}

int forth_isspace(char ch) {
    return ch == ' ' || ch == '\t' || ch == '\r' || ch == '\n';
}

int forth_parse_i32(const char* token, int16_t* value) {
    int sign = 1;
    int16_t acc = 0;
    if(*token == '-') {
        sign = -1;
        token++;
    }
    if(*token == '\0') return 0;
    while(*token) {
        if(*token < '0' || *token > '9') return 0;
        acc = (acc * 10) + (*token - '0');
        token++;
    }
    *value = acc * sign;
    return 1;
}

int forth_append_i32(char* output, size_t size, int16_t value) {
    char scratch[16];
    char* cursor = scratch + sizeof(scratch) - 1;
    size_t used;
    uint16_t mag;
    *cursor = '\0';
    if(size == 0) return 0;
    mag = (value < 0) ? (uint16_t)(-value) : (uint16_t)value;
    do {
        *--cursor = (char)('0' + (mag % 10u));
        mag /= 10u;
    } while(mag != 0u);
    if(value < 0) *--cursor = '-';
    used = forth_strlen(output);
    if(used + forth_strlen(cursor) + 2 > size) return 0;
    if(used != 0) output = append_char(output + used, ' ');
    forth_strcpy(output, cursor);
    return 1;
}
