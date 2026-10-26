#include "game.h"

#include <stddef.h>

static const uint8_t MAP[RAY_MAP_H][RAY_MAP_W] = {
    {1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1},
    {1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1},
    {1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1},
    {1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1},
    {1, 1, 1, 0, 0, 2, 0, 0, 0, 0, 2, 0, 0, 1, 1, 1},
    {1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1},
    {1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1},
    {1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1},
    {1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1},
    {1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1},
    {1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1},
    {1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1},
    {1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1},
    {1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1},
    {1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1},
    {1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1},
};


static const int16_t SIN256[256] = {
    0, 3, 6, 9, 13, 16, 19, 22, 25, 28, 31, 34, 37, 40, 43, 46,
    49, 52, 55, 58, 60, 63, 66, 68, 71, 74, 76, 79, 81, 84, 86, 88,
    91, 93, 95, 97, 99, 101, 103, 105, 106, 108, 110, 111, 113, 114, 116, 117,
    118, 119, 121, 122, 122, 123, 124, 125, 126, 126, 127, 127, 127, 128, 128, 128,
    128, 128, 128, 128, 127, 127, 127, 126, 126, 125, 124, 123, 122, 122, 121, 119,
    118, 117, 116, 114, 113, 111, 110, 108, 106, 105, 103, 101, 99, 97, 95, 93,
    91, 88, 86, 84, 81, 79, 76, 74, 71, 68, 66, 63, 60, 58, 55, 52,
    49, 46, 43, 40, 37, 34, 31, 28, 25, 22, 19, 16, 13, 9, 6, 3,
    0, -3, -6, -9, -13, -16, -19, -22, -25, -28, -31, -34, -37, -40, -43, -46,
    -49, -52, -55, -58, -60, -63, -66, -68, -71, -74, -76, -79, -81, -84, -86, -88,
    -91, -93, -95, -97, -99, -101, -103, -105, -106, -108, -110, -111, -113, -114, -116, -117,
    -118, -119, -121, -122, -122, -123, -124, -125, -126, -126, -127, -127, -127, -128, -128, -128,
    -128, -128, -128, -128, -127, -127, -127, -126, -126, -125, -124, -123, -122, -122, -121, -119,
    -118, -117, -116, -114, -113, -111, -110, -108, -106, -105, -103, -101, -99, -97, -95, -93,
    -91, -88, -86, -84, -81, -79, -76, -74, -71, -68, -66, -63, -60, -58, -55, -52,
    -49, -46, -43, -40, -37, -34, -31, -28, -25, -22, -19, -16, -13, -9, -6, -3,
};

static const int16_t COS256[256] = {
    128, 128, 128, 128, 127, 127, 127, 126, 126, 125, 124, 123, 122, 122, 121, 119,
    118, 117, 116, 114, 113, 111, 110, 108, 106, 105, 103, 101, 99, 97, 95, 93,
    91, 88, 86, 84, 81, 79, 76, 74, 71, 68, 66, 63, 60, 58, 55, 52,
    49, 46, 43, 40, 37, 34, 31, 28, 25, 22, 19, 16, 13, 9, 6, 3,
    0, -3, -6, -9, -13, -16, -19, -22, -25, -28, -31, -34, -37, -40, -43, -46,
    -49, -52, -55, -58, -60, -63, -66, -68, -71, -74, -76, -79, -81, -84, -86, -88,
    -91, -93, -95, -97, -99, -101, -103, -105, -106, -108, -110, -111, -113, -114, -116, -117,
    -118, -119, -121, -122, -122, -123, -124, -125, -126, -126, -127, -127, -127, -128, -128, -128,
    -128, -128, -128, -128, -127, -127, -127, -126, -126, -125, -124, -123, -122, -122, -121, -119,
    -118, -117, -116, -114, -113, -111, -110, -108, -106, -105, -103, -101, -99, -97, -95, -93,
    -91, -88, -86, -84, -81, -79, -76, -74, -71, -68, -66, -63, -60, -58, -55, -52,
    -49, -46, -43, -40, -37, -34, -31, -28, -25, -22, -19, -16, -13, -9, -6, -3,
    0, 3, 6, 9, 13, 16, 19, 22, 25, 28, 31, 34, 37, 40, 43, 46,
    49, 52, 55, 58, 60, 63, 66, 68, 71, 74, 76, 79, 81, 84, 86, 88,
    91, 93, 95, 97, 99, 101, 103, 105, 106, 108, 110, 111, 113, 114, 116, 117,
    118, 119, 121, 122, 122, 123, 124, 125, 126, 126, 127, 127, 127, 128, 128, 128,
};

