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
    ROLE_IMPOSTOR = 1,
    ROLE_NEUTRAL  = 2
} AgentRole;

typedef enum {
    SROLE_NONE = 0,
    // Crewmate special roles (1..6)
    SROLE_DOCTOR       = 1,
    SROLE_INVESTIGATOR = 2,
    SROLE_WATCHER      = 3,
    SROLE_ENGINEER     = 4,
    SROLE_TRACKER      = 5,
    SROLE_MAYOR        = 6,
    // Impostor special roles (7..11)
    SROLE_SHAPESHIFTER = 7,
    SROLE_JANITOR      = 8,
    SROLE_SABOTEUR     = 9,
    SROLE_SILENCER     = 10,
    SROLE_INFILTRATOR  = 11,
    // Neutral (12)
    SROLE_JESTER       = 12
} SpecialRole;

typedef enum {
    PERS_ASTUTO     = 0,
    PERS_CETICO     = 1,
    PERS_BURRO      = 2,
    PERS_BRINCALHAO = 3,
    PERS_PARANOICO  = 4,
    PERS_ANALITICO  = 5,
    PERS_SOCIAL     = 6,
    PERS_MEDROSO    = 7
} Personality;

typedef enum {
    ACT_WALKING       = 0,
    ACT_DOING_TASK    = 1,
    ACT_FAKE_TASK     = 2,
    ACT_HUNTING       = 3,
    ACT_FLEEING       = 4,
    ACT_FIXING_LIGHTS = 5,
    ACT_WATCHING_CAMS = 6,
    ACT_DEAD          = 7
} AgentActivity;

typedef enum {
    PHASE_ROAMING      = 0,
    PHASE_MEETING      = 1,
    PHASE_CREW_WIN     = 2,
    PHASE_IMPOSTOR_WIN = 3,
    PHASE_JESTER_WIN   = 4
} GamePhase;

typedef enum {
    EVT_SYSTEM   = 0,
    EVT_SIGHTING = 1,
    EVT_KILL     = 2,
    EVT_REPORT   = 3,
    EVT_ALIBI    = 4,
    EVT_ACCUSE   = 5,
    EVT_VOUCH    = 6,
    EVT_VOTE     = 7,
    EVT_EJECT    = 8,
    EVT_SABOTAGE = 9,
    EVT_ROLE     = 10
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
    int sender_id;
    int target_id;
    char text[160];
} AclEvent;

typedef struct {
    int id;
    const char *name;
    const char *color;
    const char *symbol;
    AgentRole role;
    SpecialRole srole;
    Personality personality;
    float sus_mult;
    float vote_offset;
    float chaos_prob;
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
    int ability_cooldown;

    // Special role states
    int protected_by;      // agent_id of doctor protecting this agent, or -1
    int protect_timer;
    int tracked_target;    // tracker's target
    int track_timer;
    int disguise_id;       // shapeshifter's copied agent_id, or -1
    int disguise_timer;
    bool silenced;         // silenced for next meeting
    int watch_timer;       // security watcher timer
    int inv_target;        // investigator proximity target
    int inv_ticks;         // ticks spent near inv_target
    bool role_revealed;

    int last_seen_by_tick[MAX_AGENTS];
    int follow_streak[MAX_AGENTS];
    int last_room_of[MAX_AGENTS];

    float suspicion[MAX_AGENTS];
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
    int locked_room;
    int locked_timer;

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
