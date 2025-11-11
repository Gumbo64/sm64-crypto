#include <stdint.h>
#include <stdio.h>
#include <sys/stat.h>
#include <stdlib.h>
#include <string.h>
#include <ultra64.h>
#include <unistd.h>

#include "controller_api.h"

static uint16_t button = 0;
static int8_t stick_x = 0;
static int8_t stick_y = 0;

void set_tas_controller(uint16_t b, int8_t x, int8_t y) {
    button = b;
    stick_x = x;
    stick_y = y;
}

static void tas_init(void) {}
static void tas_read(OSContPad *pad) {
    pad->button = button;
    pad->stick_x = stick_x;
    pad->stick_y = stick_y;
    pad->errnum = 0;
}

struct ControllerAPI controller_recorded_tas = {
    tas_init,
    tas_read
};