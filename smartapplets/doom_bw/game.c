#include "game.h"

static const uint8_t MAP[RAY_MAP_H][RAY_MAP_W] = {
    {1, 1, 1, 1, 1, 1, 1, 1},
    {1, 0, 0, 0, 0, 0, 0, 1},
    {1, 0, 1, 0, 0, 1, 0, 1},
    {1, 0, 0, 0, 0, 1, 0, 1},
    {1, 0, 0, 1, 0, 0, 0, 1},
    {1, 0, 0, 1, 0, 0, 0, 1},
    {1, 0, 0, 0, 0, 0, 0, 1},
    {1, 1, 1, 1, 1, 1, 1, 1}
};

static const int8_t SIN64[RAY_ANGLE_COUNT] = {
    0, 12, 25, 37, 49, 60, 71, 81, 90, 98, 106, 112, 117, 122, 125, 126,
    127, 126, 125, 122, 117, 112, 106, 98, 90, 81, 71, 60, 49, 37, 25, 12,
    0, -12, -25, -37, -49, -60, -71, -81, -90, -98, -106, -112, -117, -122, -125, -126,
    -127, -126, -125, -122, -117, -112, -106, -98, -90, -81, -71, -60, -49, -37, -25, -12
};

static const int8_t COS64[RAY_ANGLE_COUNT] = {
    127, 126, 125, 122, 117, 112, 106, 98, 90, 81, 71, 60, 49, 37, 25, 12,
    0, -12, -25, -37, -49, -60, -71, -81, -90, -98, -106, -112, -117, -122, -125, -126,
    -127, -126, -125, -122, -117, -112, -106, -98, -90, -81, -71, -60, -49, -37, -25, -12,
    0, 12, 25, 37, 49, 60, 71, 81, 90, 98, 106, 112, 117, 122, 125, 126
};

static const uint16_t INV_RAY[192] = {
    0, 32768, 16384, 10922, 8192, 6553, 5461, 4681, 4096, 3640, 3276, 2978,
    2730, 2520, 2340, 2184, 2048, 1927, 1820, 1724, 1638, 1560, 1489, 1424,
    1365, 1310, 1260, 1213, 1170, 1129, 1092, 1057, 1024, 992, 963, 936,
    910, 885, 862, 840, 819, 799, 780, 762, 744, 728, 712, 697,
    682, 668, 655, 642, 630, 618, 606, 595, 585, 574, 564, 555,
    546, 537, 528, 520, 512, 504, 496, 489, 481, 474, 468, 461,
    455, 448, 442, 436, 431, 425, 420, 414, 409, 404, 399, 394,
    390, 385, 381, 376, 372, 368, 364, 360, 356, 352, 348, 344,
    341, 337, 334, 330, 327, 324, 321, 318, 315, 312, 309, 306,
    303, 300, 297, 295, 292, 289, 287, 284, 282, 280, 277, 275,
    273, 270, 268, 266, 264, 262, 260, 258, 256, 254, 252, 250,
    248, 246, 244, 242, 240, 239, 237, 235, 234, 232, 230, 229,
    227, 225, 224, 222, 221, 219, 218, 217, 215, 214, 212, 211,
    210, 208, 207, 206, 204, 203, 202, 201, 199, 198, 197, 196,
    195, 193, 192, 191, 190, 189, 188, 187, 186, 185, 184, 183,
    182, 181, 180, 179, 178, 177, 176, 175, 174, 173, 172, 171
};

static bool WallAtCell(uint8_t x, uint8_t y) {
    if(x >= RAY_MAP_W || y >= RAY_MAP_H) return true;
    return MAP[y][x] != 0;
}

bool ray_game_wall_at_cell(uint8_t x, uint8_t y) {
    return WallAtCell(x, y);
}

static bool WallAtPoint(int16_t x, int16_t y) {
    if(x < 0 || y < 0) return true;
    return WallAtCell((uint8_t)(x / RAY_Q8), (uint8_t)(y / RAY_Q8));
}

static bool PlayerBlockedAt(int16_t x, int16_t y) {
    enum { RADIUS = 32 };
    return WallAtPoint((int16_t)(x - RADIUS), (int16_t)(y - RADIUS)) ||
           WallAtPoint((int16_t)(x + RADIUS), (int16_t)(y - RADIUS)) ||
           WallAtPoint((int16_t)(x - RADIUS), (int16_t)(y + RADIUS)) ||
           WallAtPoint((int16_t)(x + RADIUS), (int16_t)(y + RADIUS));
}

static uint8_t Angle(uint8_t angle, int8_t offset) {
    return (uint8_t)((angle + RAY_ANGLE_COUNT + offset) & 63);
}

static uint16_t ScaleByInvRay(uint16_t value, uint16_t ray_abs) {
    if(ray_abs == 0) return 0xffffu;
    if(ray_abs >= 192) ray_abs = 191;
    uint32_t product = value;
    uint16_t scale = INV_RAY[ray_abs];
    __asm__ volatile ("mulu.w %1,%0" : "+d"(product) : "d"(scale));
    return (uint16_t)(product >> 8);
}

static uint16_t RayInv(uint16_t ray_abs) {
    if(ray_abs == 0) return 0xffffu;
    if(ray_abs >= 192) ray_abs = 191;
    return INV_RAY[ray_abs];
}

