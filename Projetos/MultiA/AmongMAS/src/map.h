#ifndef MAP_H
#define MAP_H

#include "types.h"

extern const Room MAP_ROOMS[MAX_ROOMS];
extern const TaskPoint MAP_TASKS[MAX_TASKS];
extern const Vent MAP_VENTS[MAX_VENTS];
extern const WallSeg MAP_WALLS[];
extern const int MAP_WALL_COUNT;

typedef struct {
    float x, y;
    int room_id; // -1 if hallway
} Waypoint;

extern const Waypoint MAP_WAYPOINTS[15];

void map_init(void);
int map_room_at(float x, float y);
bool map_has_los(float x1, float y1, float x2, float y2);
int map_nearest_waypoint(float x, float y);
int map_find_path(int start_wp, int goal_wp, int out_path[MAX_WAYPOINTS]);
int map_room_waypoint(int room_id);
float map_dist(float x1, float y1, float x2, float y2);

#endif
