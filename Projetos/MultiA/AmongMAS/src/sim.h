#ifndef SIM_H
#define SIM_H

#include "types.h"

void sim_init(SimState *s, int impostor_count);
void sim_reset(SimState *s, int impostor_count);
void sim_step(SimState *s);
void sim_sabotage_lights(SimState *s);
int sim_serialize_json(const SimState *s, char *buf, int buf_size);

#endif
