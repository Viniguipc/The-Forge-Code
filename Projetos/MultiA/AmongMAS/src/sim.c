#include "sim.h"
#include "map.h"
#include <math.h>
#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef struct {
    const char *name;
    const char *color;
    const char *symbol;
    Personality pers;
    float sus_mult;
    float vote_offset;
    float chaos_prob;
} AgentProfile;

#define POOL_SIZE 40

static const AgentProfile AGENT_POOL[POOL_SIZE] = {
    // 5 Astutos
    {"Vega",    "#38bdf8", "★", PERS_ASTUTO,     1.25f, -0.04f, 0.00f},
    {"Kael",    "#818cf8", "◈", PERS_ASTUTO,     1.15f,  0.02f, 0.05f},
    {"Nyx",     "#c084fc", "✦", PERS_ASTUTO,     1.30f, -0.06f, 0.02f},
    {"Soren",   "#2dd4bf", "❖", PERS_ASTUTO,     1.10f,  0.05f, 0.00f},
    {"Kira",    "#f43f5e", "✪", PERS_ASTUTO,     1.20f, -0.02f, 0.08f},
    // 5 Ceticos
    {"Orion",   "#64748b", "■", PERS_CETICO,     0.90f,  0.10f, 0.00f},
    {"Dante",   "#94a3b8", "▲", PERS_CETICO,     1.05f,  0.05f, 0.02f},
    {"Otto",    "#475569", "◆", PERS_CETICO,     0.85f,  0.14f, 0.00f},
    {"Boris",   "#78716c", "⬢", PERS_CETICO,     0.95f,  0.08f, 0.04f},
    {"Iris",    "#0ea5e9", "◎", PERS_CETICO,     1.10f,  0.03f, 0.00f},
    // 5 Burros
    {"Pipoca",  "#fbbf24", "●", PERS_BURRO,      0.75f, -0.05f, 0.35f},
    {"Bidu",    "#f59e0b", "◉", PERS_BURRO,      0.85f, -0.08f, 0.28f},
    {"Caco",    "#a3e635", "◌", PERS_BURRO,      0.70f,  0.02f, 0.40f},
    {"Guga",    "#fb923c", "△", PERS_BURRO,      0.90f, -0.04f, 0.22f},
    {"Zeca",    "#eab308", "□", PERS_BURRO,      0.80f,  0.04f, 0.30f},
    // 5 Brincalhoes
    {"Loki",    "#ec4899", "♠", PERS_BRINCALHAO, 0.95f, -0.06f, 0.42f},
    {"Roxy",    "#f472b6", "♥", PERS_BRINCALHAO, 1.05f, -0.03f, 0.30f},
    {"Tito",    "#e879f9", "♣", PERS_BRINCALHAO, 0.90f, -0.08f, 0.38f},
    {"Lola",    "#fb7185", "♦", PERS_BRINCALHAO, 1.10f,  0.00f, 0.25f},
    {"Pingo",   "#34d399", "☺", PERS_BRINCALHAO, 1.00f, -0.05f, 0.34f},
    // 5 Paranoicos
    {"Zed",     "#ef4444", "⚡", PERS_PARANOICO,  1.40f, -0.10f, 0.10f},
    {"Ragnar",  "#dc2626", "✖", PERS_PARANOICO,  1.30f, -0.08f, 0.05f},
    {"Igor",    "#f97316", "▲", PERS_PARANOICO,  1.45f, -0.12f, 0.12f},
    {"Fenix",   "#ea580c", "✹", PERS_PARANOICO,  1.25f, -0.05f, 0.04f},
    {"Tadeu",   "#e11d48", "✚", PERS_PARANOICO,  1.35f, -0.09f, 0.08f},
    // 5 Analiticos
    {"Atlas",   "#3b82f6", "⊕", PERS_ANALITICO,  1.20f,  0.02f, 0.00f},
    {"Maya",    "#06b6d4", "✧", PERS_ANALITICO,  1.15f, -0.02f, 0.02f},
    {"Juno",    "#22d3ee", "❄", PERS_ANALITICO,  1.25f,  0.04f, 0.00f},
    {"Yuki",    "#60a5fa", "✳", PERS_ANALITICO,  1.10f,  0.00f, 0.03f},
    {"Cleo",    "#a78bfa", "❂", PERS_ANALITICO,  1.30f,  0.01f, 0.01f},
    // 5 Sociais
    {"Luna",    "#10b981", "☀", PERS_SOCIAL,     1.05f, -0.03f, 0.03f},
    {"Bia",     "#4ade80", "✿", PERS_SOCIAL,     1.10f, -0.05f, 0.05f},
    {"Milo",    "#84cc16", "♪", PERS_SOCIAL,     0.95f,  0.02f, 0.06f},
    {"Sol",     "#fde047", "☼", PERS_SOCIAL,     1.15f, -0.04f, 0.02f},
    {"Vivi",    "#22c55e", "❀", PERS_SOCIAL,     1.00f,  0.00f, 0.04f},
    // 5 Medrosos
    {"Bento",   "#cbd5e1", "◐", PERS_MEDROSO,    1.15f,  0.06f, 0.05f},
    {"Neko",    "#d8b4fe", "◑", PERS_MEDROSO,    1.20f,  0.08f, 0.08f},
    {"Nora",    "#bae6fd", "◒", PERS_MEDROSO,    1.10f,  0.04f, 0.04f},
    {"Teca",    "#fed7aa", "◓", PERS_MEDROSO,    1.25f,  0.10f, 0.10f},
    {"Zara",    "#fbcfe8", "◔", PERS_MEDROSO,    1.05f,  0.02f, 0.03f}
};

static const char *SROLE_NAMES[] = {
    "Comum", "Medico", "Investigador", "Vigia", "Engenheiro", "Rastreador", "Prefeito",
    "Metamorfo", "Limpador", "Sabotador", "Silenciador", "Camaleao", "Bobo"
};

static float clampf(float v, float lo, float hi) {
    if (v < lo) return lo;
    if (v > hi) return hi;
    return v;
}

static int rand_range(int lo, int hi) {
    if (hi <= lo) return lo;
    return lo + (rand() % (hi - lo + 1));
}

static void shuffle_ints(int *arr, int n) {
    for (int i = n - 1; i > 0; i--) {
        int j = rand() % (i + 1);
        int tmp = arr[i];
        arr[i] = arr[j];
        arr[j] = tmp;
    }
}

static void push_event(SimState *s, EventKind kind, int sender, int target, const char *fmt, ...) {
    AclEvent *ev;
    if (s->event_count < MAX_EVENTS) {
        ev = &s->events[s->event_count++];
    } else {
        memmove(&s->events[0], &s->events[1], sizeof(AclEvent) * (MAX_EVENTS - 1));
        ev = &s->events[MAX_EVENTS - 1];
    }
    ev->id = s->next_event_id++;
    ev->tick = s->tick;
    ev->kind = kind;
    ev->sender_id = sender;
    ev->target_id = target;

    va_list args;
    va_start(args, fmt);
    vsnprintf(ev->text, sizeof(ev->text), fmt, args);
    va_end(args);
}

