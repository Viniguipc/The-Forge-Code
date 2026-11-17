#include "map.h"
#include <math.h>
#include <string.h>

const Room MAP_ROOMS[MAX_ROOMS] = {
    {0, "Enfermaria",  40.0f,  40.0f,  230.0f, 190.0f, 155.0f, 135.0f},
    {1, "Refeitorio",  370.0f, 40.0f,  260.0f, 190.0f, 500.0f, 135.0f},
    {2, "Navegacao",   730.0f, 40.0f,  230.0f, 190.0f, 845.0f, 135.0f},
    {3, "Seguranca",   40.0f,  370.0f, 230.0f, 190.0f, 155.0f, 465.0f},
    {4, "Eletrica",    370.0f, 370.0f, 260.0f, 190.0f, 500.0f, 465.0f},
    {5, "Reator",      730.0f, 370.0f, 230.0f, 190.0f, 845.0f, 465.0f}
};

const TaskPoint MAP_TASKS[MAX_TASKS] = {
    {0, 0, "Escanear Amostra",     90.0f,  150.0f},
    {1, 0, "Analisar Dados",       220.0f, 95.0f},
    {2, 1, "Esvaziar Lixo",        430.0f, 90.0f},
    {3, 1, "Baixar Relatorio",     575.0f, 95.0f},
    {4, 2, "Ajustar Rota",         890.0f, 155.0f},
    {5, 2, "Calibrar Antena",      780.0f, 95.0f},
    {6, 3, "Checar Cameras",       210.0f, 500.0f},
    {7, 4, "Religar Fios",         430.0f, 500.0f}, // Also lights fix location
    {8, 4, "Carregar Bateria",     560.0f, 420.0f},
    {9, 5, "Estabilizar Reator",   885.0f, 495.0f}
};

const Vent MAP_VENTS[MAX_VENTS] = {
    {0, 0, 80.0f,  85.0f,  1},
    {1, 3, 80.0f,  515.0f, 2},
    {2, 4, 590.0f, 515.0f, 3},
    {3, 2, 920.0f, 85.0f,  0}
};

// Wall segments (leaving 60px doorways at [125..185], [470..530], [815..875])
const WallSeg MAP_WALLS[] = {
    // Enfermaria (0): 40..270 x 40..230
    {40, 40, 270, 40}, {40, 40, 40, 230}, {270, 40, 270, 230},
    {40, 230, 125, 230}, {185, 230, 270, 230},

    // Refeitorio (1): 370..630 x 40..230
    {370, 40, 630, 40}, {370, 40, 370, 230}, {630, 40, 630, 230},
    {370, 230, 470, 230}, {530, 230, 630, 230},

    // Navegacao (2): 730..960 x 40..230
    {730, 40, 960, 40}, {730, 40, 730, 230}, {960, 40, 960, 230},
    {730, 230, 815, 230}, {875, 230, 960, 230},

    // Seguranca (3): 40..270 x 370..560
    {40, 560, 270, 560}, {40, 370, 40, 560}, {270, 370, 270, 560},
    {40, 370, 125, 370}, {185, 370, 270, 370},

    // Eletrica (4): 370..630 x 370..560
    {370, 560, 630, 560}, {370, 370, 370, 560}, {630, 370, 630, 560},
    {370, 370, 470, 370}, {530, 370, 630, 370},

    // Reator (5): 730..960 x 370..560
    {730, 560, 960, 560}, {730, 370, 730, 560}, {960, 370, 960, 560},
    {730, 370, 815, 370}, {875, 370, 960, 370}
};

const int MAP_WALL_COUNT = (int)(sizeof(MAP_WALLS) / sizeof(MAP_WALLS[0]));

const Waypoint MAP_WAYPOINTS[15] = {
    // 0..5: room centers
    {155, 135, 0}, {500, 135, 1}, {845, 135, 2},
    {155, 465, 3}, {500, 465, 4}, {845, 465, 5},
    // 6..11: doorways
    {155, 230, -1}, {500, 230, -1}, {845, 230, -1},
    {155, 370, -1}, {500, 370, -1}, {845, 370, -1},
    // 12..14: hallway spine
    {155, 300, -1}, {500, 300, -1}, {845, 300, -1}
};

