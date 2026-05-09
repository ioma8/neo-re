#include "draw.h"

#include "../betawise-sdk/bw.h"

enum {
    CELL_W = 4,
    CELL_H = 4,
    PLAY_W = 200,
    LCD_H = 64,
    SIDEBAR_X = 200
};

static void Fill(uint16_t x, uint16_t y, uint16_t w, uint16_t h, RopCode_e rop) {
    bw_fill_rect(x, y, w, h, rop);
}

static void DrawCell(uint8_t x, uint8_t y, RopCode_e rop) {
    Fill((uint16_t)x * CELL_W, (uint16_t)y * CELL_H, CELL_W, CELL_H, rop);
}

static void PaintCell(uint8_t x, uint8_t y) {
    DrawCell(x, y, ROP_BLACKNESS);
}

static void ClearCell(uint8_t x, uint8_t y) {
    DrawCell(x, y, ROP_WHITENESS);
}

static void DrawText(const char* text, uint16_t x, uint16_t y, uint8_t scale) {
    bw_draw_text_3x5(text, x, y, scale);
}

static void DrawNumber(uint8_t value, uint16_t x, uint16_t y) {
    bw_draw_u8_3digits(value, x, y, 1);
}

static void DrawSidebar(const SnakeGame_t* game) {
    Fill(SIDEBAR_X, 0, 64, LCD_H, ROP_WHITENESS);
    DrawText("SNAKE", 210, 4, 2);
    DrawText("P PAUSE", 204, 22, 1);
    DrawText("R RESET", 204, 32, 1);
    DrawText("SCORE", 204, 48, 1);
    DrawNumber(game->score, 228, 48);
}

static void DrawPauseMark(void) {
    Fill(92, 22, 5, 20, ROP_BLACKNESS);
    Fill(104, 22, 5, 20, ROP_BLACKNESS);
}

static void DrawGameOverMark(void) {
    for(uint8_t i = 0; i < 28; i++) {
        Fill((uint16_t)(86 + i), (uint16_t)(18 + i), 3, 3, ROP_BLACKNESS);
        Fill((uint16_t)(114 - i), (uint16_t)(18 + i), 3, 3, ROP_BLACKNESS);
    }
}

void snake_draw_full(const SnakeGame_t* game) {
    Fill(0, 0, PLAY_W, LCD_H, ROP_WHITENESS);
    PaintCell(game->food_x, game->food_y);
    for(uint8_t i = 0; i < game->length; i++) {
        PaintCell(game->body_x[i], game->body_y[i]);
    }
    if(game->paused) DrawPauseMark();
    if(game->game_over) DrawGameOverMark();
    DrawSidebar(game);
}

void snake_draw_step(
    SnakeGame_t* game,
    uint8_t old_tail_x,
    uint8_t old_tail_y,
    uint8_t old_food_x,
    uint8_t old_food_y,
    uint8_t old_length
) {
    if(game->game_over) {
        snake_draw_full(game);
        return;
    }
    if(game->length == old_length && !snake_game_contains(game, old_tail_x, old_tail_y)) {
        ClearCell(old_tail_x, old_tail_y);
    }
    if((game->food_x != old_food_x || game->food_y != old_food_y) &&
       !snake_game_contains(game, old_food_x, old_food_y)) {
        ClearCell(old_food_x, old_food_y);
    }
    PaintCell(game->food_x, game->food_y);
    PaintCell(game->body_x[0], game->body_y[0]);
    if(game->length != old_length) DrawSidebar(game);
}