static void record_sighting(Agent *observer, int seen_id, int room_id, uint32_t tick, AgentActivity act) {
    Sighting *slot = &observer->memory[observer->memory_head];
    slot->agent_id = seen_id;
    slot->room_id = room_id;
    slot->tick = tick;
    slot->activity = act;
    slot->valid = true;
    observer->memory_head = (observer->memory_head + 1) % MAX_MEMORY;
}

static void set_agent_path_to(Agent *a, float tx, float ty) {
    a->goal_x = tx;
    a->goal_y = ty;
    a->path_len = map_astar(a->x, a->y, tx, ty, a->path_x, a->path_y);
    a->path_idx = 0;
}

static void move_agent_toward(const SimState *s, Agent *a, float tx, float ty, float speed) {
    float dx = tx - a->x;
    float dy = ty - a->y;
    float d = sqrtf(dx * dx + dy * dy);
    float nx, ny;
    if (d <= speed) {
        nx = tx;
        ny = ty;
    } else {
        nx = a->x + (dx / d) * speed;
        ny = a->y + (dy / d) * speed;
    }

    // Saboteur locked room check: cannot cross doorway of locked_room
    if (s->locked_room >= 0) {
        int cur_r = map_room_at(a->x, a->y);
        int nxt_r = map_room_at(nx, ny);
        if ((cur_r == s->locked_room && nxt_r != s->locked_room) ||
            (cur_r != s->locked_room && nxt_r == s->locked_room)) {
            a->vx = 0;
            a->vy = 0;
            return;
        }
    }

    if (map_has_los(a->x, a->y, nx, ny)) {
        a->vx = nx - a->x;
        a->vy = ny - a->y;
        a->x = nx;
        a->y = ny;
    } else if (map_has_los(a->x, a->y, nx, a->y)) {
        a->vx = nx - a->x;
        a->vy = 0;
        a->x = nx;
    } else if (map_has_los(a->x, a->y, a->x, ny)) {
        a->vx = 0;
        a->vy = ny - a->y;
        a->y = ny;
    } else {
        a->vx = 0;
        a->vy = 0;
    }
}

static void follow_path_step(const SimState *s, Agent *a, float speed) {
    float rem = speed;
    while (a->path_idx < a->path_len && rem > 0.5f) {
        float wx = a->path_x[a->path_idx];
        float wy = a->path_y[a->path_idx];
        float d = map_dist(a->x, a->y, wx, wy);
        if (d <= rem) {
            float ox = a->x, oy = a->y;
            move_agent_toward(s, a, wx, wy, d);
            if (a->x == ox && a->y == oy) break; // blocked by locked door
            rem -= d;
            a->path_idx++;
        } else {
            move_agent_toward(s, a, wx, wy, rem);
            rem = 0.0f;
        }
    }
}

static void assign_random_tasks(Agent *a) {
    int used[MAX_TASKS] = {0};
    for (int i = 0; i < MAX_TASKS_PER_AGENT; i++) {
        int t;
        do {
            t = rand() % MAX_TASKS;
        } while (used[t]);
        used[t] = 1;
        a->task_queue[i] = t;
    }
    a->task_idx = 0;
    a->tasks_completed = 0;
}

static void scatter_around_cafeteria(SimState *s) {
    for (int i = 0; i < MAX_AGENTS; i++) {
        Agent *a = &s->agents[i];
        if (!a->alive) continue;
        float angle = (float)i * (6.283185f / (float)MAX_AGENTS);
        a->x = 500.0f + cosf(angle) * 58.0f;
        a->y = 135.0f + sinf(angle) * 42.0f;
        a->vx = 0;
        a->vy = 0;
        a->current_room = 1;
        a->path_len = 0;
        a->path_idx = 0;
        for (int j = 0; j < MAX_AGENTS; j++) {
            a->last_seen_by_tick[j] = -100;
            a->last_room_of[j] = -1;
            a->follow_streak[j] = 0;
        }
    }
}

void sim_init(SimState *s, int impostor_count) {
    map_init();
    sim_reset(s, impostor_count);
}

void sim_reset(SimState *s, int impostor_count) {
    memset(s, 0, sizeof(*s));
    if (impostor_count < 1) impostor_count = 1;
    if (impostor_count > 3) impostor_count = 3;

    s->impostor_count = impostor_count;
    s->paused = false;
    s->speed_ms = 140;
    s->phase = PHASE_ROAMING;
    s->next_event_id = 1;
    s->sabotage_cooldown = 25;
    s->locked_room = -1;

    int pool_idx[POOL_SIZE];
    for (int i = 0; i < POOL_SIZE; i++) pool_idx[i] = i;
    shuffle_ints(pool_idx, POOL_SIZE);

    int indices[MAX_AGENTS];
    for (int i = 0; i < MAX_AGENTS; i++) indices[i] = i;
    shuffle_ints(indices, MAX_AGENTS);

    bool is_imp[MAX_AGENTS] = {false};
    for (int i = 0; i < impostor_count; i++) {
        is_imp[indices[i]] = true;
    }

    // Initialize base agents
    for (int i = 0; i < MAX_AGENTS; i++) {
        Agent *a = &s->agents[i];
        const AgentProfile *prof = &AGENT_POOL[pool_idx[i]];
        a->id = i;
        a->name = prof->name;
        a->color = prof->color;
        a->symbol = prof->symbol;
        a->role = is_imp[i] ? ROLE_IMPOSTOR : ROLE_CREWMATE;
        a->srole = SROLE_NONE;
        a->personality = prof->pers;
        a->sus_mult = prof->sus_mult;
        a->vote_offset = prof->vote_offset;
        a->chaos_prob = prof->chaos_prob;
        a->alive = true;
        a->ejected = false;
        a->activity = ACT_WALKING;
        a->kill_cooldown = rand_range(14 + impostor_count * 6, 22 + impostor_count * 8);
        a->vent_cooldown = 10;
        a->ability_cooldown = rand_range(10, 22);
        a->protected_by = -1;
        a->tracked_target = -1;
        a->disguise_id = -1;
        a->inv_target = -1;
        a->vote_target = -2;
        a->claimed_room = 1;
        for (int j = 0; j < MAX_AGENTS; j++) {
            a->last_seen_by_tick[j] = -100;
            a->last_room_of[j] = -1;
        }
        assign_random_tasks(a);
    }

    // Role rotation: random number of special roles (0..all) for Crewmates and Impostors!
    int crew_ids[MAX_AGENTS], crew_n = 0;
    int imp_ids[MAX_AGENTS], imp_n = 0;
    for (int i = 0; i < MAX_AGENTS; i++) {
        if (s->agents[i].role == ROLE_IMPOSTOR) imp_ids[imp_n++] = i;
        else crew_ids[crew_n++] = i;
    }
    shuffle_ints(crew_ids, crew_n);
    shuffle_ints(imp_ids, imp_n);

    // Optional Jester (Bobo) ~35% chance
    int jester_used = 0;
    if (crew_n >= 4 && (rand() % 100) < 35) {
        int jid = crew_ids[0];
        s->agents[jid].role = ROLE_NEUTRAL;
        s->agents[jid].srole = SROLE_JESTER;
        jester_used = 1;
    }

    // Crewmate special roles pool (6 roles, shuffled)
    int crew_pool[6] = {
        SROLE_DOCTOR, SROLE_INVESTIGATOR, SROLE_WATCHER,
        SROLE_ENGINEER, SROLE_TRACKER, SROLE_MAYOR
    };
    shuffle_ints(crew_pool, 6);
    int avail_crew = crew_n - jester_used;
    int num_crew_roles = rand_range(0, avail_crew < 6 ? avail_crew : 6);
    for (int r = 0; r < num_crew_roles; r++) {
        s->agents[crew_ids[jester_used + r]].srole = (SpecialRole)crew_pool[r];
    }

    // Impostor special roles pool (5 roles, shuffled)
    int imp_pool[5] = {
        SROLE_SHAPESHIFTER, SROLE_JANITOR, SROLE_SABOTEUR,
        SROLE_SILENCER, SROLE_INFILTRATOR
    };
    shuffle_ints(imp_pool, 5);
    int num_imp_roles = rand_range(0, imp_n);
    for (int r = 0; r < num_imp_roles; r++) {
        s->agents[imp_ids[r]].srole = (SpecialRole)imp_pool[r];
    }

    int real_crew = crew_n - jester_used;
    s->global_tasks_total = real_crew * MAX_TASKS_PER_AGENT;

    for (int i = 0; i < MAX_AGENTS; i++) {
        Agent *a = &s->agents[i];
        snprintf(a->last_thought, sizeof(a->last_thought),
                 a->role == ROLE_IMPOSTOR ? "Disfarce ativo" :
                 (a->srole == SROLE_JESTER ? "Quero ser ejetado!" : "Indo para task"));
    }

    scatter_around_cafeteria(s);
    for (int i = 0; i < MAX_AGENTS; i++) {
        Agent *a = &s->agents[i];
        const TaskPoint *tp = &MAP_TASKS[a->task_queue[0]];
        set_agent_path_to(a, tp->x, tp->y);
    }

    push_event(s, EVT_SYSTEM, -1, -1,
               "Partida: %d imp | %d roles Crew | %d roles Imp%s.",
               impostor_count, num_crew_roles, num_imp_roles,
               jester_used ? " + Bobo" : "");
}

