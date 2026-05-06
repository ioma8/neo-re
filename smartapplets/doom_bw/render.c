#include "render.h"

#include "os3k.h"

enum {
    LCD_W = 264,
    VIEW_W = 160,
    LCD_H = 64,
    HORIZON = 31,
    COL_W = 4,
    MAP_X = 178,
    MAP_Y = 4,
    MAP_CELL = 7
};

typedef struct {
    uint16_t x;
    int16_t top;
    int16_t bot;
    uint8_t wall_x;
    uint8_t wall_y;
    bool hit;
} Slice_t;

static void Fill(int16_t x, int16_t y, int16_t w, int16_t h, RopCode_e rop) {
    if(x < 0) {
        w = (int16_t)(w + x);
        x = 0;
    }
    if(y < 0) {
        h = (int16_t)(h + y);
        y = 0;
    }
    if(x + w > LCD_W) w = (int16_t)(LCD_W - x);
    if(y + h > LCD_H) h = (int16_t)(LCD_H - y);
    if(w > 0 && h > 0) RasterOp((uint16_t)x, (uint16_t)y, (uint16_t)w, (uint16_t)h, 0, rop);
}

static int16_t Abs16(int16_t value) {
    return value < 0 ? (int16_t)-value : value;
}

static int16_t WallHeight(uint16_t dist) {
    if(dist < 136) return 64;
    if(dist > 1120) return 9;
    return (int16_t)(9 + ((1120 - dist) >> 4));
}

static int16_t ClampY(int16_t y) {
    if(y < 0) return 0;
    if(y >= LCD_H) return LCD_H - 1;
    return y;
}

static void DrawWallFace(const Slice_t* left, const Slice_t* right) {
    int16_t x0 = (int16_t)left->x;
    int16_t x1 = (int16_t)(right->x + COL_W - 1);
    int16_t span = (int16_t)(x1 - x0);
    int16_t top = left->top;
    int16_t bot = left->bot;
    int16_t top_step = left->top <= right->top ? 1 : -1;
    int16_t bot_step = left->bot <= right->bot ? 1 : -1;
    int16_t top_delta = Abs16((int16_t)(right->top - left->top));
    int16_t bot_delta = Abs16((int16_t)(right->bot - left->bot));
    int16_t top_err = 0;
    int16_t bot_err = 0;
    uint8_t shaded = (uint8_t)((left->wall_x + left->wall_y) & 1u);
    for(int16_t x = x0; x <= x1; x = (int16_t)(x + COL_W)) {
        int16_t clamped_top = ClampY(top);
        int16_t clamped_bot = ClampY(bot);
        int16_t h = (int16_t)(clamped_bot - clamped_top + 1);
        Fill(x, clamped_top, COL_W, h, ROP_BLACKNESS);
        if(shaded && h > 6) {
            Fill((int16_t)(x + COL_W - 1), (int16_t)(clamped_top + 2), 1, (int16_t)(h - 4), ROP_WHITENESS);
        }
        if(span > 0) {
            top_err = (int16_t)(top_err + top_delta * COL_W);
            while(top_err >= span) {
                top = (int16_t)(top + top_step);
                top_err = (int16_t)(top_err - span);
            }
            bot_err = (int16_t)(bot_err + bot_delta * COL_W);
            while(bot_err >= span) {
                bot = (int16_t)(bot + bot_step);
                bot_err = (int16_t)(bot_err - span);
            }
        }
    }
}

static void DrawSeam(uint16_t x, int16_t top, int16_t bot) {
    Fill((int16_t)x, top, 1, (int16_t)(bot - top + 1), ROP_WHITENESS);
}

static void DrawWalls(const RayGame_t* game) {
    Slice_t first = {0};
    Slice_t prev = {0};
    Slice_t group_start = {0};
    bool have_group = false;
    int16_t dir_x = ray_game_dir_x(game);
    int16_t dir_y = ray_game_dir_y(game);
    int16_t plane_x = (int16_t)(-dir_y / 2);
    int16_t plane_y = (int16_t)(dir_x / 2);
    int16_t ray_scale_x = (int16_t)(plane_x * -(VIEW_W / 2));
    int16_t ray_scale_y = (int16_t)(plane_y * -(VIEW_W / 2));
    int16_t ray_step_x = (int16_t)(plane_x * COL_W);
    int16_t ray_step_y = (int16_t)(plane_y * COL_W);
    for(uint16_t x = 0; x < VIEW_W; x += COL_W) {
        int16_t ray_x = (int16_t)(dir_x + (ray_scale_x >> 7));
        int16_t ray_y = (int16_t)(dir_y + (ray_scale_y >> 7));
        RayHit_t hit = ray_game_cast_vector(game, ray_x, ray_y);
        uint16_t dist = hit.hit && hit.distance > 32 ? hit.distance : 32;
        int16_t wall_h = WallHeight(dist);
        int16_t top = (int16_t)(HORIZON - wall_h / 2);
        int16_t bot = (int16_t)(top + wall_h);
        top = ClampY(top);
        bot = ClampY(bot);
        Slice_t current = {x, top, bot, hit.wall_x, hit.wall_y, hit.hit};
        if(!have_group) {
            first = current;
            group_start = current;
            have_group = true;
        } else if(current.wall_x != prev.wall_x || current.wall_y != prev.wall_y || current.hit != prev.hit) {
            DrawWallFace(&group_start, &prev);
            int16_t seam_top = current.top < prev.top ? current.top : prev.top;
            int16_t seam_bot = current.bot > prev.bot ? current.bot : prev.bot;
            DrawSeam(current.x, seam_top, seam_bot);
            group_start = current;
        }
        prev = current;
        ray_scale_x = (int16_t)(ray_scale_x + ray_step_x);
        ray_scale_y = (int16_t)(ray_scale_y + ray_step_y);
    }
    if(have_group) DrawWallFace(&group_start, &prev);
    DrawSeam(first.x, first.top, first.bot);
    DrawSeam((uint16_t)(prev.x + COL_W - 1), prev.top, prev.bot);
}

