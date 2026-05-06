#include "../game.h"

#include <assert.h>

static void ray_hits_east_wall(void) {
    RayGame_t game;
    ray_game_init(&game);

    RayHit_t hit = ray_game_cast_center(&game);

    assert(hit.hit);
    assert(hit.distance > 400);
    assert(hit.wall_x == 7);
}

static void forward_moves_player_when_open(void) {
    RayGame_t game;
    ray_game_init(&game);
    int16_t x = game.player_x;

    ray_game_move(&game, RAY_MOVE_FORWARD);

    assert(game.player_x > x);
}

static void movement_step_is_responsive(void) {
    RayGame_t game;
    ray_game_init(&game);
    int16_t x = game.player_x;

    ray_game_move(&game, RAY_MOVE_FORWARD);

    assert(game.player_x - x > 20);
}

static void turning_step_is_responsive(void) {
    RayGame_t game;
    ray_game_init(&game);
    uint8_t angle = game.angle;

    ray_game_turn_right(&game);

    assert(game.angle == ((angle + 2) & 63));
}

static void walls_block_player(void) {
    RayGame_t game;
    ray_game_init(&game);
    game.player_x = 6 * RAY_Q8 + RAY_Q8 - 4;
    game.angle = RAY_ANGLE_EAST;

    ray_game_move(&game, RAY_MOVE_FORWARD);

    assert(game.player_x == 6 * RAY_Q8 + RAY_Q8 - 4);
}

static void player_has_wall_collision_radius(void) {
    RayGame_t game;
    ray_game_init(&game);
    game.player_x = 6 * RAY_Q8 + RAY_Q8 - 28;
    game.angle = RAY_ANGLE_EAST;

    ray_game_move(&game, RAY_MOVE_FORWARD);

    assert(game.player_x == 6 * RAY_Q8 + RAY_Q8 - 28);
}

static void shallow_rays_advance_both_axes(void) {
    RayGame_t game;
    ray_game_init(&game);

    RayHit_t hit = ray_game_cast_vector(&game, 12, -127);

    assert(hit.hit);
    assert(hit.wall_x > 4);
}

static void shooting_visible_enemy_removes_it(void) {
    RayGame_t game;
    ray_game_init(&game);
    game.player_x = 4 * RAY_Q8 + RAY_Q8 / 2;
    game.player_y = RAY_Q8 + RAY_Q8 / 2;
    game.angle = RAY_ANGLE_EAST;
    game.enemy_x = 6;
    game.enemy_y = 1;
    game.enemy_alive = 1;

    assert(ray_game_shoot(&game));
    assert(game.enemy_alive == 0);
}

static void walls_block_shots(void) {
    RayGame_t game;
    ray_game_init(&game);
    game.player_x = 4 * RAY_Q8 + RAY_Q8 / 2;
    game.player_y = 2 * RAY_Q8 + RAY_Q8 / 2;
    game.angle = RAY_ANGLE_EAST;
    game.enemy_x = 6;
    game.enemy_y = 2;
    game.enemy_alive = 1;

    assert(!ray_game_shoot(&game));
    assert(game.enemy_alive == 1);
}

int main(void) {
    ray_hits_east_wall();
    forward_moves_player_when_open();
    movement_step_is_responsive();
    turning_step_is_responsive();
    walls_block_player();
    player_has_wall_collision_radius();
    shallow_rays_advance_both_axes();
    shooting_visible_enemy_removes_it();
    walls_block_shots();
    return 0;
}