void sim_sabotage_lights(SimState *s) {
    if (s->phase != PHASE_ROAMING || s->lights_sabotaged_timer > 0) return;
    s->lights_sabotaged_timer = 32;
    s->sabotage_cooldown = 75;
    push_event(s, EVT_SABOTAGE, -1, -1,
               "LUZES SABOTADAS! Visao dos tripulantes reduzida.");
}

static float agent_vision_radius(const SimState *s, const Agent *a) {
    if (a->activity == ACT_WATCHING_CAMS) return 900.0f;
    if (a->role == ROLE_IMPOSTOR) return 215.0f;
    if (s->lights_sabotaged_timer > 0) return 75.0f;
    return 175.0f;
}

static int count_witnesses(const SimState *s, int killer_id, int victim_id, float x, float y) {
    int count = 0;
    for (int i = 0; i < MAX_AGENTS; i++) {
        if (i == killer_id || i == victim_id) continue;
        const Agent *obs = &s->agents[i];
        if (!obs->alive || obs->role == ROLE_IMPOSTOR) continue;
        float r = agent_vision_radius(s, obs);
        if (obs->activity == ACT_WATCHING_CAMS) {
            if (map_dist(obs->x, obs->y, x, y) <= r) count++;
        } else if (map_dist(obs->x, obs->y, x, y) <= r && map_has_los(obs->x, obs->y, x, y)) {
            count++;
        }
    }
    return count;
}

static void adjust_tasks_on_loss(SimState *s, const Agent *victim) {
    if (victim->role == ROLE_CREWMATE) {
        int rem = MAX_TASKS_PER_AGENT - victim->tasks_completed;
        if (rem > 0) {
            s->global_tasks_total -= rem;
            if (s->global_tasks_total < s->global_tasks_done) {
                s->global_tasks_total = s->global_tasks_done;
            }
        }
    }
}

static void check_win_conditions(SimState *s) {
    if (s->phase == PHASE_JESTER_WIN) return;
    int alive_imp = 0, alive_non_imp = 0;
    for (int i = 0; i < MAX_AGENTS; i++) {
        if (!s->agents[i].alive) continue;
        if (s->agents[i].role == ROLE_IMPOSTOR) alive_imp++;
        else alive_non_imp++;
    }
    if (alive_imp == 0) {
        s->phase = PHASE_CREW_WIN;
        push_event(s, EVT_SYSTEM, -1, -1, "VITORIA DOS TRIPULANTES! Todos os impostores ejetados.");
    } else if (s->global_tasks_total > 0 && s->global_tasks_done >= s->global_tasks_total) {
        s->phase = PHASE_CREW_WIN;
        push_event(s, EVT_SYSTEM, -1, -1, "VITORIA DOS TRIPULANTES! Todas as tasks concluidas.");
    } else if (alive_imp >= alive_non_imp) {
        s->phase = PHASE_IMPOSTOR_WIN;
        push_event(s, EVT_SYSTEM, -1, -1, "VITORIA DOS IMPOSTORES! Maioria alcancada.");
    }
}

static void trigger_meeting(SimState *s, int reporter_id, int body_idx) {
    s->phase = PHASE_MEETING;
    s->meeting_step = 0;
    s->meeting_reporter = reporter_id;
    s->meeting_victim = s->bodies[body_idx].victim_id;
    s->meeting_body_room = s->bodies[body_idx].room_id;
    s->lights_sabotaged_timer = 0;
    s->locked_room = -1;
    s->locked_timer = 0;

    for (int i = 0; i < s->body_count; i++) {
        s->bodies[i].reported = true;
    }

    for (int i = 0; i < MAX_AGENTS; i++) {
        s->agents[i].vote_target = -2;
        s->agents[i].disguise_id = -1;
        s->agents[i].disguise_timer = 0;
        s->agents[i].claimed_room = s->agents[i].current_room >= 0 ? s->agents[i].current_room : 1;
    }

    const char *rname = (s->meeting_body_room >= 0) ? MAP_ROOMS[s->meeting_body_room].name : "Corredor";
    push_event(s, EVT_REPORT, reporter_id, s->meeting_victim,
               "%s reportou o corpo de %s em %s!",
               s->agents[reporter_id].name,
               s->agents[s->meeting_victim].name,
               rname);

    // Investigator forensics on body
    for (int i = 0; i < MAX_AGENTS; i++) {
        Agent *inv = &s->agents[i];
        if (!inv->alive || inv->srole != SROLE_INVESTIGATOR) continue;
        uint32_t age = s->tick - s->bodies[body_idx].tick_killed;
        push_event(s, EVT_ROLE, i, -1,
                   "%s (Investigador): Corpo morreu ha %u ticks!", inv->name, age);
        for (int m = 0; m < MAX_MEMORY; m++) {
            const Sighting *sg = &inv->memory[m];
            if (!sg->valid) continue;
            if (sg->room_id == s->meeting_body_room &&
                abs((int)sg->tick - (int)s->bodies[body_idx].tick_killed) <= 12) {
                inv->suspicion[sg->agent_id] = clampf(
                    inv->suspicion[sg->agent_id] + 0.50f, 0.0f, 1.0f);
            }
        }
    }

    scatter_around_cafeteria(s);
}

