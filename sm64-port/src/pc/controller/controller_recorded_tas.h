#ifndef CONTROLLER_RECORDED_TAS_H
#define CONTROLLER_RECORDED_TAS_H

#include "controller_api.h"
#include <stdint.h>
#include <stdio.h>

extern struct ControllerAPI controller_recorded_tas;
void set_tas_controller(uint16_t b, uint8_t x, uint8_t y);

#endif
