#include "../game.h"

#include <assert.h>

static void starts_moving_right_with_three_segments(void) {
    SnakeGame_t game;
    snake_game_init(&game);

    assert(game.length == 3);
    assert(game.score == 0);
    assert(game.direction == SNAKE_RIGHT);
    assert(game.head_x == SNAKE_COLS / 2);
    assert(game.head_y == SNAKE_ROWS / 2);
}

static void uses_200_pixel_playfield(void) {
    assert(SNAKE_COLS == 50);
    assert(SNAKE_ROWS == 16);
}

static void seeded_starts_place_food_differently(void) {
    SnakeGame_t first;
    SnakeGame_t second;
    snake_game_init_seeded(&first, 31);
    snake_game_init_seeded(&second, 32);

    assert(first.food_x != second.food_x || first.food_y != second.food_y);
}

static void ignores_instant_reverse(void) {
    SnakeGame_t game;
    snake_game_init(&game);

    snake_game_turn(&game, SNAKE_LEFT);

    assert(game.pending_direction == SNAKE_RIGHT);
}

static void eating_food_grows_and_scores(void) {
    SnakeGame_t game;
    snake_game_init(&game);
    game.food_x = game.head_x + 1;
    game.food_y = game.head_y;

    snake_game_tick(&game);

    assert(game.score == 1);
    assert(game.length == 4);
    assert(!game.game_over);
}

static void pause_stops_movement(void) {
    SnakeGame_t game;
    snake_game_init(&game);
    uint8_t x = game.head_x;

    snake_game_toggle_pause(&game);
    snake_game_tick(&game);

    assert(game.paused);
    assert(game.head_x == x);
}

static void restart_restores_initial_state(void) {
    SnakeGame_t game;
    snake_game_init(&game);
    game.score = 9;
    game.game_over = true;

    snake_game_restart(&game);

    assert(game.score == 0);
    assert(!game.game_over);
    assert(game.length == 3);
}

static void moving_past_right_edge_wraps_to_left(void) {
    SnakeGame_t game;
    snake_game_init(&game);
    game.head_x = SNAKE_COLS - 1;
    game.direction = SNAKE_RIGHT;
    game.pending_direction = SNAKE_RIGHT;

    snake_game_tick(&game);

    assert(!game.game_over);
    assert(game.head_x == 0);
}

int main(void) {
    starts_moving_right_with_three_segments();
    uses_200_pixel_playfield();
    seeded_starts_place_food_differently();
    ignores_instant_reverse();
    eating_food_grows_and_scores();
    pause_stops_movement();
    restart_restores_initial_state();
    moving_past_right_edge_wraps_to_left();
    return 0;
}