static void step_perception_and_suspicion(SimState *s) {
    for (int i = 0; i < MAX_AGENTS; i++) {
        Agent *a = &s->agents[i];
        if (!a->alive) continue;
        a->current_room = map_room_at(a->x, a->y);
        float vr = agent_vision_radius(s, a);
        bool cams = (a->activity == ACT_WATCHING_CAMS);

        for (int j = 0; j < MAX_AGENTS; j++) {
            if (i == j) continue;
            const Agent *b = &s->agents[j];
            if (!b->alive) continue;

            float d = map_dist(a->x, a->y, b->x, b->y);
            bool visible = cams ? (d <= vr) : (d <= vr && map_has_los(a->x, a->y, b->x, b->y));
            if (visible) {
                // If Shapeshifter is disguised, observers see the copied identity!
                int apparent_id = (b->disguise_id >= 0 && a->role != ROLE_IMPOSTOR) ? b->disguise_id : j;
                int b_room = map_room_at(b->x, b->y);

                // Impossible teleport / vent speed check
                if (a->role == ROLE_CREWMATE && a->last_room_of[apparent_id] >= 0 && b_room >= 0 &&
                    a->last_room_of[apparent_id] != b_room &&
                    (int)s->tick - a->last_seen_by_tick[apparent_id] <= 4) {
                    float room_d = map_dist(MAP_ROOMS[a->last_room_of[apparent_id]].center_x,
                                            MAP_ROOMS[a->last_room_of[apparent_id]].center_y,
                                            MAP_ROOMS[b_room].center_x,
                                            MAP_ROOMS[b_room].center_y);
                    if (room_d > 260.0f) {
                        a->suspicion[apparent_id] = clampf(a->suspicion[apparent_id] + 0.85f, 0.0f, 1.0f);
                        snprintf(a->last_thought, sizeof(a->last_thought),
                                 "Viu %s sair do duto!", s->agents[apparent_id].name);
                        push_event(s, EVT_SIGHTING, i, apparent_id,
                                   "%s flagrou %s usando duto!",
                                   a->name, s->agents[apparent_id].name);
                    }
                }

                // Stalking check
                int stalk_lim = (a->personality == PERS_PARANOICO) ? 2 : 3;
                if (a->role == ROLE_CREWMATE && a->activity == ACT_WALKING &&
                    b->activity == ACT_WALKING && b_room >= 0 && b_room == a->current_room) {
                    if (a->last_room_of[apparent_id] != b_room) {
                        a->follow_streak[apparent_id]++;
                        if (a->follow_streak[apparent_id] >= stalk_lim) {
                            float bump = ((a->personality == PERS_PARANOICO) ? 0.28f : 0.18f) * a->sus_mult;
                            a->suspicion[apparent_id] = clampf(a->suspicion[apparent_id] + bump, 0.0f, 1.0f);
                            snprintf(a->last_thought, sizeof(a->last_thought),
                                     "%s esta me seguindo!", s->agents[apparent_id].name);
                            a->follow_streak[apparent_id] = 0;
                        }
                    }
                } else {
                    a->follow_streak[apparent_id] = 0;
                }

                // Analytical / Astuto fake-task check (Infiltrator fools this!)
                if (a->role == ROLE_CREWMATE &&
                    (a->personality == PERS_ANALITICO || a->personality == PERS_ASTUTO) &&
                    b->activity == ACT_FAKE_TASK && b->srole != SROLE_INFILTRATOR &&
                    s->global_tasks_done == s->prev_global_tasks_done &&
                    (rand() % 100) < (a->personality == PERS_ASTUTO ? 10 : 7)) {
                    a->suspicion[apparent_id] = clampf(a->suspicion[apparent_id] + 0.18f * a->sus_mult, 0.0f, 1.0f);
                    snprintf(a->last_thought, sizeof(a->last_thought),
                             "%s parece fingir task...", s->agents[apparent_id].name);
                }

                // Investigator proximity scan
                if (a->srole == SROLE_INVESTIGATOR && d <= 130.0f) {
                    if (a->inv_target == j) {
                        a->inv_ticks++;
                        if (a->inv_ticks >= 5) {
                            bool looks_imp = (b->role == ROLE_IMPOSTOR);
                            if ((rand() % 100) < 15) looks_imp = !looks_imp; // 15% noise
                            if (looks_imp) {
                                a->suspicion[j] = clampf(a->suspicion[j] + 0.60f, 0.0f, 1.0f);
                                snprintf(a->last_thought, sizeof(a->last_thought),
                                         "Pista: %s suspeito!", b->name);
                            } else {
                                a->suspicion[j] = clampf(a->suspicion[j] - 0.25f, 0.0f, 1.0f);
                            }
                            a->inv_ticks = 0;
                            a->inv_target = -1;
                        }
                    } else {
                        a->inv_target = j;
                        a->inv_ticks = 1;
                    }
                }

                record_sighting(a, apparent_id, b_room, s->tick, b->activity);
                if (b_room >= 0) {
                    a->last_seen_by_tick[apparent_id] = (int)s->tick;
                    a->last_room_of[apparent_id] = b_room;
                }
            }
        }

        // Tracker updates real-time room of tracked_target
        if (a->srole == SROLE_TRACKER && a->tracked_target >= 0 && a->track_timer > 0) {
            const Agent *tr = &s->agents[a->tracked_target];
            if (tr->alive) {
                int tr_r = map_room_at(tr->x, tr->y);
                if (tr_r >= 0) {
                    a->last_room_of[a->tracked_target] = tr_r;
                    a->last_seen_by_tick[a->tracked_target] = (int)s->tick;
                }
            }
        }

        // Check for unreported dead bodies
        if (a->role == ROLE_CREWMATE) {
            for (int b = 0; b < s->body_count; b++) {
                DeadBody *db = &s->bodies[b];
                if (db->reported) continue;
                float d = map_dist(a->x, a->y, db->x, db->y);
                bool sees_body = cams ? (d <= vr) : (d <= vr && map_has_los(a->x, a->y, db->x, db->y));
                if (sees_body) {
                    for (int m = 0; m < MAX_MEMORY; m++) {
                        Sighting *sg = &a->memory[m];
                        if (!sg->valid) continue;
                        if (sg->room_id == db->room_id && s->tick - sg->tick <= 24) {
                            a->suspicion[sg->agent_id] = clampf(
                                a->suspicion[sg->agent_id] + 0.50f * a->sus_mult, 0.0f, 1.0f);
                        }
                    }
                    trigger_meeting(s, i, b);
                    return;
                }
            }
        }
    }
}