void ray_game_init(RayGame_t* game) {
    game->player_x = 4 * RAY_Q8 + RAY_Q8 / 2;
    game->player_y = 6 * RAY_Q8 + RAY_Q8 / 2;
    game->angle = 56;
    game->enemy_x = 6;
    game->enemy_y = 3;
    game->enemy_alive = 1;
}

void ray_game_turn_left(RayGame_t* game) {
    game->angle = Angle(game->angle, -2);
}

void ray_game_turn_right(RayGame_t* game) {
    game->angle = Angle(game->angle, 2);
}

void ray_game_move(RayGame_t* game, RayMove_t move) {
    int16_t dx = (int16_t)((COS64[game->angle] >> 2) * (int8_t)move);
    int16_t dy = (int16_t)((SIN64[game->angle] >> 2) * (int8_t)move);
    int16_t nx = (int16_t)(game->player_x + dx);
    int16_t ny = (int16_t)(game->player_y + dy);
    if(!PlayerBlockedAt(nx, game->player_y)) game->player_x = nx;
    if(!PlayerBlockedAt(game->player_x, ny)) game->player_y = ny;
}

int16_t ray_game_dir_x(const RayGame_t* game) {
    return COS64[game->angle];
}

int16_t ray_game_dir_y(const RayGame_t* game) {
    return SIN64[game->angle];
}

RayHit_t ray_game_cast_vector(const RayGame_t* game, int16_t ray_x, int16_t ray_y) {
    RayHit_t hit = {false, 0, 0, 0};
    if(ray_x == 0 && ray_y == 0) return hit;

    int16_t abs_x = ray_x < 0 ? (int16_t)-ray_x : ray_x;
    int16_t abs_y = ray_y < 0 ? (int16_t)-ray_y : ray_y;
    uint16_t delta_x = RayInv((uint16_t)abs_x);
    uint16_t delta_y = RayInv((uint16_t)abs_y);
    uint8_t cell_x = (uint8_t)(game->player_x / RAY_Q8);
    uint8_t cell_y = (uint8_t)(game->player_y / RAY_Q8);
    uint8_t step_x = ray_x < 0 ? 0xffu : 1u;
    uint8_t step_y = ray_y < 0 ? 0xffu : 1u;
    uint16_t side_x = 0xffffu;
    uint16_t side_y = 0xffffu;

    if(abs_x != 0) {
        uint16_t to_grid = ray_x < 0
            ? (uint16_t)(game->player_x & (RAY_Q8 - 1))
            : (uint16_t)(RAY_Q8 - (game->player_x & (RAY_Q8 - 1)));
        side_x = ScaleByInvRay(to_grid, (uint16_t)abs_x);
    }
    if(abs_y != 0) {
        uint16_t to_grid = ray_y < 0
            ? (uint16_t)(game->player_y & (RAY_Q8 - 1))
            : (uint16_t)(RAY_Q8 - (game->player_y & (RAY_Q8 - 1)));
        side_y = ScaleByInvRay(to_grid, (uint16_t)abs_y);
    }

    for(uint8_t steps = 0; steps < 16; steps++) {
        uint16_t distance;
        if(side_x < side_y) {
            distance = side_x;
            cell_x = (uint8_t)(cell_x + step_x);
            side_x = (uint16_t)(side_x + delta_x);
        } else {
            distance = side_y;
            cell_y = (uint8_t)(cell_y + step_y);
            side_y = (uint16_t)(side_y + delta_y);
        }
        if(WallAtCell(cell_x, cell_y)) {
            hit.hit = true;
            hit.distance = distance > 1 ? distance : 1;
            hit.wall_x = cell_x;
            hit.wall_y = cell_y;
            return hit;
        }
    }
    return hit;
}

RayHit_t ray_game_cast(const RayGame_t* game, int8_t angle_offset) {
    uint8_t angle = Angle(game->angle, angle_offset);
    return ray_game_cast_vector(game, COS64[angle], SIN64[angle]);
}

RayHit_t ray_game_cast_center(const RayGame_t* game) {
    return ray_game_cast(game, 0);
}

bool ray_game_enemy_visible(const RayGame_t* game, uint16_t* distance) {
    if(!game->enemy_alive) return false;
    int16_t ex = (int16_t)(game->enemy_x * RAY_Q8 + RAY_Q8 / 2);
    int16_t ey = (int16_t)(game->enemy_y * RAY_Q8 + RAY_Q8 / 2);
    int16_t dx = (int16_t)(ex - game->player_x);
    int16_t dy = (int16_t)(ey - game->player_y);
    if(game->angle == RAY_ANGLE_EAST && dx <= 0) return false;
    if(game->angle == RAY_ANGLE_WEST && dx >= 0) return false;
    if(game->angle == RAY_ANGLE_SOUTH && dy <= 0) return false;
    if(game->angle == RAY_ANGLE_NORTH && dy >= 0) return false;
    uint16_t approx = (uint16_t)((dx < 0 ? -dx : dx) + (dy < 0 ? -dy : dy));
    RayHit_t wall = ray_game_cast_center(game);
    if(distance != 0) *distance = approx;
    return wall.hit && approx < wall.distance + 80;
}

bool ray_game_shoot(RayGame_t* game) {
    if(!ray_game_enemy_visible(game, 0)) return false;
    game->enemy_alive = 0;
    return true;
}