static const uint16_t INV_RAY[256] = {
    0, 32768, 16384, 10922, 8192, 6553, 5461, 4681, 4096, 3640, 3276, 2978, 2730, 2520, 2340, 2184,
    2048, 1927, 1820, 1724, 1638, 1560, 1489, 1424, 1365, 1310, 1260, 1213, 1170, 1129, 1092, 1057,
    1024, 992, 963, 936, 910, 885, 862, 840, 819, 799, 780, 762, 744, 728, 712, 697,
    682, 668, 655, 642, 630, 618, 606, 595, 585, 574, 564, 555, 546, 537, 528, 520,
    512, 504, 496, 489, 481, 474, 468, 461, 455, 448, 442, 436, 431, 425, 420, 414,
    409, 404, 399, 394, 390, 385, 381, 376, 372, 368, 364, 360, 356, 352, 348, 344,
    341, 337, 334, 330, 327, 324, 321, 318, 315, 312, 309, 306, 303, 300, 297, 295,
    292, 289, 287, 284, 282, 280, 277, 275, 273, 270, 268, 266, 264, 262, 260, 258,
    256, 254, 252, 250, 248, 246, 244, 242, 240, 239, 237, 235, 234, 232, 230, 229,
    227, 225, 224, 222, 221, 219, 218, 217, 215, 214, 212, 211, 210, 208, 207, 206,
    204, 203, 202, 201, 199, 198, 197, 196, 195, 193, 192, 191, 190, 189, 188, 187,
    186, 185, 184, 183, 182, 181, 180, 179, 178, 177, 176, 175, 174, 173, 172, 171,
    170, 169, 168, 168, 167, 166, 165, 164, 163, 163, 162, 161, 160, 159, 159, 158,
    157, 156, 156, 155, 154, 153, 153, 152, 151, 151, 150, 149, 148, 148, 147, 146,
    146, 145, 144, 144, 143, 143, 142, 141, 141, 140, 140, 139, 138, 138, 137, 137,
    136, 135, 135, 134, 134, 133, 133, 132, 132, 131, 131, 130, 130, 129, 129, 128,
};

static inline int32_t MulS16(int16_t a, int16_t b) {
#ifdef __mc68000__
    int32_t product = (int32_t)a;
    __asm__ ("muls.w %1,%0" : "+d"(product) : "d"(b));
    return product;
#else
    return (int32_t)a * (int32_t)b;
#endif
}

static inline uint16_t ScaleInvRay(uint16_t value, uint16_t ray_abs) {
    uint16_t inv;
    if(ray_abs == 0) return 0xffffu;
    if(ray_abs >= RAY_ANGLE_COUNT) ray_abs = RAY_ANGLE_COUNT - 1;
    inv = INV_RAY[ray_abs];
#ifdef __mc68000__
    {
        uint32_t product = value;
        __asm__ ("mulu.w %1,%0" : "+d"(product) : "d"(inv));
        return (uint16_t)(product >> 8);
    }
#else
    return (uint16_t)(((uint32_t)value * inv) >> 8);
#endif
}

static inline uint16_t DivU16(uint16_t numerator, uint16_t denominator) {
#ifdef __mc68000__
    uint32_t value = numerator;
    __asm__ ("divu.w %1,%0" : "+d"(value) : "d"(denominator));
    return (uint16_t)value;
#else
    return denominator ? (uint16_t)(numerator / denominator) : 0xffffu;
#endif
}

static inline int16_t DivS16(int32_t numerator, int16_t denominator) {
#ifdef __mc68000__
    __asm__ ("divs.w %1,%0" : "+d"(numerator) : "d"(denominator));
    return (int16_t)numerator;
#else
    return denominator ? (int16_t)(numerator / denominator) : 0;
#endif
}

static inline int16_t Abs16(int16_t value) {
    return value < 0 ? (int16_t)-value : value;
}

static bool WallAtCell(uint8_t x, uint8_t y) {
    if(x >= RAY_MAP_W || y >= RAY_MAP_H) return true;
    return MAP[y][x] != 0;
}

uint8_t ray_game_wall_type(uint8_t x, uint8_t y) {
    if(x >= RAY_MAP_W || y >= RAY_MAP_H) return 1;
    return MAP[y][x];
}

static bool WallAtPoint(int16_t x, int16_t y) {
    if(x < 0 || y < 0) return true;
    return WallAtCell((uint8_t)(x >> 8), (uint8_t)(y >> 8));
}

static bool PlayerBlockedAt(int16_t x, int16_t y) {
    enum { RADIUS = 44 };
    return WallAtPoint((int16_t)(x - RADIUS), (int16_t)(y - RADIUS)) ||
           WallAtPoint((int16_t)(x + RADIUS), (int16_t)(y - RADIUS)) ||
           WallAtPoint((int16_t)(x - RADIUS), (int16_t)(y + RADIUS)) ||
           WallAtPoint((int16_t)(x + RADIUS), (int16_t)(y + RADIUS));
}

