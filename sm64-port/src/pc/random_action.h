#include <stdint.h>
#include <SDL2/SDL_stdinc.h>
#include <stdint.h>
#include <ultra64.h>

void rng_init(uint32_t seed, uint32_t max_random_action, uint32_t max_window_length, float A_prob, float B_prob, float Z_prob);
int is_random_action();
void rng_update(uint32_t input);
OSContPad *rng_pad(uint16_t button, uint8_t stick_x, uint8_t stick_y);
