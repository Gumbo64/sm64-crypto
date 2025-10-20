#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <ultra64.h>
#include <unistd.h>

#include "controller_api.h"

#ifdef TARGET_WEB
#include <emscripten.h>
#include <emscripten/html5.h>
#endif

static FILE *fp = NULL;
static int is_finished_playback = 0;
static int record_mode = 0;

#ifdef HEADLESS_VERSION
static float speed = 100000;
#else
static float speed = 10;
#endif


const int max_playtime_sec = 20;
static uint32_t file_length = 0;

static char filename[FILENAME_MAX] = "cont.m64";

void exit_game(int code);
void trunc_seek() {
    if (ftruncate(fileno(fp), file_length) != 0) {
        perror("Failed to truncate the file");
        exit_game(1);
    }
    if (fseek(fp, file_length, SEEK_SET) != 0) {
        perror("Error seeking in file");
        exit_game(1);
    }
}
void exit_game(int code) {
    trunc_seek();
    fclose(fp);

    #ifdef TARGET_WEB
    // emscripten_cancel_main_loop();
    emscripten_force_exit(code);
    #endif
    exit(code);
}

typedef struct {
    uint32_t window_cur_amount;
    uint32_t window_length_max;
    uint32_t random_action_amount;
    uint32_t random_action_max;

    // Use a seed for randomness
    uint32_t state;
} RandomAction;

static RandomAction rng;
uint32_t rng_next() {
    // this is xorshift32
    uint32_t x = rng.state;
    x ^= (x << 13);
    x ^= (x >> 17);
    x ^= (x << 5);
    rng.state = x;
    return x;
}

// from 0 to 1
float rng_next_prob() {
    return rng_next() / (float)UINT32_MAX;
}

void rng_update(uint32_t input) {
    rng.state = rng_next() ^ input;
    rng.state = rng_next();
}

void true_tas_init(char supplied_filename[FILENAME_MAX], int rec_mode, uint32_t seed) {
    record_mode = rec_mode;
    strncpy(filename, supplied_filename, FILENAME_MAX);

    rng.state = seed;
    rng.window_cur_amount = 0;
    rng.random_action_amount = 0;
    rng.window_length_max = 100;
    rng.random_action_max = 5;

    // Open/create the file in append and read mode
    fp = fopen(filename, "rb+");
    
    if (fp == NULL) {
        fp = fopen(filename, "wb+");
        if (fp == NULL) {
            exit_game(1);
        }
    }

}

float get_speed() {
    return speed;
}




static void tas_init(void) {}


static void playback_game(OSContPad *pad, OSContPad *rng_pad) {
    // Allow the player to take over at any point during playback
    if (record_mode && (pad->button & START_BUTTON)) {
        trunc_seek();
    }

    // Try to read the current pad state from the file
    uint8_t bytes[4] = {0};
    size_t bytesRead = fread(bytes, 1, sizeof(bytes), fp);
    // if we can read exactly 4 bytes then we use them. Otherwise start recording or fail if we are evalating
    if (bytesRead == sizeof(bytes)) {
        pad->button = (bytes[0] << 8) | bytes[1];
        pad->stick_x = bytes[2];
        pad->stick_y = bytes[3];

        // check that the playback follows the random input
        if (rng_pad != NULL && (pad->stick_x != rng_pad->stick_x || pad->stick_y != rng_pad->stick_y || pad->button != rng_pad->button)) {
            exit_game(1);
        }

        file_length += bytesRead;
    } else {
        // truncate file to the last reasonable length (in case there was between 1 and 3 bytes on last read)
        trunc_seek();

        is_finished_playback = 1;
        speed = 1;
        if (!record_mode) {
            exit_game(1); // failed to complete within the evaluation time
        }
        printf("FINISHED READING\n");
    }
}

static void record_game(OSContPad *pad, OSContPad *rng_pad) {
    // wait for the start button to release for at least one frame before actually registering start button presses
    // because we use the start button for manual takeover
    static int is_resuming = 1;
    if (!(pad->button & START_BUTTON)) {is_resuming = 0;}
    if (is_resuming) {pad->button &= ~START_BUTTON;}

    if (rng_pad != NULL) {
        pad->button = rng_pad->button;
        pad->stick_x = rng_pad->stick_x;
        pad->stick_y = rng_pad->stick_y;
    }
    
    // Writing the current pad state back to the file
    uint8_t bytes[4];
    bytes[0] = (pad->button >> 8) & 0xFF; // High byte of button state
    bytes[1] = pad->button & 0xFF;        // Low byte of button state
    bytes[2] = pad->stick_x;               // Stick X value
    bytes[3] = pad->stick_y;               // Stick Y value

    fwrite(bytes, 1, sizeof(bytes), fp);  // Append the 4 bytes to the file
    file_length += sizeof(bytes);

}

static OSContPad random_pad() {
    OSContPad pad = {0};
    pad.button |= A_BUTTON * (rng_next_prob() < 0.5);
    pad.button |= B_BUTTON * (rng_next_prob() < 0.5);
    pad.button |= Z_TRIG * (rng_next_prob() < 0.2);

    // between -80 and 80
    pad.stick_x = (int8_t)(rng_next() % 161) - 80;
    pad.stick_y = (int8_t)(rng_next() % 161) - 80;

    pad.errnum = 0;
    return pad;
}

static int is_random_action() {
    if (rng.window_cur_amount == 0) {
        // Reset the window length and random action amount
        rng.window_cur_amount = rng.window_length_max;
        rng.random_action_amount = rng.random_action_max;
    }

    double prob_random = (double)rng.random_action_amount / (double)rng.window_cur_amount;
    float r = rng_next_prob();
    int is_random = r < prob_random;
    if (is_random) {
        rng.random_action_amount -= 1;
    }
    
    rng.window_cur_amount -= 1;

    return is_random;
}

static void tas_read(OSContPad *pad) {
    if (fp == NULL) {
        return; // Early exit if not initialized or file is closed
    }

    if (file_length >= 4*30*max_playtime_sec) {
        exit_game(1);
    }

    OSContPad rng_pad;
    OSContPad* rng_pad_p = NULL;
    if (is_random_action()) {
        rng_pad = random_pad();
        rng_pad_p = &rng_pad;
    }

    if (!is_finished_playback) {
        // This function also decides when playback ends, so we might record immediately after this
        playback_game(pad, rng_pad_p);
    }
    if (is_finished_playback) {
        record_game(pad, rng_pad_p);
    }
}

struct ControllerAPI controller_recorded_tas = {
    tas_init,
    tas_read
};