#ifndef CONTROLLER_RECORDED_TAS_H
#define CONTROLLER_RECORDED_TAS_H

#include "controller_api.h"
#include <stdint.h>
#include <stdio.h>

extern struct ControllerAPI controller_recorded_tas;
void true_tas_init(char supplied_filename[FILENAME_MAX], char info_filename[FILENAME_MAX]);
float get_speed();

void rng_update(uint32_t input);
uint32_t rng_next();
void exit_game(int code);


#endif
