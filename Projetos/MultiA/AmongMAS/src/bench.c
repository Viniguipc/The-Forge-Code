#include <stdio.h>
#include <stdlib.h>
#include <time.h>
#include "sim.h"
#include "map.h"

static const char *PERS_LABELS[8] = {
    "Astuto", "Cetico", "Burro", "Brincalhao",
    "Paranoico", "Analitico", "Social", "Medroso"
};

static void run_batch(int impostor_count, int num_games) {
    int crew_win_eject = 0;
    int crew_win_tasks = 0;
    int imp_win = 0;
    int jester_win = 0;
    int timeouts = 0;

    long long total_ticks = 0;
    long long total_kills = 0;
    long long total_meetings = 0;
    long long correct_ejects = 0;
    long long wrong_ejects = 0;

    int pers_games_crew[8] = {0}, pers_wins_crew[8] = {0};
    int pers_games_imp[8] = {0}, pers_wins_imp[8] = {0};

    SimState s;
    for (int g = 0; g < num_games; g++) {
        sim_init(&s, impostor_count);

        int max_ticks = 2500;
        uint32_t last_ev_id = 0;
        int meetings = 0;

        while (s.phase == PHASE_ROAMING || s.phase == PHASE_MEETING) {
            if ((int)s.tick >= max_ticks) {
                timeouts++;
                break;
            }
            sim_step(&s);

            for (int e = 0; e < s.event_count; e++) {
                if (s.events[e].id >= last_ev_id) {
                    last_ev_id = s.events[e].id + 1;
                    if (s.events[e].kind == EVT_REPORT) meetings++;
                    if (s.events[e].kind == EVT_EJECT && s.events[e].target_id >= 0) {
                        int tgt = s.events[e].target_id;
                        if (s.agents[tgt].role == ROLE_IMPOSTOR) correct_ejects++;
                        else wrong_ejects++;
                    }
                }
            }
        }

        total_ticks += s.tick;
        total_meetings += meetings;
        total_kills += s.body_count;

        bool crew_won = (s.phase == PHASE_CREW_WIN);
        if (s.phase == PHASE_CREW_WIN) {
            if (s.global_tasks_total > 0 && s.global_tasks_done >= s.global_tasks_total) crew_win_tasks++;
            else crew_win_eject++;
        } else if (s.phase == PHASE_IMPOSTOR_WIN) {
            imp_win++;
        } else if (s.phase == PHASE_JESTER_WIN) {
            jester_win++;
        }

        for (int i = 0; i < MAX_AGENTS; i++) {
            int p = (int)s.agents[i].personality;
            if (s.agents[i].role == ROLE_CREWMATE) {
                pers_games_crew[p]++;
                if (crew_won) pers_wins_crew[p]++;
            } else {
                pers_games_imp[p]++;
                if (s.phase == PHASE_IMPOSTOR_WIN) pers_wins_imp[p]++;
            }
        }
    }

    int crew_total = crew_win_eject + crew_win_tasks;
    printf("\n=== %d IMPOSTOR%s (%d partidas) ===\n",
           impostor_count, impostor_count > 1 ? "ES" : "", num_games);
    printf("Vitoria Tripulantes: %d (%.1f%%) [Ejetando: %d, Tasks: %d]\n",
           crew_total, 100.0 * crew_total / num_games, crew_win_eject, crew_win_tasks);
    printf("Vitoria Impostores : %d (%.1f%%)\n",
           imp_win, 100.0 * imp_win / num_games);
    printf("Vitoria Bobo       : %d (%.1f%%)\n",
           jester_win, 100.0 * jester_win / num_games);
    if (timeouts > 0) printf("Timeouts           : %d\n", timeouts);
    printf("Media ticks/partida: %.1f | Reunioes: %.2f | Kills: %.2f\n",
           (double)total_ticks / num_games,
           (double)total_meetings / num_games,
           (double)total_kills / num_games);
    printf("Ejecoes            : Corretas=%lld | Inocentes=%lld (precisao %.1f%%)\n",
           correct_ejects, wrong_ejects,
           (correct_ejects + wrong_ejects) > 0
               ? 100.0 * correct_ejects / (correct_ejects + wrong_ejects)
               : 0.0);

    printf("Winrate por Personalidade (Crew / Imp):\n");
    for (int p = 0; p < 8; p++) {
        double cw = pers_games_crew[p] > 0 ? 100.0 * pers_wins_crew[p] / pers_games_crew[p] : 0.0;
        double iw = pers_games_imp[p] > 0 ? 100.0 * pers_wins_imp[p] / pers_games_imp[p] : 0.0;
        printf("  %-11s: Crew %.1f%% (%d) | Imp %.1f%% (%d)\n",
               PERS_LABELS[p], cw, pers_games_crew[p], iw, pers_games_imp[p]);
    }
}

int main(void) {
    srand((unsigned int)time(NULL));
    clock_t t0 = clock();
    run_batch(1, 500);
    run_batch(2, 500);
    run_batch(3, 500);
    double secs = (double)(clock() - t0) / CLOCKS_PER_SEC;
    printf("\n1500 simulacoes concluidas em %.3fs!\n", secs);
    return 0;
}
