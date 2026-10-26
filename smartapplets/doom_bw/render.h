#ifndef DOOM_RENDER_H
#define DOOM_RENDER_H

#include "game.h"

void ray_render_frame(
    const RayGame_t* game,
    uint8_t* framebuffer,
    uint16_t* depth,
    int16_t* ray_x,
    int16_t* ray_y
);

#endif
