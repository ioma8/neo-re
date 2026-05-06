#ifndef DOOM_GAME_H
#define DOOM_GAME_H

#include <stdbool.h>
#include <stdint.h>

enum {
    RAY_Q8 = 256,
    RAY_MAP_W = 8,
    RAY_MAP_H = 8,
    RAY_ANGLE_COUNT = 64,
    RAY_ANGLE_EAST = 0,
    RAY_ANGLE_SOUTH = 16,
    RAY_ANGLE_WEST = 32,
    RAY_ANGLE_NORTH = 48
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
} RayHit_t;

typedef struct {
    int16_t player_x;
    int16_t player_y;
    uint8_t angle;
    uint8_t enemy_x;
    uint8_t enemy_y;
    uint8_t enemy_alive;
} RayGame_t;

void ray_game_init(RayGame_t* game);
void ray_game_turn_left(RayGame_t* game);
void ray_game_turn_right(RayGame_t* game);
void ray_game_move(RayGame_t* game, RayMove_t move);
RayHit_t ray_game_cast_center(const RayGame_t* game);
RayHit_t ray_game_cast(const RayGame_t* game, int8_t angle_offset);
RayHit_t ray_game_cast_vector(const RayGame_t* game, int16_t ray_x, int16_t ray_y);
int16_t ray_game_dir_x(const RayGame_t* game);
int16_t ray_game_dir_y(const RayGame_t* game);
bool ray_game_wall_at_cell(uint8_t x, uint8_t y);
bool ray_game_enemy_visible(const RayGame_t* game, uint16_t* distance);
bool ray_game_shoot(RayGame_t* game);

#endif
