# DOOM?

A small 1-bit raycaster for the AlphaSmart NEO's 264x64 LCD.

## Controls

| Key | Action |
| --- | --- |
| Up / W | Move forward |
| Down / S | Move backward |
| Left / A | Turn left |
| Right / D | Turn right |
| Space | Shoot |
| R | Restart |
| Esc / Applets | Exit |

Holding a direction queues one step. Steps run on a 120 ms timer so the
renderer stays inside the LCD/firmware budget.

## Map

One compact 10x10 stone hall with two pillars and three skulls. There are no
corridors or side rooms to get lost in; the whole arena is visible from the
spawn point.

## Rendering

The world is rendered directly into a page-major 264x64 framebuffer and
blitted with `DrawBitmap(0, 0, 264, 64, fb)`. The renderer is intentionally
sparse:

- a DDA raycaster with perpendicular-depth wall projection at 1-pixel columns,
- solid black walls with white top/bottom edges; N/S faces get a 50% dither
  and E/W faces stay black, which reads as directional lighting,
- a very light gray stipple floor and a plain white ceiling,
- a billboard demon skull with a white contact halo,
- a gun, a short muzzle flash and a small inverted crosshair.

There is no minimap and no wall texture; the arena is simple enough to read at
a glance. A frame takes about 3.55M cycles (~108 ms at 33 MHz), so the step
timer is 120 ms.

The framebuffer lives at `state + 25`, where `state = A5 + 0x300`; this is
well below the firmware stack.

## Build and test

```sh
./scripts/build-smartapplet.sh doom_bw
cc -std=c99 -Wall -Wextra -I smartapplets/doom_bw \
  smartapplets/doom_bw/tests/test_game.c smartapplets/doom_bw/game.c \
  -o /tmp/test_doom && /tmp/test_doom
(cd alpha-emu && cargo run -- --headless --validate-doom \
  ../analysis/cab/os3kneorom.os3kos)
```

Host tests cover spawn, ray hits, movement, collision, turning, shooting,
line-of-sight, off-axis rejection, behind-the-player rejection and
out-of-bounds walls. The emulator validation covers focus, timer stepping,
turning, movement, view updates, shooting, restart and Escape exit.
