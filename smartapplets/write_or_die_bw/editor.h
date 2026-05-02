#ifndef WRITE_OR_DIE_EDITOR_H
#define WRITE_OR_DIE_EDITOR_H

#include <stdbool.h>
#include <stdint.h>

#include "app_state.h"

bool editor_insert_byte(WodEditor_t* editor, char byte);
bool editor_backspace(WodEditor_t* editor);
void editor_move_left(WodEditor_t* editor);
void editor_move_right(WodEditor_t* editor);
void editor_move_up(WodEditor_t* editor);
void editor_move_down(WodEditor_t* editor);
uint32_t editor_word_count(const WodEditor_t* editor);
bool editor_delete_last_byte(WodEditor_t* editor);
bool editor_delete_last_word(WodEditor_t* editor);
void editor_render_row(const WodEditor_t* editor, uint8_t row, char* output);

#endif
