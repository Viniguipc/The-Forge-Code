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
    {7, 4, "Religar Fios",         430.0f, 500.0f},
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

static uint8_t grid_blocked[GRID_H][GRID_W];

static float point_seg_dist(float px, float py, float x1, float y1, float x2, float y2) {
    float vx = x2 - x1, vy = y2 - y1;
    float wx = px - x1, wy = py - y1;
    float c1 = wx * vx + wy * vy;
    if (c1 <= 0.0f) return sqrtf(wx * wx + wy * wy);
    float c2 = vx * vx + vy * vy;
    if (c2 <= c1) {
        float dx = px - x2, dy = py - y2;
        return sqrtf(dx * dx + dy * dy);
    }
    float b = c1 / c2;
    float bx = x1 + b * vx, by = y1 + b * vy;
    float dx = px - bx, dy = py - by;
    return sqrtf(dx * dx + dy * dy);
}

void map_init(void) {
    for (int r = 0; r < GRID_H; r++) {
        for (int c = 0; c < GRID_W; c++) {
            float cx = c * GRID_CELL + GRID_CELL * 0.5f;
            float cy = r * GRID_CELL + GRID_CELL * 0.5f;
            uint8_t blk = 0;
            // Outside station bounds
            if (cx < 45.0f || cx > 955.0f || cy < 45.0f || cy > 555.0f) {
                blk = 1;
            } else {
                for (int i = 0; i < MAP_WALL_COUNT; i++) {
                    const WallSeg *w = &MAP_WALLS[i];
                    if (point_seg_dist(cx, cy, w->x1, w->y1, w->x2, w->y2) < 8.5f) {
                        blk = 1;
                        break;
                    }
                }
            }
            grid_blocked[r][c] = blk;
        }
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

static void snap_to_walkable(float x, float y, int *out_c, int *out_r) {
    int c = (int)(x / GRID_CELL);
    int r = (int)(y / GRID_CELL);
    if (c < 0) c = 0;
    if (c >= GRID_W) c = GRID_W - 1;
    if (r < 0) r = 0;
    if (r >= GRID_H) r = GRID_H - 1;
    if (!grid_blocked[r][c]) {
        *out_c = c; *out_r = r;
        return;
    }
    for (int rad = 1; rad <= 5; rad++) {
        for (int dr = -rad; dr <= rad; dr++) {
            for (int dc = -rad; dc <= rad; dc++) {
                int nr = r + dr, nc = c + dc;
                if (nr >= 0 && nr < GRID_H && nc >= 0 && nc < GRID_W && !grid_blocked[nr][nc]) {
                    *out_c = nc; *out_r = nr;
                    return;
                }
            }
        }
    }
    *out_c = c; *out_r = r;
}

typedef struct {
    int idx;
    float f;
} HeapNode;

static void heap_push(HeapNode *h, int *sz, int idx, float f) {
    int i = (*sz)++;
    h[i].idx = idx;
    h[i].f = f;
    while (i > 0) {
        int p = (i - 1) / 2;
        if (h[p].f <= h[i].f) break;
        HeapNode tmp = h[p]; h[p] = h[i]; h[i] = tmp;
        i = p;
    }
}

static HeapNode heap_pop(HeapNode *h, int *sz) {
    HeapNode top = h[0];
    h[0] = h[--(*sz)];
    int i = 0;
    for (;;) {
        int l = 2 * i + 1, r = 2 * i + 2, best = i;
        if (l < *sz && h[l].f < h[best].f) best = l;
        if (r < *sz && h[r].f < h[best].f) best = r;
        if (best == i) break;
        HeapNode tmp = h[i]; h[i] = h[best]; h[best] = tmp;
        i = best;
    }
    return top;
}

static float heuristic(int c1, int r1, int c2, int r2) {
    float dx = fabsf((float)(c2 - c1));
    float dy = fabsf((float)(r2 - r1));
    float mn = dx < dy ? dx : dy;
    float mx = dx > dy ? dx : dy;
    return 1.4142f * mn + (mx - mn);
}

int map_astar(float sx, float sy, float gx, float gy,
              float out_x[MAX_ASTAR_PATH], float out_y[MAX_ASTAR_PATH]) {
    int sc, sr, gc, gr;
    snap_to_walkable(sx, sy, &sc, &sr);
    snap_to_walkable(gx, gy, &gc, &gr);

    int start = sr * GRID_W + sc;
    int goal = gr * GRID_W + gc;
    if (start == goal) {
        out_x[0] = gx;
        out_y[0] = gy;
        return 1;
    }

    const int total = GRID_W * GRID_H;
    static float g_score[GRID_W * GRID_H];
    static int parent[GRID_W * GRID_H];
    static uint8_t closed[GRID_W * GRID_H];
    static HeapNode heap[GRID_W * GRID_H * 2];

    for (int i = 0; i < total; i++) {
        g_score[i] = 1e9f;
        parent[i] = -1;
        closed[i] = 0;
    }

    int heap_sz = 0;
    g_score[start] = 0.0f;
    heap_push(heap, &heap_sz, start, heuristic(sc, sr, gc, gr));

    const int dc[8] = {1, -1, 0, 0, 1, 1, -1, -1};
    const int dr[8] = {0, 0, 1, -1, 1, -1, 1, -1};
    const float cost[8] = {1.0f, 1.0f, 1.0f, 1.0f, 1.4142f, 1.4142f, 1.4142f, 1.4142f};

    bool found = false;
    while (heap_sz > 0) {
        HeapNode cur = heap_pop(heap, &heap_sz);
        int u = cur.idx;
        if (closed[u]) continue;
        closed[u] = 1;
        if (u == goal) {
            found = true;
            break;
        }
        int uc = u % GRID_W;
        int ur = u / GRID_W;
        for (int k = 0; k < 8; k++) {
            int nc = uc + dc[k];
            int nr = ur + dr[k];
            if (nc < 0 || nc >= GRID_W || nr < 0 || nr >= GRID_H) continue;
            if (grid_blocked[nr][nc]) continue;
            // Prevent diagonal corner cutting
            if (k >= 4 && (grid_blocked[ur][nc] || grid_blocked[nr][uc])) continue;

            int v = nr * GRID_W + nc;
            if (closed[v]) continue;
            float ng = g_score[u] + cost[k];
            if (ng < g_score[v]) {
                g_score[v] = ng;
                parent[v] = u;
                heap_push(heap, &heap_sz, v, ng + heuristic(nc, nr, gc, gr));
            }
        }
    }

    if (!found) return 0;

    static int rev[GRID_W * GRID_H];
    int len = 0;
    int cur = goal;
    while (cur != -1 && cur != start) {
        rev[len++] = cur;
        cur = parent[cur];
    }

    // Subsample if longer than MAX_ASTAR_PATH
    int step = (len > MAX_ASTAR_PATH - 1) ? ((len + MAX_ASTAR_PATH - 2) / (MAX_ASTAR_PATH - 1)) : 1;
    int out_len = 0;
    for (int i = len - 1; i >= 0 && out_len < MAX_ASTAR_PATH - 1; i -= step) {
        int idx = rev[i];
        out_x[out_len] = (idx % GRID_W) * GRID_CELL + GRID_CELL * 0.5f;
        out_y[out_len] = (idx / GRID_W) * GRID_CELL + GRID_CELL * 0.5f;
        out_len++;
    }
    if (out_len < MAX_ASTAR_PATH) {
        out_x[out_len] = gx;
        out_y[out_len] = gy;
        out_len++;
    }
    return out_len;
}
