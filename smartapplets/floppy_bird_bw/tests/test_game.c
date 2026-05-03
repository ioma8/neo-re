#include "../game.h"

#include <assert.h>
#include <stdbool.h>

static void new_game_starts_centered_with_zero_score(void) {
    BirdGame_t game;
    bird_game_init(&game);

    assert(game.bird_y_q8 == BIRD_START_Y_Q8);
    assert(game.score == 0);
    assert(!game.game_over);
}

static void game_uses_full_lcd_pixel_world(void) {
    assert(BIRD_COLS == 264);
    assert(BIRD_ROWS == 64);
    assert(BIRD_START_Y_Q8 == 32 * BIRD_Q8);
}

static void flap_moves_bird_up_against_gravity(void) {
    BirdGame_t game;
    bird_game_init(&game);
    int16_t before = game.bird_y_q8;

    bird_game_flap(&game);
    bird_game_tick(&game);

    assert(game.bird_y_q8 < before);
}

static void passing_barrier_increases_score(void) {
    BirdGame_t game;
    bird_game_init(&game);
    game.barrier_x = BIRD_X - 8;
    game.passed_barrier = false;

    bird_game_tick(&game);

    assert(game.score == 1);
    assert(game.passed_barrier);
}

static void barrier_is_wide_with_pixel_gap(void) {
    BirdGame_t game;
    bird_game_init(&game);
    game.barrier_x = 80;
    game.gap_row = 20;

    assert(bird_game_barrier_at(&game, 80, 8));
    assert(bird_game_barrier_at(&game, 87, 8));
    assert(!bird_game_barrier_at(&game, 88, 8));
    assert(!bird_game_barrier_at(&game, 82, 20));
    assert(!bird_game_barrier_at(&game, 82, 39));
    assert(bird_game_barrier_at(&game, 82, 42));
}

static void collision_sets_game_over(void) {
    BirdGame_t game;
    bird_game_init(&game);
    game.barrier_x = BIRD_X + 2;
    game.gap_row = 8;
    game.bird_y_q8 = 40 * BIRD_Q8;

    bird_game_tick(&game);

    assert(game.game_over);
}

int main(void) {
    new_game_starts_centered_with_zero_score();
    game_uses_full_lcd_pixel_world();
    flap_moves_bird_up_against_gravity();
    passing_barrier_increases_score();
    barrier_is_wide_with_pixel_gap();
    collision_sets_game_over();
    return 0;
}