static void DrawGun(void) {
    Fill(64, 50, 40, 14, ROP_BLACKNESS);
    Fill(76, 42, 16, 16, ROP_BLACKNESS);
    Fill(81, 34, 6, 16, ROP_BLACKNESS);
    Fill(70, 50, 28, 4, ROP_WHITENESS);
}

static void DrawEnemy(const RayGame_t* game) {
    uint16_t distance = 0;
    if(!ray_game_enemy_visible(game, &distance)) return;
    int16_t size = 8;
    if(distance < 350) size = 30;
    else if(distance < 550) size = 22;
    else if(distance < 800) size = 14;
    int16_t x = (int16_t)(VIEW_W / 2 - size / 2);
    int16_t y = (int16_t)(HORIZON - size / 2 + 8);
    Fill(x, y, size, size, ROP_BLACKNESS);
    Fill((int16_t)(x + size / 4), (int16_t)(y + size / 4), 3, 3, ROP_WHITENESS);
    Fill((int16_t)(x + size - size / 4 - 3), (int16_t)(y + size / 4), 3, 3, ROP_WHITENESS);
}

static int16_t MapCoord(int16_t q8) {
    int16_t cells = (int16_t)(q8 >> 8);
    int16_t frac = (int16_t)((q8 & 0xff) >> 6);
    return (int16_t)(cells * MAP_CELL + frac);
}

static void DrawMinimap(const RayGame_t* game) {
    Fill(VIEW_W, 0, 1, LCD_H, ROP_BLACKNESS);
    Fill(MAP_X - 2, MAP_Y - 2, RAY_MAP_W * MAP_CELL + 4, 1, ROP_BLACKNESS);
    Fill(MAP_X - 2, MAP_Y + RAY_MAP_H * MAP_CELL + 1, RAY_MAP_W * MAP_CELL + 4, 1, ROP_BLACKNESS);
    Fill(MAP_X - 2, MAP_Y - 2, 1, RAY_MAP_H * MAP_CELL + 4, ROP_BLACKNESS);
    Fill(MAP_X + RAY_MAP_W * MAP_CELL + 1, MAP_Y - 2, 1, RAY_MAP_H * MAP_CELL + 4, ROP_BLACKNESS);
    for(uint8_t y = 0; y < RAY_MAP_H; y++) {
        uint8_t run_start = 0;
        uint8_t run_len = 0;
        for(uint8_t x = 0; x <= RAY_MAP_W; x++) {
            bool wall = x < RAY_MAP_W && ray_game_wall_at_cell(x, y);
            if(wall) {
                if(run_len == 0) run_start = x;
                run_len++;
            } else if(run_len != 0) {
                Fill(
                    (int16_t)(MAP_X + run_start * MAP_CELL),
                    (int16_t)(MAP_Y + y * MAP_CELL),
                    (int16_t)(run_len * MAP_CELL - 1),
                    MAP_CELL - 1,
                    ROP_BLACKNESS
                );
                run_len = 0;
            }
        }
    }
    if(game->enemy_alive) {
        Fill((int16_t)(MAP_X + game->enemy_x * MAP_CELL + 2), (int16_t)(MAP_Y + game->enemy_y * MAP_CELL + 2), 3, 3, ROP_BLACKNESS);
    }
    int16_t px = (int16_t)(MAP_X + MapCoord(game->player_x));
    int16_t py = (int16_t)(MAP_Y + MapCoord(game->player_y));
    Fill((int16_t)(px - 1), (int16_t)(py - 1), 3, 3, ROP_BLACKNESS);
    int16_t dx = (int16_t)(ray_game_dir_x(game) >> 5);
    int16_t dy = (int16_t)(ray_game_dir_y(game) >> 5);
    Fill((int16_t)(px + dx), (int16_t)(py + dy), 2, 2, ROP_BLACKNESS);
    Fill((int16_t)(px + dx + (dx >> 1)), (int16_t)(py + dy + (dy >> 1)), 2, 2, ROP_BLACKNESS);
}

void ray_render_frame(const RayGame_t* game) {
    Fill(0, 0, LCD_W, LCD_H, ROP_WHITENESS);
    DrawWalls(game);
    DrawEnemy(game);
    DrawGun();
    DrawMinimap(game);
}
