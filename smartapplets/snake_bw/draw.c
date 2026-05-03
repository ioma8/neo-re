#include "draw.h"

#include "os3k.h"

enum {
    CELL_W = 4,
    CELL_H = 4,
    PLAY_W = 200,
    LCD_H = 64,
    SIDEBAR_X = 200
};

static void Fill(uint16_t x, uint16_t y, uint16_t w, uint16_t h, RopCode_e rop) {
    RasterOp(x, y, w, h, 0, rop);
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

static uint8_t Glyph(char c, uint8_t row) {
    static const uint8_t digits[10][5] = {
        {7, 5, 5, 5, 7}, {2, 6, 2, 2, 7}, {7, 1, 7, 4, 7}, {7, 1, 7, 1, 7}, {5, 5, 7, 1, 1},
        {7, 4, 7, 1, 7}, {7, 4, 7, 5, 7}, {7, 1, 1, 1, 1}, {7, 5, 7, 5, 7}, {7, 5, 7, 1, 7}
    };
    if(c >= '0' && c <= '9') return digits[(uint8_t)(c - '0')][row];
    switch(c) {
        case 'A': { static const uint8_t v[5] = {2, 5, 7, 5, 5}; return v[row]; }
        case 'E': { static const uint8_t v[5] = {7, 4, 6, 4, 7}; return v[row]; }
        case 'K': { static const uint8_t v[5] = {5, 6, 4, 6, 5}; return v[row]; }
        case 'L': { static const uint8_t v[5] = {4, 4, 4, 4, 7}; return v[row]; }
        case 'N': { static const uint8_t v[5] = {5, 7, 7, 7, 5}; return v[row]; }
        case 'P': { static const uint8_t v[5] = {6, 5, 6, 4, 4}; return v[row]; }
        case 'R': { static const uint8_t v[5] = {6, 5, 6, 5, 5}; return v[row]; }
        case 'S': { static const uint8_t v[5] = {7, 4, 7, 1, 7}; return v[row]; }
        case 'T': { static const uint8_t v[5] = {7, 2, 2, 2, 2}; return v[row]; }
        case 'U': { static const uint8_t v[5] = {5, 5, 5, 5, 7}; return v[row]; }
        default: return 0;
    }
}

static void DrawChar(char c, uint16_t x, uint16_t y, uint8_t scale) {
    for(uint8_t row = 0; row < 5; row++) {
        uint8_t bits = Glyph(c, row);
        for(uint8_t col = 0; col < 3; col++) {
            if((bits & (1u << (2u - col))) != 0) {
                Fill((uint16_t)(x + col * scale), (uint16_t)(y + row * scale), scale, scale, ROP_BLACKNESS);
            }
        }
    }
}

static void DrawText(const char* text, uint16_t x, uint16_t y, uint8_t scale) {
    while(*text != '\0') {
        DrawChar(*text, x, y, scale);
        x = (uint16_t)(x + 4u * scale);
        text++;
    }
}

static void DrawNumber(uint8_t value, uint16_t x, uint16_t y) {
    DrawChar((char)('0' + (value / 100u) % 10u), x, y, 1);
    DrawChar((char)('0' + (value / 10u) % 10u), (uint16_t)(x + 4), y, 1);
    DrawChar((char)('0' + value % 10u), (uint16_t)(x + 8), y, 1);
}

static void DrawSidebar(const SnakeGame_t* game) {
    Fill(SIDEBAR_X, 0, 64, LCD_H, ROP_WHITENESS);
    DrawText("SNAKE", 210, 4, 2);
    DrawText("P PAUSE", 204, 22, 1);
    DrawText("R RESET", 204, 32, 1);
    DrawText("LEN", 204, 48, 1);
    DrawNumber(game->length, 220, 48);
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
