#include "../game.h"

#include <assert.h>
#include <stdio.h>

static void place(RayGame_t* game, int16_t x, int16_t y, uint8_t angle) {
    game->player_x = x;
    game->player_y = y;
    game->angle = angle;
}

static void spawn_is_inside_the_hall(void) {
    RayGame_t game;
    ray_game_init(&game);
    assert(game.player_x == 8 * RAY_Q8 + RAY_Q8 / 2);
    assert(game.player_y == 8 * RAY_Q8 + RAY_Q8 / 2);
    assert(game.angle == RAY_ANGLE_NORTH);
    assert(game.enemies[0].alive == 1);
    assert(game.enemies[0].x == 8 && game.enemies[0].y == 4);
}

static void center_ray_hits_the_back_wall(void) {
    RayGame_t game;
    ray_game_init(&game);
    RayHit_t hit = ray_game_cast_center(&game);
    assert(hit.hit);
    assert(hit.distance > 400);
    assert(hit.wall_x == 8);
    assert(hit.wall_y == 2);
}

static void forward_moves_player_when_open(void) {
    RayGame_t game;
    ray_game_init(&game);
    int16_t y = game.player_y;
    ray_game_move(&game, RAY_MOVE_FORWARD);
    assert(game.player_y == y - RAY_MOVE_STEP);
}

static void movement_step_is_responsive(void) {
    RayGame_t game;
    ray_game_init(&game);
    assert(RAY_MOVE_STEP >= 48);
}

static void turning_step_is_responsive(void) {
    RayGame_t game;
    ray_game_init(&game);
    uint8_t angle = game.angle;
    ray_game_turn_right(&game);
    assert(game.angle == ((angle + RAY_TURN_STEP) & 255));
    ray_game_turn_left(&game);
    assert(game.angle == angle);
}

static void walls_block_player(void) {
    RayGame_t game;
    ray_game_init(&game);
    place(&game, 8 * RAY_Q8 + RAY_Q8 / 2, 3 * RAY_Q8 + 80, RAY_ANGLE_NORTH);
    int16_t y = game.player_y;
    ray_game_move(&game, RAY_MOVE_FORWARD);
    assert(game.player_y == y);
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
    place(&game, 4 * RAY_Q8 + RAY_Q8 / 2, 7 * RAY_Q8 + RAY_Q8 / 2, RAY_ANGLE_EAST);
    game.enemies[0].x = 6;
    game.enemies[0].y = 7;
    game.enemies[0].alive = 1;
    assert(ray_game_shoot(&game));
    assert(game.enemies[0].alive == 0);
    assert(game.score == 1);
    assert(game.muzzle_flash != 0);
}

static void walls_block_shots(void) {
    RayGame_t game;
    ray_game_init(&game);
    place(&game, 4 * RAY_Q8 + RAY_Q8 / 2, 4 * RAY_Q8 + RAY_Q8 / 2, RAY_ANGLE_EAST);
    game.enemies[0].x = 6;
    game.enemies[0].y = 4;
    game.enemies[0].alive = 1;
    assert(!ray_game_shoot(&game));
    assert(game.enemies[0].alive == 1);
}

static void enemies_behind_the_player_are_not_shot(void) {
    RayGame_t game;
    ray_game_init(&game);
    place(&game, 7 * RAY_Q8 + RAY_Q8 / 2, 7 * RAY_Q8 + RAY_Q8 / 2, RAY_ANGLE_EAST);
    game.enemies[0].x = 5;
    game.enemies[0].y = 7;
    game.enemies[0].alive = 1;
    assert(!ray_game_shoot(&game));
    assert(game.enemies[0].alive == 1);
}

static void off_axis_enemies_are_not_shot(void) {
    RayGame_t game;
    ray_game_init(&game);
    place(&game, 4 * RAY_Q8 + RAY_Q8 / 2, 7 * RAY_Q8 + RAY_Q8 / 2, RAY_ANGLE_EAST);
    game.enemies[0].x = 6;
    game.enemies[0].y = 9;
    game.enemies[0].alive = 1;
    assert(!ray_game_shoot(&game));
    assert(game.enemies[0].alive == 1);
}


static void close_enemy_still_projects(void) {
    RayGame_t game;
    int16_t cx, w, h, by;
    uint16_t dep;
    ray_game_init(&game);
    place(&game, 4 * RAY_Q8 + 96, 7 * RAY_Q8 + RAY_Q8 / 2, RAY_ANGLE_EAST);
    game.enemies[0].x = 4;
    game.enemies[0].y = 7;
    game.enemies[0].alive = 1;
    assert(ray_game_enemy_view(&game, 0, &cx, &w, &h, &by, &dep));
    assert(w > 0 && h > 0);
    assert(dep == 32);
    assert(by <= RAY_VIEW_H);
    assert(by - h < RAY_VIEW_H);
}

static void out_of_bounds_walls_are_solid(void) {
    assert(ray_game_wall_type(255, 255) == 1);
    assert(ray_game_wall_type(0, 0) == 1);
}

int main(void) {
    spawn_is_inside_the_hall();
    center_ray_hits_the_back_wall();
    forward_moves_player_when_open();
    movement_step_is_responsive();
    turning_step_is_responsive();
    walls_block_player();
    shallow_rays_advance_both_axes();
    shooting_visible_enemy_removes_it();
    walls_block_shots();
    enemies_behind_the_player_are_not_shot();
    off_axis_enemies_are_not_shot();
    close_enemy_still_projects();
    out_of_bounds_walls_are_solid();
    puts("doom game tests passed");
    return 0;
}
