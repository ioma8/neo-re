#ifndef DOOM_GAME_H
#define DOOM_GAME_H

#include <stdbool.h>
#include <stdint.h>

enum {
    RAY_Q8 = 256,
    RAY_MAP_W = 16,
    RAY_MAP_H = 16,
    RAY_ANGLE_COUNT = 256,
    RAY_ANGLE_EAST = 0,
    RAY_ANGLE_SOUTH = 64,
    RAY_ANGLE_WEST = 128,
    RAY_ANGLE_NORTH = 192,
    RAY_MAX_ENEMIES = 3,
    RAY_VIEW_W = 264,
    RAY_COLS = RAY_VIEW_W,
    RAY_VIEW_H = 64,
    RAY_HORIZON = 32,
    RAY_FOCAL = 140,
    RAY_EYE_HEIGHT = 128,
    RAY_WALL_HEIGHT = 240,
    RAY_ENEMY_HEIGHT = 240,
    RAY_ENEMY_WIDTH = 176,
    RAY_MOVE_STEP = 60,
    RAY_TURN_STEP = 8
};

typedef enum {
    RAY_MOVE_FORWARD = 1,
    RAY_MOVE_BACKWARD = -1
} RayMove_t;

typedef struct {
    bool hit;
    uint16_t distance;
    uint8_t wall_x;
    uint8_t wall_y;
    uint8_t side;
    uint8_t wall_type;
    uint8_t tex_u;
} RayHit_t;

typedef struct {
    uint8_t x;
    uint8_t y;
    uint8_t alive;
} RayEnemy_t;

typedef struct {
    int16_t player_x;
    int16_t player_y;
    uint8_t angle;
    RayEnemy_t enemies[RAY_MAX_ENEMIES];
    uint8_t score;
    uint8_t hit_flash;
    uint8_t muzzle_flash;
} RayGame_t;

void ray_game_init(RayGame_t* game);
void ray_game_turn_left(RayGame_t* game);
void ray_game_turn_right(RayGame_t* game);
void ray_game_move(RayGame_t* game, RayMove_t move);
RayHit_t ray_game_cast_center(const RayGame_t* game);
RayHit_t ray_game_cast(const RayGame_t* game, int16_t angle_offset);
RayHit_t ray_game_cast_vector(const RayGame_t* game, int16_t ray_x, int16_t ray_y);
int16_t ray_game_dir_x(const RayGame_t* game);
int16_t ray_game_dir_y(const RayGame_t* game);
bool ray_game_wall_at_cell(uint8_t x, uint8_t y);
uint8_t ray_game_wall_type(uint8_t x, uint8_t y);
bool ray_game_enemy_visible(const RayGame_t* game, uint8_t index, uint16_t* distance);
bool ray_game_enemy_view(
    const RayGame_t* game,
    uint8_t index,
    int16_t* center_x,
    int16_t* width,
    int16_t* height,
    int16_t* bottom_y,
    uint16_t* depth
);
bool ray_game_shoot(RayGame_t* game);
void ray_game_tick(RayGame_t* game);

#endif