static void step_special_abilities(SimState *s, Agent *a) {
    if (a->ability_cooldown > 0) a->ability_cooldown--;
    if (a->protect_timer > 0) {
        a->protect_timer--;
        if (a->protect_timer == 0) a->protected_by = -1;
    }
    if (a->track_timer > 0) {
        a->track_timer--;
        if (a->track_timer == 0) a->tracked_target = -1;
    }
    if (a->disguise_timer > 0) {
        a->disguise_timer--;
        if (a->disguise_timer == 0) a->disguise_id = -1;
    }

    // Doctor: shield a trusted/nearby alive agent
    if (a->srole == SROLE_DOCTOR && a->ability_cooldown == 0) {
        int best = -1;
        float best_s = 1.0f;
        for (int j = 0; j < MAX_AGENTS; j++) {
            if (!s->agents[j].alive) continue;
            if (j != a->id && map_dist(a->x, a->y, s->agents[j].x, s->agents[j].y) <= 200.0f) {
                if (a->suspicion[j] < best_s) {
                    best_s = a->suspicion[j];
                    best = j;
                }
            }
        }
        if (best >= 0) {
            s->agents[best].protected_by = a->id;
            s->agents[best].protect_timer = 35;
            a->ability_cooldown = 45;
            snprintf(a->last_thought, sizeof(a->last_thought), "Protegendo %s", s->agents[best].name);
        }
    }

    // Tracker: mark a nearby agent
    if (a->srole == SROLE_TRACKER && a->ability_cooldown == 0 && a->tracked_target < 0) {
        for (int j = 0; j < MAX_AGENTS; j++) {
            if (j == a->id || !s->agents[j].alive) continue;
            if (map_dist(a->x, a->y, s->agents[j].x, s->agents[j].y) <= 150.0f &&
                map_has_los(a->x, a->y, s->agents[j].x, s->agents[j].y)) {
                a->tracked_target = j;
                a->track_timer = 45;
                a->ability_cooldown = 55;
                snprintf(a->last_thought, sizeof(a->last_thought), "Rastreando %s", s->agents[j].name);
                break;
            }
        }
    }

    // Shapeshifter: copy an alive crewmate's identity
    if (a->srole == SROLE_SHAPESHIFTER && a->ability_cooldown == 0 && a->disguise_id < 0) {
        for (int tries = 0; tries < 8; tries++) {
            int c = rand() % MAX_AGENTS;
            if (s->agents[c].alive && s->agents[c].role != ROLE_IMPOSTOR) {
                a->disguise_id = c;
                a->disguise_timer = 26;
                a->ability_cooldown = 55;
                snprintf(a->last_thought, sizeof(a->last_thought), "Disfarce: %s", s->agents[c].name);
                break;
            }
        }
    }

    // Saboteur: lock a room with an isolated crewmate
    if (a->srole == SROLE_SABOTEUR && a->ability_cooldown == 0 && s->locked_room < 0) {
        int r = map_room_at(a->x, a->y);
        if (r >= 0 && (rand() % 100) < 12) {
            s->locked_room = r;
            s->locked_timer = 14;
            a->ability_cooldown = 50;
            push_event(s, EVT_SABOTAGE, -1, -1, "Portas de %s trancadas!", MAP_ROOMS[r].name);
        }
    }

    // Silencer: silence a nearby crewmate before next meeting
    if (a->srole == SROLE_SILENCER && a->ability_cooldown == 0) {
        for (int j = 0; j < MAX_AGENTS; j++) {
            if (!s->agents[j].alive || s->agents[j].role == ROLE_IMPOSTOR || s->agents[j].silenced) continue;
            if (map_dist(a->x, a->y, s->agents[j].x, s->agents[j].y) <= 100.0f &&
                map_has_los(a->x, a->y, s->agents[j].x, s->agents[j].y)) {
                s->agents[j].silenced = true;
                a->ability_cooldown = 45;
                snprintf(a->last_thought, sizeof(a->last_thought), "Silenciou %s", s->agents[j].name);
                break;
            }
        }
    }
}

