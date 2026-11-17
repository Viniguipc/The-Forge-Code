#ifndef TYPES_H
#define TYPES_H

#include <stdbool.h>
#include <stdint.h>

#define MAX_AGENTS 10
#define MAX_ROOMS 6
#define MAX_ASTAR_PATH 180
#define MAX_WALLS 48
#define MAX_TASKS 10
#define MAX_VENTS 4
#define MAX_BODIES 10
#define MAX_MEMORY 24
#define MAX_EVENTS 64
#define MAX_TASKS_PER_AGENT 4

#define WORLD_W 1000.0f
#define WORLD_H 600.0f

typedef enum {
    ROLE_CREWMATE = 0,
    ROLE_IMPOSTOR = 1
} AgentRole;

typedef enum {
    PERS_ANALYTICAL = 0, // Foca em contradicoes e fake tasks
    PERS_CAUTIOUS   = 1, // Exige alta confianca para acusar/votar
    PERS_IMPULSIVE  = 2, // Acusa e vota rapido com pouca evidencia
    PERS_SOCIAL     = 3  // Valoriza alibis e confirma testemunhas
} Personality;

typedef enum {
    ACT_WALKING = 0,
    ACT_DOING_TASK = 1,
    ACT_FAKE_TASK = 2,
    ACT_HUNTING = 3,
    ACT_FLEEING = 4,
    ACT_FIXING_LIGHTS = 5,
    ACT_DEAD = 6
} AgentActivity;

typedef enum {
    PHASE_ROAMING = 0,
    PHASE_MEETING = 1,
    PHASE_CREW_WIN = 2,
    PHASE_IMPOSTOR_WIN = 3
} GamePhase;

typedef enum {
    EVT_SYSTEM = 0,
    EVT_SIGHTING = 1,
    EVT_KILL = 2,
    EVT_REPORT = 3,
    EVT_ALIBI = 4,
    EVT_ACCUSE = 5,
    EVT_VOUCH = 6,
    EVT_VOTE = 7,
    EVT_EJECT = 8,
    EVT_SABOTAGE = 9
} EventKind;

typedef struct {
    float x1, y1, x2, y2;
} WallSeg;

typedef struct {
    int id;
    const char *name;
    float x, y, w, h;
    float center_x, center_y;
} Room;

typedef struct {
    int id;
    int room_id;
    const char *name;
    float x, y;
} TaskPoint;

typedef struct {
    int id;
    int room_id;
    float x, y;
    int linked_vent_id;
} Vent;

typedef struct {
    int agent_id;
    int room_id;
    uint32_t tick;
    AgentActivity activity;
    bool valid;
} Sighting;

typedef struct {
    int victim_id;
    int room_id;
    float x, y;
    uint32_t tick_killed;
    bool reported;
} DeadBody;

typedef struct {
    uint32_t id;
    uint32_t tick;
    EventKind kind;
    int sender_id;   // -1 = System
    int target_id;   // -1 = All
    char text[160];
} AclEvent;

typedef struct {
    int id;
    const char *name;
    const char *color;
    AgentRole role;
    Personality personality;
    bool alive;
    bool ejected;

    float x, y;
    float vx, vy;
    int current_room;
    float path_x[MAX_ASTAR_PATH];
    float path_y[MAX_ASTAR_PATH];
    int path_len;
    int path_idx;
    float goal_x, goal_y;

    AgentActivity activity;
    int task_queue[MAX_TASKS_PER_AGENT];
    int task_idx;
    int tasks_completed;
    int task_timer;

    int kill_cooldown;
    int vent_cooldown;
    int last_seen_by_tick[MAX_AGENTS];
    int follow_streak[MAX_AGENTS];
    int last_room_of[MAX_AGENTS];

    float suspicion[MAX_AGENTS]; // 0.0 .. 1.0
    Sighting memory[MAX_MEMORY];
    int memory_head;

    int claimed_room;
    int vote_target; // -1 = skip, -2 = not voted yet
    char last_thought[96];
} Agent;

typedef struct {
    uint32_t tick;
    bool paused;
    int speed_ms;
    int impostor_count; // 1, 2, or 3
    GamePhase phase;

    int meeting_step;
    int meeting_reporter;
    int meeting_body_room;
    int meeting_victim;

    int lights_sabotaged_timer;
    int sabotage_cooldown;
    int global_tasks_done;
    int global_tasks_total;
    int prev_global_tasks_done;

    Agent agents[MAX_AGENTS];
    DeadBody bodies[MAX_BODIES];
    int body_count;

    AclEvent events[MAX_EVENTS];
    int event_count;
    uint32_t next_event_id;
} SimState;

#endif
