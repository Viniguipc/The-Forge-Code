#include "sim.h"
#include "map.h"
#include <math.h>
#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static const char *AGENT_NAMES[MAX_AGENTS] = {
    "Alfa", "Bravo", "Charlie", "Delta", "Echo",
    "Foxtrot", "Golf", "Hotel", "India", "Juliet"
};

static const char *AGENT_COLORS[MAX_AGENTS] = {
    "#ef4444", "#3b82f6", "#10b981", "#f59e0b", "#a855f7",
    "#ec4899", "#06b6d4", "#84cc16", "#f97316", "#e2e8f0"
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
    int start_wp = map_nearest_waypoint(a->x, a->y);
    int target_room = map_room_at(tx, ty);
    int goal_wp = (target_room >= 0) ? map_room_waypoint(target_room) : map_nearest_waypoint(tx, ty);
    a->path_len = map_find_path(start_wp, goal_wp, a->path);
    a->path_idx = 0;
    a->target_waypoint = goal_wp;
}

static void move_agent_toward(Agent *a, float tx, float ty, float speed) {
    float dx = tx - a->x;
    float dy = ty - a->y;
    float d = sqrtf(dx * dx + dy * dy);
    if (d <= speed) {
        a->x = tx;
        a->y = ty;
        a->vx = 0;
        a->vy = 0;
    } else {
        a->vx = (dx / d) * speed;
        a->vy = (dy / d) * speed;
        a->x += a->vx;
        a->y += a->vy;
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

    // Pick random impostors
    int indices[MAX_AGENTS];
    for (int i = 0; i < MAX_AGENTS; i++) indices[i] = i;
    for (int i = MAX_AGENTS - 1; i > 0; i--) {
        int j = rand() % (i + 1);
        int tmp = indices[i];
        indices[i] = indices[j];
        indices[j] = tmp;
    }
    bool is_imp[MAX_AGENTS] = {false};
    for (int i = 0; i < impostor_count; i++) {
        is_imp[indices[i]] = true;
    }

    int crew_count = MAX_AGENTS - impostor_count;
    s->global_tasks_total = crew_count * MAX_TASKS_PER_AGENT;

    for (int i = 0; i < MAX_AGENTS; i++) {
        Agent *a = &s->agents[i];
        a->id = i;
        a->name = AGENT_NAMES[i];
        a->color = AGENT_COLORS[i];
        a->role = is_imp[i] ? ROLE_IMPOSTOR : ROLE_CREWMATE;
        a->personality = (Personality)(i % 4);
        a->alive = true;
        a->ejected = false;
        a->activity = ACT_WALKING;
        a->kill_cooldown = rand_range(16, 26);
        a->vent_cooldown = 10;
        a->vote_target = -2;
        a->claimed_room = 1;
        for (int j = 0; j < MAX_AGENTS; j++) {
            a->last_seen_by_tick[j] = -100;
            a->last_room_of[j] = -1;
        }
        assign_random_tasks(a);
        snprintf(a->last_thought, sizeof(a->last_thought),
                 a->role == ROLE_IMPOSTOR ? "Disfarce ativo" : "Indo para task");
    }

    scatter_around_cafeteria(s);
    for (int i = 0; i < MAX_AGENTS; i++) {
        Agent *a = &s->agents[i];
        const TaskPoint *tp = &MAP_TASKS[a->task_queue[0]];
        set_agent_path_to(a, tp->x, tp->y);
    }

    push_event(s, EVT_SYSTEM, -1, -1,
               "Partida iniciada: 10 agentes (%d impostor%s).",
               impostor_count, impostor_count > 1 ? "es" : "");
}

void sim_sabotage_lights(SimState *s) {
    if (s->phase != PHASE_ROAMING || s->lights_sabotaged_timer > 0) return;
    s->lights_sabotaged_timer = 32;
    s->sabotage_cooldown = 60;
    push_event(s, EVT_SABOTAGE, -1, -1,
               "LUZES SABOTADAS! Visao dos tripulantes reduzida.");
}

static float agent_vision_radius(const SimState *s, const Agent *a) {
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
        if (map_dist(obs->x, obs->y, x, y) <= r && map_has_los(obs->x, obs->y, x, y)) {
            count++;
        }
    }
    return count;
}

