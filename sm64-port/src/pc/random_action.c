#include "random_action.h"


typedef struct {
    // Use a seed for randomness
    uint32_t seed;
    uint32_t max_window_length;
    uint32_t max_random_action;
    uint32_t window_cur_amount;
    uint32_t random_cur_amount;
    float A_prob;
    float B_prob;
    float Z_prob;
} RandomAction;

static RandomAction rng = {
    22, 100, 5, 0, 0, 0.5, 0.5, 0.2
};

uint32_t rng_next() {
    // this is xorshift32
    uint32_t x = rng.seed;
    x ^= (x << 13);
    x ^= (x >> 17);
    x ^= (x << 5);
    rng.seed = x;
    return x;
}

float rng_next_prob() {
    // between 0 and 1 
    return rng_next() / (float)UINT32_MAX;
}
int8_t rng_next_stick() {
    // between -80 and 80 inclusive
    return (int8_t)(rng_next() % 161) - 80;
}

OSContPad *rng_pad(uint16_t button, uint8_t stick_x, uint8_t stick_y) {
    static OSContPad pad = {0};
    pad.errnum = 0;

    if (!is_random_action()) {
        pad.button = button;
        pad.stick_x = stick_x;
        pad.stick_y = stick_y;
        pad.errnum = 1;

        return &pad;
    }

    pad.button |= A_BUTTON * (rng_next_prob() < rng.A_prob);
    pad.button |= B_BUTTON * (rng_next_prob() < rng.B_prob);
    pad.button |= Z_TRIG * (rng_next_prob() < rng.Z_prob);

    pad.stick_x = rng_next_stick();
    pad.stick_y = rng_next_stick();

    return &pad;
}

void rng_update(uint32_t input) {
    rng.seed = rng_next() ^ input;
    rng.seed = rng_next();
}


void rng_init(uint32_t seed, uint32_t max_random_action, uint32_t max_window_length, float A_prob, float B_prob, float Z_prob) {
    rng.seed = seed;
    rng.max_window_length = max_window_length;
    rng.max_random_action = max_random_action;
    rng.window_cur_amount = 0;
    rng.random_cur_amount = 0;
    rng.A_prob = A_prob;
    rng.B_prob = B_prob;
    rng.Z_prob = Z_prob;
}

int is_random_action() {
    if (rng.window_cur_amount == 0) {
        // Reset the window length and random action amount
        rng.window_cur_amount = rng.max_window_length;
        rng.random_cur_amount = rng.max_random_action;
    }

    double prob_random = (double)rng.random_cur_amount / (double)rng.window_cur_amount;
    float r = rng_next_prob();
    int is_random = r < prob_random;
    if (is_random) {
        rng.random_cur_amount -= 1;
    }
    
    rng.window_cur_amount -= 1;

    return is_random;
}