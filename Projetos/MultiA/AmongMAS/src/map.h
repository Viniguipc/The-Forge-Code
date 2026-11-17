#ifndef MAP_H
#define MAP_H

#include "types.h"

#define GRID_CELL 10.0f
#define GRID_W 100
#define GRID_H 60

extern const Room MAP_ROOMS[MAX_ROOMS];
extern const TaskPoint MAP_TASKS[MAX_TASKS];
extern const Vent MAP_VENTS[MAX_VENTS];
extern const WallSeg MAP_WALLS[];
extern const int MAP_WALL_COUNT;

void map_init(void);
int map_room_at(float x, float y);
bool map_has_los(float x1, float y1, float x2, float y2);
int map_astar(float sx, float sy, float gx, float gy,
              float out_x[MAX_ASTAR_PATH], float out_y[MAX_ASTAR_PATH]);
float map_dist(float x1, float y1, float x2, float y2);

#endif