static void step_agent_roaming(SimState *s, Agent *a) {
    if (!a->alive) return;

    if (a->kill_cooldown > 0) a->kill_cooldown--;
    if (a->vent_cooldown > 0) a->vent_cooldown--;
    step_special_abilities(s, a);

    // Watcher on cameras
    if (a->activity == ACT_WATCHING_CAMS) {
        a->watch_timer--;
        if (a->watch_timer <= 0) {
            a->activity = ACT_WALKING;
            const TaskPoint *cur_task = &MAP_TASKS[a->task_queue[a->task_idx]];
            set_agent_path_to(a, cur_task->x, cur_task->y);
        }
        return;
    }

    if (a->role == ROLE_IMPOSTOR) {
        if (s->sabotage_cooldown == 0 && s->lights_sabotaged_timer == 0 && (rand() % 100) < 4) {
            sim_sabotage_lights(s);
        }

        if (a->kill_cooldown == 0) {
            int best_target = -1;
            float best_dist = 185.0f;
            for (int j = 0; j < MAX_AGENTS; j++) {
                const Agent *t = &s->agents[j];
                if (!t->alive || t->role == ROLE_IMPOSTOR) continue;
                float d = map_dist(a->x, a->y, t->x, t->y);
                if (d < best_dist && map_has_los(a->x, a->y, t->x, t->y)) {
                    if (count_witnesses(s, a->id, j, t->x, t->y) == 0) {
                        best_dist = d;
                        best_target = j;
                    }
                }
            }

            if (best_target >= 0) {
                Agent *victim = &s->agents[best_target];
                a->activity = ACT_HUNTING;
                snprintf(a->last_thought, sizeof(a->last_thought), "Caçando %s", victim->name);
                if (map_has_los(a->x, a->y, victim->x, victim->y)) {
                    move_agent_toward(s, a, victim->x, victim->y, 12.2f);
                } else {
                    if (a->path_len == 0 || map_dist(a->goal_x, a->goal_y, victim->x, victim->y) > 25.0f) {
                        set_agent_path_to(a, victim->x, victim->y);
                    }
                    follow_path_step(s, a, 12.2f);
                }

                if (map_dist(a->x, a->y, victim->x, victim->y) <= 28.0f &&
                    count_witnesses(s, a->id, best_target, victim->x, victim->y) == 0) {
                    // Check Doctor shield!
                    if (victim->protected_by >= 0 && victim->protect_timer > 0) {
                        int doc = victim->protected_by;
                        s->agents[doc].suspicion[victim->id] = 0.0f;
                        victim->protected_by = -1;
                        victim->protect_timer = 0;
                        a->kill_cooldown = 20;
                        a->activity = ACT_FLEEING;
                        int flee_room = rand() % MAX_ROOMS;
                        set_agent_path_to(a, MAP_ROOMS[flee_room].center_x, MAP_ROOMS[flee_room].center_y);
                        push_event(s, EVT_ROLE, doc, victim->id,
                                   "Escudo medico salvou %s de um ataque!", victim->name);
                        return;
                    }

                    victim->alive = false;
                    victim->activity = ACT_DEAD;
                    adjust_tasks_on_loss(s, victim);
                    int room = map_room_at(victim->x, victim->y);

                    // Janitor cleans the body!
                    bool cleaned = (a->srole == SROLE_JANITOR && a->ability_cooldown == 0);
                    if (cleaned) {
                        a->ability_cooldown = 50;
                        push_event(s, EVT_ROLE, a->id, victim->id,
                                   "%s (Limpador) sumiu com o corpo de %s!", a->name, victim->name);
                    } else if (s->body_count < MAX_BODIES) {
                        s->bodies[s->body_count++] = (DeadBody){
                            .victim_id = victim->id,
                            .room_id = room,
                            .x = victim->x,
                            .y = victim->y,
                            .tick_killed = s->tick,
                            .reported = false
                        };
                        push_event(s, EVT_KILL, a->id, victim->id,
                                   "%s eliminou %s em %s.",
                                   a->name, victim->name,
                                   room >= 0 ? MAP_ROOMS[room].name : "Corredor");
                    }

                    a->kill_cooldown = 20 + s->impostor_count * 12;
                    a->activity = ACT_FLEEING;
                    snprintf(a->last_thought, sizeof(a->last_thought), "Eliminou %s!", victim->name);

                    bool vented = false;
                    if (a->vent_cooldown == 0) {
                        for (int v = 0; v < MAX_VENTS; v++) {
                            if (map_dist(a->x, a->y, MAP_VENTS[v].x, MAP_VENTS[v].y) < 120.0f) {
                                int dest = MAP_VENTS[v].linked_vent_id;
                                a->x = MAP_VENTS[dest].x;
                                a->y = MAP_VENTS[dest].y;
                                a->vent_cooldown = 20;
                                vented = true;
                                snprintf(a->last_thought, sizeof(a->last_thought), "Fugiu pelo duto!");
                                break;
                            }
                        }
                    }
                    if (!vented) {
                        int flee_room = (room + 2) % MAX_ROOMS;
                        set_agent_path_to(a, MAP_ROOMS[flee_room].center_x, MAP_ROOMS[flee_room].center_y);
                    }
                    check_win_conditions(s);
                }
                return;
            }
        }
    } else if (a->role == ROLE_CREWMATE) {
        int fix_chance = (a->personality == PERS_MEDROSO) ? 55 :
                         (a->personality == PERS_BURRO) ? 8 : 25;
        if (s->lights_sabotaged_timer > 0 && a->activity == ACT_WALKING &&
            (rand() % 100) < fix_chance) {
            a->activity = ACT_FIXING_LIGHTS;
            set_agent_path_to(a, MAP_TASKS[7].x, MAP_TASKS[7].y);
            snprintf(a->last_thought, sizeof(a->last_thought), "Indo religar luzes");
        }
    }

    // Jester: purposely stalk another alive agent to draw suspicion!
    if (a->srole == SROLE_JESTER) {
        int target = (a->id + 1) % MAX_AGENTS;
        for (int k = 0; k < MAX_AGENTS; k++) {
            int c = (a->id + 1 + k) % MAX_AGENTS;
            if (s->agents[c].alive) { target = c; break; }
        }
        if (a->path_len == 0 || (s->tick % 12) == 0) {
            set_agent_path_to(a, s->agents[target].x, s->agents[target].y);
        }
        follow_path_step(s, a, 11.2f);
        snprintf(a->last_thought, sizeof(a->last_thought), "Seguindo %s hehe", s->agents[target].name);
        return;
    }

    if (a->activity == ACT_FIXING_LIGHTS) {
        if (s->lights_sabotaged_timer == 0) {
            a->activity = ACT_WALKING;
            const TaskPoint *cur_task = &MAP_TASKS[a->task_queue[a->task_idx]];
            set_agent_path_to(a, cur_task->x, cur_task->y);
        } else {
            follow_path_step(s, a, 11.5f);
            if (map_dist(a->x, a->y, MAP_TASKS[7].x, MAP_TASKS[7].y) < 14.0f) {
                s->lights_sabotaged_timer = 0;
                a->activity = ACT_WALKING;
                const TaskPoint *cur_task = &MAP_TASKS[a->task_queue[a->task_idx]];
                set_agent_path_to(a, cur_task->x, cur_task->y);
                push_event(s, EVT_SYSTEM, a->id, -1, "%s religou as luzes na Eletrica!", a->name);
            }
        }
        return;
    }

    if (a->activity == ACT_DOING_TASK || a->activity == ACT_FAKE_TASK) {
        a->task_timer--;
        if (a->task_timer <= 0) {
            if (a->role == ROLE_CREWMATE && a->tasks_completed < MAX_TASKS_PER_AGENT) {
                a->tasks_completed++;
                s->global_tasks_done++;
            }
            // Watcher stops on cameras if in Seguranca (room 3)
            if (a->srole == SROLE_WATCHER && a->current_room == 3 && (rand() % 100) < 60) {
                a->activity = ACT_WATCHING_CAMS;
                a->watch_timer = 12;
                snprintf(a->last_thought, sizeof(a->last_thought), "Vigiando cameras!");
                return;
            }
            a->task_idx = (a->task_idx + 1) % MAX_TASKS_PER_AGENT;
            const TaskPoint *nxt = &MAP_TASKS[a->task_queue[a->task_idx]];
            set_agent_path_to(a, nxt->x, nxt->y);
            a->activity = ACT_WALKING;
            snprintf(a->last_thought, sizeof(a->last_thought), "Indo para %s", MAP_ROOMS[nxt->room_id].name);
            check_win_conditions(s);
        }
        return;
    }

    if (a->activity == ACT_FLEEING) {
        follow_path_step(s, a, 12.5f);
        if (a->path_idx >= a->path_len) {
            a->activity = ACT_WALKING;
        }
        return;
    }

    // Engineer shortcut through vent
    if (a->srole == SROLE_ENGINEER && a->vent_cooldown == 0) {
        const TaskPoint *cur_task = &MAP_TASKS[a->task_queue[a->task_idx]];
        if (a->current_room >= 0 && a->current_room != cur_task->room_id) {
            for (int v = 0; v < MAX_VENTS; v++) {
                if (MAP_VENTS[v].room_id == a->current_room &&
                    map_dist(a->x, a->y, MAP_VENTS[v].x, MAP_VENTS[v].y) < 90.0f) {
                    int dest = MAP_VENTS[v].linked_vent_id;
                    a->x = MAP_VENTS[dest].x;
                    a->y = MAP_VENTS[dest].y;
                    a->vent_cooldown = 35;
                    set_agent_path_to(a, cur_task->x, cur_task->y);
                    snprintf(a->last_thought, sizeof(a->last_thought), "Atalho pelo duto!");
                    return;
                }
            }
        }
    }

    const TaskPoint *cur_task = &MAP_TASKS[a->task_queue[a->task_idx]];
    if (a->path_len == 0 || map_dist(a->goal_x, a->goal_y, cur_task->x, cur_task->y) > 5.0f) {
        set_agent_path_to(a, cur_task->x, cur_task->y);
    }
    follow_path_step(s, a, 11.0f);
    if (map_dist(a->x, a->y, cur_task->x, cur_task->y) < 14.0f) {
        a->activity = (a->role == ROLE_IMPOSTOR) ? ACT_FAKE_TASK : ACT_DOING_TASK;
        a->task_timer = rand_range(7, 12);
        snprintf(a->last_thought, sizeof(a->last_thought),
                 a->role == ROLE_IMPOSTOR ? "Fingindo task..." : "Fazendo %s", cur_task->name);
    }
}