static void check_win_conditions(SimState *s) {
    int alive_imp = 0, alive_crew = 0;
    for (int i = 0; i < MAX_AGENTS; i++) {
        if (!s->agents[i].alive) continue;
        if (s->agents[i].role == ROLE_IMPOSTOR) alive_imp++;
        else alive_crew++;
    }
    if (alive_imp == 0) {
        s->phase = PHASE_CREW_WIN;
        push_event(s, EVT_SYSTEM, -1, -1, "VITORIA DOS TRIPULANTES! Todos os impostores ejetados.");
    } else if (s->global_tasks_done >= s->global_tasks_total) {
        s->phase = PHASE_CREW_WIN;
        push_event(s, EVT_SYSTEM, -1, -1, "VITORIA DOS TRIPULANTES! Todas as tasks concluidas.");
    } else if (alive_imp >= alive_crew) {
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

    for (int i = 0; i < s->body_count; i++) {
        s->bodies[i].reported = true;
    }

    for (int i = 0; i < MAX_AGENTS; i++) {
        s->agents[i].vote_target = -2;
        s->agents[i].claimed_room = s->agents[i].current_room >= 0 ? s->agents[i].current_room : 1;
    }

    const char *rname = (s->meeting_body_room >= 0) ? MAP_ROOMS[s->meeting_body_room].name : "Corredor";
    push_event(s, EVT_REPORT, reporter_id, s->meeting_victim,
               "%s reportou o corpo de %s em %s!",
               s->agents[reporter_id].name,
               s->agents[s->meeting_victim].name,
               rname);
    scatter_around_cafeteria(s);
}

static void step_perception_and_suspicion(SimState *s) {
    for (int i = 0; i < MAX_AGENTS; i++) {
        Agent *a = &s->agents[i];
        if (!a->alive) continue;
        a->current_room = map_room_at(a->x, a->y);
        float vr = agent_vision_radius(s, a);

        // Observe other agents
        for (int j = 0; j < MAX_AGENTS; j++) {
            if (i == j) continue;
            const Agent *b = &s->agents[j];
            if (!b->alive) continue;

            float d = map_dist(a->x, a->y, b->x, b->y);
            if (d <= vr && map_has_los(a->x, a->y, b->x, b->y)) {
                int b_room = map_room_at(b->x, b->y);

                // Impossible teleport / vent speed check
                if (a->role == ROLE_CREWMATE && a->last_room_of[j] >= 0 && b_room >= 0 &&
                    a->last_room_of[j] != b_room &&
                    (int)s->tick - a->last_seen_by_tick[j] <= 4) {
                    float room_d = map_dist(MAP_ROOMS[a->last_room_of[j]].center_x,
                                            MAP_ROOMS[a->last_room_of[j]].center_y,
                                            MAP_ROOMS[b_room].center_x,
                                            MAP_ROOMS[b_room].center_y);
                    if (room_d > 260.0f) {
                        a->suspicion[j] = clampf(a->suspicion[j] + 0.85f, 0.0f, 1.0f);
                        snprintf(a->last_thought, sizeof(a->last_thought),
                                 "Viu %s teleportar/ventar!", b->name);
                        push_event(s, EVT_SIGHTING, i, j,
                                   "%s flagrou %s em deslocamento impossivel!",
                                   a->name, b->name);
                    }
                }

                // Stalking check
                if (a->role == ROLE_CREWMATE && a->activity == ACT_WALKING &&
                    b->activity == ACT_WALKING && b_room >= 0 && b_room == a->current_room) {
                    if (a->last_room_of[j] != b_room) {
                        a->follow_streak[j]++;
                        if (a->follow_streak[j] >= 3) {
                            a->suspicion[j] = clampf(a->suspicion[j] + 0.22f, 0.0f, 1.0f);
                            snprintf(a->last_thought, sizeof(a->last_thought),
                                     "%s esta me seguindo!", b->name);
                            a->follow_streak[j] = 0;
                        }
                    }
                } else {
                    a->follow_streak[j] = 0;
                }

                // Analytical fake-task check
                if (a->role == ROLE_CREWMATE && a->personality == PERS_ANALYTICAL &&
                    b->activity == ACT_FAKE_TASK &&
                    s->global_tasks_done == s->prev_global_tasks_done &&
                    (rand() % 100) < 6) {
                    a->suspicion[j] = clampf(a->suspicion[j] + 0.16f, 0.0f, 1.0f);
                    snprintf(a->last_thought, sizeof(a->last_thought),
                             "%s parece fingir task...", b->name);
                }

                record_sighting(a, j, b_room, s->tick, b->activity);
                a->last_seen_by_tick[j] = (int)s->tick;
                if (b_room >= 0) a->last_room_of[j] = b_room;
            }
        }

        // Check for unreported dead bodies
        if (a->role == ROLE_CREWMATE) {
            for (int b = 0; b < s->body_count; b++) {
                DeadBody *db = &s->bodies[b];
                if (db->reported) continue;
                float d = map_dist(a->x, a->y, db->x, db->y);
                if (d <= vr && map_has_los(a->x, a->y, db->x, db->y)) {
                    // Check memory for anyone seen near body room recently
                    for (int m = 0; m < MAX_MEMORY; m++) {
                        Sighting *sg = &a->memory[m];
                        if (!sg->valid) continue;
                        if (sg->room_id == db->room_id && s->tick - sg->tick <= 24) {
                            a->suspicion[sg->agent_id] = clampf(
                                a->suspicion[sg->agent_id] + 0.55f, 0.0f, 1.0f);
                        }
                    }
                    trigger_meeting(s, i, b);
                    return;
                }
            }
        }
    }
}

static void step_agent_roaming(SimState *s, Agent *a) {
    if (!a->alive) return;

    if (a->kill_cooldown > 0) a->kill_cooldown--;
    if (a->vent_cooldown > 0) a->vent_cooldown--;

    if (a->role == ROLE_IMPOSTOR) {
        // Consider sabotaging lights
        if (s->sabotage_cooldown == 0 && s->lights_sabotaged_timer == 0 && (rand() % 100) < 4) {
            sim_sabotage_lights(s);
        }

        // Look for an isolated crewmate to kill
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
                move_agent_toward(a, victim->x, victim->y, 13.5f);

                if (map_dist(a->x, a->y, victim->x, victim->y) <= 28.0f &&
                    count_witnesses(s, a->id, best_target, victim->x, victim->y) == 0) {
                    victim->alive = false;
                    victim->activity = ACT_DEAD;
                    int room = map_room_at(victim->x, victim->y);
                    if (s->body_count < MAX_BODIES) {
                        s->bodies[s->body_count++] = (DeadBody){
                            .victim_id = victim->id,
                            .room_id = room,
                            .x = victim->x,
                            .y = victim->y,
                            .tick_killed = s->tick,
                            .reported = false
                        };
                    }
                    a->kill_cooldown = 30;
                    a->activity = ACT_FLEEING;
                    snprintf(a->last_thought, sizeof(a->last_thought), "Eliminou %s!", victim->name);
                    push_event(s, EVT_KILL, a->id, victim->id,
                               "%s eliminou %s em %s (sem testemunhas).",
                               a->name, victim->name,
                               room >= 0 ? MAP_ROOMS[room].name : "Corredor");

                    // Try vent escape if near one and no witnesses
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
    } else {
        // Crewmate: if lights are out and nearby, occasionally go fix lights at Eletrica (task 7)
        if (s->lights_sabotaged_timer > 0 && a->activity == ACT_WALKING &&
            a->personality != PERS_IMPULSIVE && (rand() % 100) < 25) {
            a->activity = ACT_FIXING_LIGHTS;
            set_agent_path_to(a, MAP_TASKS[7].x, MAP_TASKS[7].y);
            snprintf(a->last_thought, sizeof(a->last_thought), "Indo religar luzes");
        }
    }

    // Handle fixing lights
    if (a->activity == ACT_FIXING_LIGHTS) {
        if (s->lights_sabotaged_timer == 0) {
            a->activity = ACT_WALKING;
        } else if (a->path_idx < a->path_len) {
            const Waypoint *wp = &MAP_WAYPOINTS[a->path[a->path_idx]];
            move_agent_toward(a, wp->x, wp->y, 11.5f);
            if (map_dist(a->x, a->y, wp->x, wp->y) < 6.0f) a->path_idx++;
        } else {
            move_agent_toward(a, MAP_TASKS[7].x, MAP_TASKS[7].y, 11.5f);
            if (map_dist(a->x, a->y, MAP_TASKS[7].x, MAP_TASKS[7].y) < 12.0f) {
                s->lights_sabotaged_timer = 0;
                a->activity = ACT_WALKING;
                push_event(s, EVT_SYSTEM, a->id, -1, "%s religou as luzes na Eletrica!", a->name);
            }
        }
        return;
    }

    // Task execution / Fake task execution
    if (a->activity == ACT_DOING_TASK || a->activity == ACT_FAKE_TASK) {
        a->task_timer--;
        if (a->task_timer <= 0) {
            if (a->role == ROLE_CREWMATE && a->tasks_completed < MAX_TASKS_PER_AGENT) {
                a->tasks_completed++;
                s->global_tasks_done++;
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

    // Walking along waypoint path to current task
    const TaskPoint *cur_task = &MAP_TASKS[a->task_queue[a->task_idx]];
    if (a->path_idx < a->path_len) {
        const Waypoint *wp = &MAP_WAYPOINTS[a->path[a->path_idx]];
        move_agent_toward(a, wp->x, wp->y, 11.0f);
        if (map_dist(a->x, a->y, wp->x, wp->y) < 8.0f) {
            a->path_idx++;
        }
    } else {
        move_agent_toward(a, cur_task->x, cur_task->y, 11.0f);
        if (map_dist(a->x, a->y, cur_task->x, cur_task->y) < 12.0f) {
            a->activity = (a->role == ROLE_IMPOSTOR) ? ACT_FAKE_TASK : ACT_DOING_TASK;
            a->task_timer = rand_range(7, 12);
            snprintf(a->last_thought, sizeof(a->last_thought),
                     a->role == ROLE_IMPOSTOR ? "Fingindo task..." : "Fazendo %s", cur_task->name);
        }
    }
}

static void step_meeting(SimState *s) {
    s->meeting_step++;

    if (s->meeting_step == 1) {
        // Step 1: Broadcast alibis
        for (int i = 0; i < MAX_AGENTS; i++) {
            Agent *a = &s->agents[i];
            if (!a->alive) continue;
            if (a->role == ROLE_IMPOSTOR) {
                // Bluff a plausible room (avoid body room)
                int fake_room = (s->meeting_body_room + 1 + (rand() % 4)) % MAX_ROOMS;
                a->claimed_room = fake_room;
            }
            push_event(s, EVT_ALIBI, i, -1, "%s: Estava em %s.",
                       a->name,
                       a->claimed_room >= 0 ? MAP_ROOMS[a->claimed_room].name : "Corredor");
        }
    } else if (s->meeting_step == 2) {
        // Step 2: Cross-examine alibis vs memory & emit accusations
        for (int i = 0; i < MAX_AGENTS; i++) {
            Agent *a = &s->agents[i];
            if (!a->alive) continue;

            // Check contradictions
            for (int j = 0; j < MAX_AGENTS; j++) {
                if (i == j || !s->agents[j].alive) continue;
                if (a->role == ROLE_IMPOSTOR && s->agents[j].role == ROLE_IMPOSTOR) continue;

                // If 'a' recently saw 'j' in another room != j->claimed_room
                if (a->last_room_of[j] >= 0 && (int)s->tick - a->last_seen_by_tick[j] <= 22 &&
                    a->last_room_of[j] != s->agents[j].claimed_room && a->role == ROLE_CREWMATE) {
                    a->suspicion[j] = clampf(a->suspicion[j] + 0.30f, 0.0f, 1.0f);
                    push_event(s, EVT_ACCUSE, i, j,
                               "%s: Contradicao! Vi %s em %s, nao em %s!",
                               a->name, s->agents[j].name,
                               MAP_ROOMS[a->last_room_of[j]].name,
                               MAP_ROOMS[s->agents[j].claimed_room].name);
                    break;
                }
            }

            // Pick top suspect to accuse or vouch
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
            if (top >= 0 && top_s >= 0.42f) {
                push_event(s, EVT_ACCUSE, i, top,
                           "%s acusa %s (suspeita %.0f%%).",
                           a->name, s->agents[top].name, top_s * 100.0f);
                // Social influence on other crewmates
                for (int k = 0; k < MAX_AGENTS; k++) {
                    if (k != i && k != top && s->agents[k].alive && s->agents[k].role == ROLE_CREWMATE) {
                        s->agents[k].suspicion[top] = clampf(
                            s->agents[k].suspicion[top] + 0.08f, 0.0f, 1.0f);
                    }
                }
            }
        }
    } else if (s->meeting_step == 3) {
        // Step 3: Cast votes
        for (int i = 0; i < MAX_AGENTS; i++) {
            Agent *a = &s->agents[i];
            if (!a->alive) continue;

            float thresh = 0.45f;
            if (a->personality == PERS_IMPULSIVE) thresh = 0.32f;
            else if (a->personality == PERS_CAUTIOUS) thresh = 0.62f;

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

            // Impostor deflects onto a random crewmate if crewmates are suspicious
            if (a->role == ROLE_IMPOSTOR && best == -1) {
                for (int tries = 0; tries < 6; tries++) {
                    int c = rand() % MAX_AGENTS;
                    if (s->agents[c].alive && s->agents[c].role == ROLE_CREWMATE) {
                        best = c;
                        break;
                    }
                }
            }

            a->vote_target = best;
            if (best >= 0) {
                push_event(s, EVT_VOTE, i, best, "%s votou em %s.", a->name, s->agents[best].name);
            } else {
                push_event(s, EVT_VOTE, i, -1, "%s votou SKIP.", a->name);
            }
        }
    } else if (s->meeting_step >= 4) {
        // Step 4: Tally & eject
        int votes[MAX_AGENTS] = {0};
        int skips = 0;
        for (int i = 0; i < MAX_AGENTS; i++) {
            if (!s->agents[i].alive) continue;
            int vt = s->agents[i].vote_target;
            if (vt >= 0 && vt < MAX_AGENTS) votes[vt]++;
            else skips++;
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
            push_event(s, EVT_EJECT, -1, ejected,
                       "%s foi EJETADO (%d votos). Era %s!",
                       s->agents[ejected].name, max_v,
                       s->agents[ejected].role == ROLE_IMPOSTOR ? "IMPOSTOR" : "Inocente");
        } else {
            push_event(s, EVT_EJECT, -1, -1, "Ninguem ejetado (empate ou skip).");
        }

        // Decay suspicion slightly after meeting & resume roaming
        for (int i = 0; i < MAX_AGENTS; i++) {
            Agent *a = &s->agents[i];
            if (!a->alive) continue;
            for (int j = 0; j < MAX_AGENTS; j++) {
                a->suspicion[j] *= 0.65f;
            }
            const TaskPoint *tp = &MAP_TASKS[a->task_queue[a->task_idx]];
            set_agent_path_to(a, tp->x, tp->y);
            a->activity = ACT_WALKING;
            if (a->role == ROLE_IMPOSTOR) a->kill_cooldown = 18;
        }

        s->phase = PHASE_ROAMING;
        check_win_conditions(s);
    }
}

void sim_step(SimState *s) {
    if (s->phase == PHASE_CREW_WIN || s->phase == PHASE_IMPOSTOR_WIN) return;
    s->tick++;

    if (s->phase == PHASE_MEETING) {
        step_meeting(s);
        return;
    }

    if (s->lights_sabotaged_timer > 0) s->lights_sabotaged_timer--;
    if (s->sabotage_cooldown > 0) s->sabotage_cooldown--;

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
        "\"phase\":%d,\"meeting_step\":%d,\"lights_out\":%s,"
        "\"tasks_done\":%d,\"tasks_total\":%d,\"agents\":[",
        s->tick, s->paused ? "true" : "false", s->speed_ms, s->impostor_count,
        (int)s->phase, s->meeting_step, s->lights_sabotaged_timer > 0 ? "true" : "false",
        s->global_tasks_done, s->global_tasks_total);

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
            "%s{\"id\":%d,\"name\":\"%s\",\"color\":\"%s\",\"role\":%d,"
            "\"pers\":%d,\"alive\":%s,\"ejected\":%s,\"x\":%.1f,\"y\":%.1f,"
            "\"room\":%d,\"act\":%d,\"tasks\":%d,\"top_sus\":%d,\"top_sus_val\":%.2f,"
            "\"thought\":\"%s\"}",
            i > 0 ? "," : "", a->id, a->name, a->color, (int)a->role,
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