static bool adj[15][15];

void map_init(void) {
    memset(adj, 0, sizeof(adj));
    int edges[][2] = {
        {0, 6}, {6, 12},
        {1, 7}, {7, 13},
        {2, 8}, {8, 14},
        {3, 9}, {9, 12},
        {4, 10}, {10, 13},
        {5, 11}, {11, 14},
        {12, 13}, {13, 14}
    };
    int n = (int)(sizeof(edges) / sizeof(edges[0]));
    for (int i = 0; i < n; i++) {
        int a = edges[i][0], b = edges[i][1];
        adj[a][b] = true;
        adj[b][a] = true;
    }
}

float map_dist(float x1, float y1, float x2, float y2) {
    float dx = x2 - x1;
    float dy = y2 - y1;
    return sqrtf(dx * dx + dy * dy);
}

int map_room_at(float x, float y) {
    for (int i = 0; i < MAX_ROOMS; i++) {
        const Room *r = &MAP_ROOMS[i];
        if (x >= r->x && x <= r->x + r->w && y >= r->y && y <= r->y + r->h) {
            return r->id;
        }
    }
    return -1;
}

static bool seg_intersect(float ax, float ay, float bx, float by,
                          float cx, float cy, float dx, float dy) {
    float r_x = bx - ax, r_y = by - ay;
    float s_x = dx - cx, s_y = dy - cy;
    float denom = r_x * s_y - r_y * s_x;
    if (fabsf(denom) < 1e-6f) return false;
    float qpx = cx - ax, qpy = cy - ay;
    float t = (qpx * s_y - qpy * s_x) / denom;
    float u = (qpx * r_y - qpy * r_x) / denom;
    return (t > 0.001f && t < 0.999f && u >= 0.0f && u <= 1.0f);
}

bool map_has_los(float x1, float y1, float x2, float y2) {
    for (int i = 0; i < MAP_WALL_COUNT; i++) {
        const WallSeg *w = &MAP_WALLS[i];
        if (seg_intersect(x1, y1, x2, y2, w->x1, w->y1, w->x2, w->y2)) {
            return false;
        }
    }
    return true;
}

int map_nearest_waypoint(float x, float y) {
    int best = 0;
    float best_d = 1e9f;
    for (int i = 0; i < 15; i++) {
        float d = map_dist(x, y, MAP_WAYPOINTS[i].x, MAP_WAYPOINTS[i].y);
        if (d < best_d) {
            best_d = d;
            best = i;
        }
    }
    return best;
}

int map_room_waypoint(int room_id) {
    if (room_id >= 0 && room_id < MAX_ROOMS) return room_id;
    return 13; // central hallway
}

int map_find_path(int start_wp, int goal_wp, int out_path[MAX_WAYPOINTS]) {
    if (start_wp == goal_wp) {
        out_path[0] = goal_wp;
        return 1;
    }
    int parent[15];
    bool vis[15];
    for (int i = 0; i < 15; i++) {
        parent[i] = -1;
        vis[i] = false;
    }
    int q[15], head = 0, tail = 0;
    q[tail++] = start_wp;
    vis[start_wp] = true;

    while (head < tail) {
        int cur = q[head++];
        if (cur == goal_wp) break;
        for (int nxt = 0; nxt < 15; nxt++) {
            if (adj[cur][nxt] && !vis[nxt]) {
                vis[nxt] = true;
                parent[nxt] = cur;
                q[tail++] = nxt;
            }
        }
    }
    if (!vis[goal_wp]) return 0;

    int rev[15], len = 0;
    int cur = goal_wp;
    while (cur != -1) {
        rev[len++] = cur;
        cur = parent[cur];
    }
    for (int i = 0; i < len; i++) {
        out_path[i] = rev[len - 1 - i];
    }
    return len;
}