static uint8_t AngleAdd(uint8_t angle, int16_t offset) {
    return (uint8_t)((angle + RAY_ANGLE_COUNT + offset) & (RAY_ANGLE_COUNT - 1));
}

static void EnemyRelative(
    const RayGame_t* game,
    uint8_t index,
    int16_t* forward,
    int16_t* lateral
) {
    const RayEnemy_t* enemy = &game->enemies[index];
    int16_t rx = (int16_t)((int16_t)enemy->x * RAY_Q8 + RAY_Q8 / 2 - game->player_x);
    int16_t ry = (int16_t)((int16_t)enemy->y * RAY_Q8 + RAY_Q8 / 2 - game->player_y);
    int16_t dir_x = COS256[game->angle];
    int16_t dir_y = SIN256[game->angle];
    *forward = (int16_t)((MulS16(rx, dir_x) + MulS16(ry, dir_y)) >> 7);
    *lateral = (int16_t)((MulS16(-rx, dir_y) + MulS16(ry, dir_x)) >> 7);
}

static bool LineOfSight(const RayGame_t* game, uint8_t target_x, uint8_t target_y) {
    int16_t x = (int16_t)(game->player_x >> 8);
    int16_t y = (int16_t)(game->player_y >> 8);
    int16_t dx = (int16_t)(target_x - x);
    int16_t dy = (int16_t)(target_y - y);
    int16_t step_x = dx < 0 ? -1 : 1;
    int16_t step_y = dy < 0 ? -1 : 1;
    int16_t err;
    if(dx < 0) dx = (int16_t)-dx;
    if(dy < 0) dy = (int16_t)-dy;
    err = (int16_t)(dx - dy);
    if(x == target_x && y == target_y) return true;
    for(uint8_t guard = 0; guard < 40; guard++) {
        int16_t e2 = (int16_t)(err << 1);
        if(e2 > -dy) {
            err = (int16_t)(err - dy);
            x = (int16_t)(x + step_x);
        }
        if(e2 < dx) {
            err = (int16_t)(err + dx);
            y = (int16_t)(y + step_y);
        }
        if(x == target_x && y == target_y) return true;
        if(WallAtCell((uint8_t)x, (uint8_t)y)) return false;
    }
    return false;
}

void ray_game_init(RayGame_t* game) {
    game->player_x = 8 * RAY_Q8 + RAY_Q8 / 2;
    game->player_y = 8 * RAY_Q8 + RAY_Q8 / 2;
    game->angle = RAY_ANGLE_NORTH;
    game->enemies[0].x = 8;
    game->enemies[0].y = 4;
    game->enemies[0].alive = 1;
    game->enemies[1].x = 5;
    game->enemies[1].y = 9;
    game->enemies[1].alive = 1;
    game->enemies[2].x = 11;
    game->enemies[2].y = 9;
    game->enemies[2].alive = 1;
    game->score = 0;
    game->muzzle_flash = 0;
    game->reserved = 0;
}

void ray_game_turn_left(RayGame_t* game) {
    game->angle = AngleAdd(game->angle, -RAY_TURN_STEP);
}

void ray_game_turn_right(RayGame_t* game) {
    game->angle = AngleAdd(game->angle, RAY_TURN_STEP);
}

void ray_game_move(RayGame_t* game, RayMove_t move) {
    int8_t amount = (int8_t)move;
    int16_t dx = (int16_t)(MulS16(COS256[game->angle], (int16_t)(RAY_MOVE_STEP * amount)) >> 7);
    int16_t dy = (int16_t)(MulS16(SIN256[game->angle], (int16_t)(RAY_MOVE_STEP * amount)) >> 7);
    int16_t nx = (int16_t)(game->player_x + dx);
    int16_t ny = (int16_t)(game->player_y + dy);
    if(!PlayerBlockedAt(nx, game->player_y)) game->player_x = nx;
    if(!PlayerBlockedAt(game->player_x, ny)) game->player_y = ny;
}

int16_t ray_game_dir_x(const RayGame_t* game) {
    return COS256[game->angle];
}

int16_t ray_game_dir_y(const RayGame_t* game) {
    return SIN256[game->angle];
}