static void step_meeting(SimState *s) {
    s->meeting_step++;

    if (s->meeting_step == 1) {
        for (int i = 0; i < MAX_AGENTS; i++) {
            Agent *a = &s->agents[i];
            if (!a->alive) continue;
            if (a->silenced) {
                push_event(s, EVT_ROLE, i, -1, "%s foi SILENCIADO e nao pode falar!", a->name);
                continue;
            }
            if (a->role == ROLE_IMPOSTOR) {
                int fake_room = (s->meeting_body_room + 1 + (rand() % 4)) % MAX_ROOMS;
                a->claimed_room = fake_room;
            } else if (a->srole == SROLE_JESTER) {
                // Jester lies on purpose to look suspicious!
                a->claimed_room = s->meeting_body_room >= 0 ? s->meeting_body_room : (rand() % MAX_ROOMS);
            } else if ((float)(rand() % 100) / 100.0f < a->chaos_prob * 0.6f) {
                a->claimed_room = rand() % MAX_ROOMS;
            }
            push_event(s, EVT_ALIBI, i, -1, "%s: Estava em %s.",
                       a->name,
                       a->claimed_room >= 0 ? MAP_ROOMS[a->claimed_room].name : "Corredor");

            if (a->role == ROLE_CREWMATE && s->meeting_body_room >= 0) {
                for (int m = 0; m < MAX_MEMORY; m++) {
                    const Sighting *sg = &a->memory[m];
                    if (!sg->valid) continue;
                    if (sg->room_id == s->meeting_body_room && s->tick - sg->tick <= 24 &&
                        s->agents[sg->agent_id].alive) {
                        int tgt = sg->agent_id;
                        push_event(s, EVT_SIGHTING, i, tgt,
                                   "%s: Vi %s perto de %s!",
                                   a->name, s->agents[tgt].name,
                                   MAP_ROOMS[s->meeting_body_room].name);
                        for (int k = 0; k < MAX_AGENTS; k++) {
                            if (k != tgt && s->agents[k].alive && s->agents[k].role == ROLE_CREWMATE) {
                                s->agents[k].suspicion[tgt] = clampf(
                                    s->agents[k].suspicion[tgt] + 0.16f * s->agents[k].sus_mult,
                                    0.0f, 1.0f);
                            }
                        }
                        break;
                    }
                }
            }
        }
    } else if (s->meeting_step == 2) {
        for (int i = 0; i < MAX_AGENTS; i++) {
            Agent *a = &s->agents[i];
            if (!a->alive || a->silenced) continue;

            // Engineer defends self if suspected
            if (a->srole == SROLE_ENGINEER && !a->role_revealed) {
                for (int k = 0; k < MAX_AGENTS; k++) {
                    if (s->agents[k].alive && s->agents[k].suspicion[i] >= 0.6f) {
                        a->role_revealed = true;
                        push_event(s, EVT_ROLE, i, -1,
                                   "%s revela: Sou ENGENHEIRO, uso dutos!", a->name);
                        for (int m = 0; m < MAX_AGENTS; m++) {
                            if (s->agents[m].alive && s->agents[m].role == ROLE_CREWMATE) {
                                s->agents[m].suspicion[i] *= 0.4f;
                            }
                        }
                        break;
                    }
                }
            }

            // Vouch for someone seen doing task
            if (a->role == ROLE_CREWMATE) {
                for (int m = 0; m < MAX_MEMORY; m++) {
                    const Sighting *sg = &a->memory[m];
                    if (!sg->valid) continue;
                    if (sg->activity == ACT_DOING_TASK && s->tick - sg->tick <= 18 &&
                        s->agents[sg->agent_id].alive && a->suspicion[sg->agent_id] < 0.3f) {
                        int v = sg->agent_id;
                        push_event(s, EVT_VOUCH, i, v,
                                   "%s garante %s (fazendo task).", a->name, s->agents[v].name);
                        for (int k = 0; k < MAX_AGENTS; k++) {
                            if (s->agents[k].alive && s->agents[k].role == ROLE_CREWMATE) {
                                s->agents[k].suspicion[v] = clampf(
                                    s->agents[k].suspicion[v] - 0.10f, 0.0f, 1.0f);
                            }
                        }
                        break;
                    }
                }
            }

            // Check contradictions (including Tracker!)
            for (int j = 0; j < MAX_AGENTS; j++) {
                if (i == j || !s->agents[j].alive) continue;
                if (a->role == ROLE_IMPOSTOR && s->agents[j].role == ROLE_IMPOSTOR) continue;

                if (a->last_room_of[j] >= 0 && (int)s->tick - a->last_seen_by_tick[j] <= 22 &&
                    a->last_room_of[j] != s->agents[j].claimed_room && a->role == ROLE_CREWMATE) {
                    float boost = (a->srole == SROLE_TRACKER && a->tracked_target == j) ? 0.55f : 0.30f * a->sus_mult;
                    a->suspicion[j] = clampf(a->suspicion[j] + boost, 0.0f, 1.0f);
                    push_event(s, EVT_ACCUSE, i, j,
                               "%s: Contradicao! Vi %s em %s, nao em %s!",
                               a->name, s->agents[j].name,
                               MAP_ROOMS[a->last_room_of[j]].name,
                               MAP_ROOMS[s->agents[j].claimed_room].name);
                    break;
                }
            }

            if (a->personality == PERS_BRINCALHAO && ((float)(rand() % 100) / 100.0f) < a->chaos_prob) {
                int rnd = rand() % MAX_AGENTS;
                if (rnd != i && s->agents[rnd].alive) {
                    push_event(s, EVT_ACCUSE, i, rnd,
                               "%s (Brincalhao): Foi %s, confia!", a->name, s->agents[rnd].name);
                }
                continue;
            }

            int top = -1;
            float top_s = 0.0f;
            for (int j = 0; j < MAX_AGENTS; j++) {
                if (i == j || !s->agents[j].alive) continue;
                if (a->role == ROLE_IMPOSTOR && s->agents[j].role == ROLE_IMPOSTOR) continue;
                if (a->suspicion[j] > top_s) {
                    top_s = a->suspicion[j];
                    top = j;
                }
            }
            float acc_thresh = clampf(0.40f + a->vote_offset, 0.22f, 0.72f);
            if (top >= 0 && top_s >= acc_thresh) {
                push_event(s, EVT_ACCUSE, i, top,
                           "%s acusa %s (suspeita %.0f%%).",
                           a->name, s->agents[top].name, top_s * 100.0f);
                float inf = (a->personality == PERS_SOCIAL) ? 0.14f :
                            (a->personality == PERS_ASTUTO) ? 0.12f : 0.07f;
                for (int k = 0; k < MAX_AGENTS; k++) {
                    if (k != i && k != top && s->agents[k].alive && s->agents[k].role == ROLE_CREWMATE) {
                        s->agents[k].suspicion[top] = clampf(
                            s->agents[k].suspicion[top] + inf * s->agents[k].sus_mult, 0.0f, 1.0f);
                    }
                }
            }
        }
    } else if (s->meeting_step == 3) {
        for (int i = 0; i < MAX_AGENTS; i++) {
            Agent *a = &s->agents[i];
            if (!a->alive) continue;

            float thresh = clampf(0.42f + a->vote_offset, 0.25f, 0.75f);
            int best = -1;
            float best_s = thresh;
            for (int j = 0; j < MAX_AGENTS; j++) {
                if (i == j || !s->agents[j].alive) continue;
                if (a->role == ROLE_IMPOSTOR && s->agents[j].role == ROLE_IMPOSTOR) continue;
                if (a->suspicion[j] > best_s) {
                    best_s = a->suspicion[j];
                    best = j;
                }
            }

            if (best == -1 && a->role == ROLE_CREWMATE &&
                ((float)(rand() % 100) / 100.0f) < a->chaos_prob * 0.7f) {
                for (int tries = 0; tries < 6; tries++) {
                    int c = rand() % MAX_AGENTS;
                    if (c != i && s->agents[c].alive) { best = c; break; }
                }
            }

            if (a->role == ROLE_IMPOSTOR && best == -1) {
                for (int tries = 0; tries < 6; tries++) {
                    int c = rand() % MAX_AGENTS;
                    if (s->agents[c].alive && s->agents[c].role != ROLE_IMPOSTOR) { best = c; break; }
                }
            }

            a->vote_target = best;
            if (best >= 0) {
                push_event(s, EVT_VOTE, i, best, "%s%s votou em %s.",
                           a->name, a->srole == SROLE_MAYOR ? " (Prefeito x2)" : "",
                           s->agents[best].name);
            } else {
                push_event(s, EVT_VOTE, i, -1, "%s votou SKIP.", a->name);
            }
        }
    } else if (s->meeting_step >= 4) {
        int votes[MAX_AGENTS] = {0};
        int skips = 0;
        for (int i = 0; i < MAX_AGENTS; i++) {
            if (!s->agents[i].alive) continue;
            int w = (s->agents[i].srole == SROLE_MAYOR) ? 2 : 1;
            int vt = s->agents[i].vote_target;
            if (vt >= 0 && vt < MAX_AGENTS) votes[vt] += w;
            else skips += w;
        }

        int max_v = skips;
        int ejected = -1;
        bool tie = false;
        for (int i = 0; i < MAX_AGENTS; i++) {
            if (votes[i] > max_v) {
                max_v = votes[i];
                ejected = i;
                tie = false;
            } else if (votes[i] == max_v && votes[i] > 0) {
                tie = true;
            }
        }

        if (ejected >= 0 && !tie) {
            s->agents[ejected].alive = false;
            s->agents[ejected].ejected = true;
            s->agents[ejected].activity = ACT_DEAD;
            adjust_tasks_on_loss(s, &s->agents[ejected]);
            const char *rlabel = s->agents[ejected].role == ROLE_IMPOSTOR ? "IMPOSTOR" :
                                 (s->agents[ejected].srole == SROLE_JESTER ? "BOBO" : "Inocente");
            push_event(s, EVT_EJECT, -1, ejected,
                       "%s foi EJETADO (%d votos). Era %s!",
                       s->agents[ejected].name, max_v, rlabel);

            if (s->agents[ejected].srole == SROLE_JESTER) {
                s->phase = PHASE_JESTER_WIN;
                push_event(s, EVT_SYSTEM, -1, ejected,
                           "VITORIA DO BOBO (%s)! Conseguiu ser ejetado!", s->agents[ejected].name);
                return;
            }
        } else {
            push_event(s, EVT_EJECT, -1, -1, "Ninguem ejetado (empate ou skip).");
        }

        for (int i = 0; i < MAX_AGENTS; i++) {
            Agent *a = &s->agents[i];
            a->silenced = false;
            if (!a->alive) continue;
            for (int j = 0; j < MAX_AGENTS; j++) {
                a->suspicion[j] *= 0.65f;
            }
            const TaskPoint *tp = &MAP_TASKS[a->task_queue[a->task_idx]];
            set_agent_path_to(a, tp->x, tp->y);
            a->activity = ACT_WALKING;
            if (a->role == ROLE_IMPOSTOR) a->kill_cooldown = 14 + s->impostor_count * 7;
        }

        s->phase = PHASE_ROAMING;
        check_win_conditions(s);
    }
}

