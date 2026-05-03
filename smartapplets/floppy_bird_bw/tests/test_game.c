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
    game.barrier_x = BIRD_X - 1;
    game.passed_barrier = false;

    bird_game_tick(&game);

    assert(game.score == 1);
    assert(game.passed_barrier);
}

static void collision_sets_game_over(void) {
    BirdGame_t game;
    bird_game_init(&game);
    game.barrier_x = BIRD_X;
    game.gap_row = 0;
    game.bird_y_q8 = 3 * BIRD_Q8;

    bird_game_tick(&game);

    assert(game.game_over);
}

int main(void) {
    new_game_starts_centered_with_zero_score();
    flap_moves_bird_up_against_gravity();
    passing_barrier_increases_score();
    collision_sets_game_over();
    return 0;
}