RayHit_t ray_game_cast_vector(const RayGame_t* game, int16_t ray_x, int16_t ray_y) {
    RayHit_t hit = {false, 0, 0, 0, 0};
    int16_t abs_x = ray_x < 0 ? (int16_t)-ray_x : ray_x;
    int16_t abs_y = ray_y < 0 ? (int16_t)-ray_y : ray_y;
    uint16_t delta_x = abs_x != 0 ? INV_RAY[abs_x < RAY_ANGLE_COUNT ? abs_x : RAY_ANGLE_COUNT - 1] : 0;
    uint16_t delta_y = abs_y != 0 ? INV_RAY[abs_y < RAY_ANGLE_COUNT ? abs_y : RAY_ANGLE_COUNT - 1] : 0;
    uint8_t cell_x = (uint8_t)(game->player_x >> 8);
    uint8_t cell_y = (uint8_t)(game->player_y >> 8);
    uint8_t step_x = ray_x < 0 ? 0xffu : 1u;
    uint8_t step_y = ray_y < 0 ? 0xffu : 1u;
    uint16_t side_x = 0xffffu;
    uint16_t side_y = 0xffffu;
    uint16_t distance;
    if(abs_x == 0 && abs_y == 0) return hit;

    if(abs_x != 0) {
        uint16_t to_grid = ray_x < 0
            ? (uint16_t)(game->player_x & (RAY_Q8 - 1))
            : (uint16_t)(RAY_Q8 - (game->player_x & (RAY_Q8 - 1)));
        side_x = ScaleInvRay(to_grid, (uint16_t)abs_x);
    }
    if(abs_y != 0) {
        uint16_t to_grid = ray_y < 0
            ? (uint16_t)(game->player_y & (RAY_Q8 - 1))
            : (uint16_t)(RAY_Q8 - (game->player_y & (RAY_Q8 - 1)));
        side_y = ScaleInvRay(to_grid, (uint16_t)abs_y);
    }

    for(uint8_t steps = 0; steps < 32; steps++) {
        uint8_t side;
        if(side_x < side_y) {
            distance = side_x;
            cell_x = (uint8_t)(cell_x + step_x);
            side_x = (uint16_t)(side_x + delta_x);
            side = 0;
        } else {
            distance = side_y;
            cell_y = (uint8_t)(cell_y + step_y);
            side_y = (uint16_t)(side_y + delta_y);
            side = 1;
        }
        if(WallAtCell(cell_x, cell_y)) {
            hit.hit = true;
            hit.distance = distance > 1 ? distance : 1;
            hit.wall_x = cell_x;
            hit.wall_y = cell_y;
            hit.side = side;
            return hit;
        }
    }
    return hit;
}

RayHit_t ray_game_cast_center(const RayGame_t* game) {
    return ray_game_cast_vector(game, COS256[game->angle], SIN256[game->angle]);
}

bool ray_game_enemy_view(
    const RayGame_t* game,
    uint8_t index,
    int16_t* center_x,
    int16_t* width,
    int16_t* height,
    int16_t* bottom_y,
    uint16_t* depth
) {
    int16_t forward;
    int16_t lateral;
    uint16_t sprite_h;
    uint16_t sprite_w;
    const RayEnemy_t* enemy;
    if(index >= RAY_MAX_ENEMIES) return false;
    enemy = &game->enemies[index];
    if(!enemy->alive) return false;
    EnemyRelative(game, index, &forward, &lateral);
    if(forward <= 0) return false;
    {
        int16_t projection = forward < 48 ? 48 : forward;
        sprite_h = DivU16((uint16_t)(RAY_FOCAL * RAY_ENEMY_HEIGHT), (uint16_t)projection);
        if(sprite_h < 3) return false;
        if(sprite_h > 120) sprite_h = 120;
        sprite_w = (uint16_t)((sprite_h * 7) >> 3);
        *center_x = (int16_t)(RAY_VIEW_W / 2 + DivS16(MulS16(lateral, RAY_FOCAL), projection));
        *width = (int16_t)sprite_w;
        *height = (int16_t)sprite_h;
        *bottom_y = (int16_t)(RAY_HORIZON + DivU16((uint16_t)(RAY_FOCAL * RAY_EYE_HEIGHT), (uint16_t)projection));
        if(*bottom_y > RAY_VIEW_H) *bottom_y = RAY_VIEW_H;
        *depth = (uint16_t)forward;
    }
    return true;
}

bool ray_game_shoot(RayGame_t* game) {
    uint8_t best = 0xffu;
    int16_t best_forward = 0x7fff;
    for(uint8_t i = 0; i < RAY_MAX_ENEMIES; i++) {
        int16_t forward;
        int16_t lateral;
        int16_t lat_abs;
        if(!game->enemies[i].alive) continue;
        EnemyRelative(game, i, &forward, &lateral);
        if(forward <= 0 || forward > 12 * RAY_Q8) continue;
        lat_abs = Abs16(lateral);
        if((int32_t)lat_abs * 10 > forward) continue;
        if(!LineOfSight(game, game->enemies[i].x, game->enemies[i].y)) continue;
        if(forward < best_forward) {
            best_forward = forward;
            best = i;
        }
    }
    game->muzzle_flash = 2;
    if(best == 0xffu) return false;
    game->enemies[best].alive = 0;
    if(game->score < 255) game->score++;
    return true;
}

void ray_game_tick(RayGame_t* game) {
    if(game->muzzle_flash > 0) game->muzzle_flash--;
}