void sim_step(SimState *s) {
    if (s->phase == PHASE_CREW_WIN || s->phase == PHASE_IMPOSTOR_WIN || s->phase == PHASE_JESTER_WIN) return;
    s->tick++;

    if (s->phase == PHASE_MEETING) {
        step_meeting(s);
        return;
    }

    if (s->lights_sabotaged_timer > 0) s->lights_sabotaged_timer--;
    if (s->sabotage_cooldown > 0) s->sabotage_cooldown--;
    if (s->locked_timer > 0) {
        s->locked_timer--;
        if (s->locked_timer == 0) s->locked_room = -1;
    }

    s->prev_global_tasks_done = s->global_tasks_done;

    for (int i = 0; i < MAX_AGENTS; i++) {
        step_agent_roaming(s, &s->agents[i]);
        if (s->phase != PHASE_ROAMING) return;
    }

    step_perception_and_suspicion(s);
}

int sim_serialize_json(const SimState *s, char *buf, int buf_size) {
    int pos = 0;
    pos += snprintf(buf + pos, buf_size - pos,
        "{\"tick\":%u,\"paused\":%s,\"speed_ms\":%d,\"impostor_count\":%d,"
        "\"phase\":%d,\"meeting_step\":%d,\"lights_out\":%s,\"locked_room\":%d,"
        "\"tasks_done\":%d,\"tasks_total\":%d,\"agents\":[",
        s->tick, s->paused ? "true" : "false", s->speed_ms, s->impostor_count,
        (int)s->phase, s->meeting_step, s->lights_sabotaged_timer > 0 ? "true" : "false",
        s->locked_room, s->global_tasks_done, s->global_tasks_total);

    for (int i = 0; i < MAX_AGENTS; i++) {
        const Agent *a = &s->agents[i];
        int top_sus = -1;
        float top_val = 0.0f;
        for (int j = 0; j < MAX_AGENTS; j++) {
            if (j != i && a->suspicion[j] > top_val) {
                top_val = a->suspicion[j];
                top_sus = j;
            }
        }
        pos += snprintf(buf + pos, buf_size - pos,
            "%s{\"id\":%d,\"name\":\"%s\",\"color\":\"%s\",\"sym\":\"%s\",\"role\":%d,"
            "\"srole\":%d,\"srole_name\":\"%s\",\"disguise\":%d,\"silenced\":%s,"
            "\"pers\":%d,\"alive\":%s,\"ejected\":%s,\"x\":%.1f,\"y\":%.1f,"
            "\"room\":%d,\"act\":%d,\"tasks\":%d,\"top_sus\":%d,\"top_sus_val\":%.2f,"
            "\"thought\":\"%s\"}",
            i > 0 ? "," : "", a->id, a->name, a->color, a->symbol, (int)a->role,
            (int)a->srole, SROLE_NAMES[a->srole], a->disguise_id, a->silenced ? "true" : "false",
            (int)a->personality, a->alive ? "true" : "false", a->ejected ? "true" : "false",
            a->x, a->y, a->current_room, (int)a->activity, a->tasks_completed,
            top_sus, top_val, a->last_thought);
    }

    pos += snprintf(buf + pos, buf_size - pos, "],\"bodies\":[");
    for (int i = 0; i < s->body_count; i++) {
        const DeadBody *b = &s->bodies[i];
        pos += snprintf(buf + pos, buf_size - pos,
            "%s{\"victim\":%d,\"x\":%.1f,\"y\":%.1f,\"reported\":%s}",
            i > 0 ? "," : "", b->victim_id, b->x, b->y, b->reported ? "true" : "false");
    }

    pos += snprintf(buf + pos, buf_size - pos, "],\"events\":[");
    int start = s->event_count > 24 ? s->event_count - 24 : 0;
    for (int i = start; i < s->event_count; i++) {
        const AclEvent *ev = &s->events[i];
        pos += snprintf(buf + pos, buf_size - pos,
            "%s{\"id\":%u,\"tick\":%u,\"kind\":%d,\"sender\":%d,\"target\":%d,\"text\":\"%s\"}",
            i > start ? "," : "", ev->id, ev->tick, (int)ev->kind,
            ev->sender_id, ev->target_id, ev->text);
    }
    pos += snprintf(buf + pos, buf_size - pos, "]}");
    return pos;
}
