#ifndef FORTH_UTIL_H
#define FORTH_UTIL_H

#include <stddef.h>
#include <stdint.h>

void* forth_memcpy(void* dst, const void* src, size_t len);
void* forth_memset(void* dst, int value, size_t len);
size_t forth_strlen(const char* text);
int forth_strcmp(const char* left, const char* right);
void forth_strncpy(char* dst, const char* src, size_t len);
void forth_strcpy(char* dst, const char* src);
int forth_isspace(char ch);
int forth_parse_i32(const char* token, int16_t* value);
int forth_append_i32(char* output, size_t size, int16_t value);

#endif
