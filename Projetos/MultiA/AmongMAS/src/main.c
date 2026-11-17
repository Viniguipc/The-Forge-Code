#define _WIN32_WINNT 0x0601
#include <winsock2.h>
#include <ws2tcpip.h>
#include <windows.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

#include "sim.h"
#include "map.h"

#define PORT 3040
#define MAX_SSE_CLIENTS 16
#define JSON_BUF_SIZE 16384

static SimState g_sim;
static CRITICAL_SECTION g_lock;
static SOCKET g_sse_clients[MAX_SSE_CLIENTS];
static int g_sse_count = 0;
static char g_json_buf[JSON_BUF_SIZE];

static void broadcast_snapshot(void) {
    EnterCriticalSection(&g_lock);
    int jlen = sim_serialize_json(&g_sim, g_json_buf, JSON_BUF_SIZE);
    char *packet = (char *)malloc((size_t)jlen + 32);
    if (!packet) {
        LeaveCriticalSection(&g_lock);
        return;
    }
    int plen = sprintf(packet, "data: %s\n\n", g_json_buf);

    for (int i = 0; i < g_sse_count; ) {
        int sent = send(g_sse_clients[i], packet, plen, 0);
        if (sent <= 0) {
            closesocket(g_sse_clients[i]);
            g_sse_clients[i] = g_sse_clients[--g_sse_count];
        } else {
            i++;
        }
    }
    free(packet);
    LeaveCriticalSection(&g_lock);
}

static DWORD WINAPI sim_thread(LPVOID param) {
    (void)param;
    for (;;) {
        int sleep_ms = 140;
        bool stepped = false;

        EnterCriticalSection(&g_lock);
        sleep_ms = g_sim.speed_ms;
        if (!g_sim.paused && (g_sim.phase == PHASE_ROAMING || g_sim.phase == PHASE_MEETING)) {
            sim_step(&g_sim);
            stepped = true;
        }
        LeaveCriticalSection(&g_lock);

        if (stepped) {
            broadcast_snapshot();
        }
        Sleep((DWORD)sleep_ms);
    }
    return 0;
}

static void send_http_file(SOCKET client, const char *path, const char *ctype) {
    FILE *f = fopen(path, "rb");
    if (!f) {
        const char *nf = "HTTP/1.1 404 Not Found\r\nContent-Length: 9\r\n\r\nNot Found";
        send(client, nf, (int)strlen(nf), 0);
        return;
    }
    fseek(f, 0, SEEK_END);
    long sz = ftell(f);
    fseek(f, 0, SEEK_SET);
    char *body = (char *)malloc((size_t)sz);
    if (body) {
        fread(body, 1, (size_t)sz, f);
        char hdr[256];
        int hlen = sprintf(hdr,
            "HTTP/1.1 200 OK\r\nContent-Type: %s; charset=utf-8\r\n"
            "Content-Length: %ld\r\nCache-Control: no-cache\r\nConnection: close\r\n\r\n",
            ctype, sz);
        send(client, hdr, hlen, 0);
        send(client, body, (int)sz, 0);
        free(body);
    }
    fclose(f);
}

static const char *find_param(const char *query, const char *key, char *out, int out_sz) {
    char pat[64];
    snprintf(pat, sizeof(pat), "%s=", key);
    const char *p = strstr(query, pat);
    if (!p) return NULL;
    p += strlen(pat);
    int i = 0;
    while (*p && *p != '&' && *p != ' ' && *p != '\r' && *p != '\n' && i < out_sz - 1) {
        out[i++] = *p++;
    }
    out[i] = '\0';
    return out;
}

static void handle_cmd(const char *url) {
    char action[32] = {0};
    char val[32] = {0};
    if (!find_param(url, "action", action, sizeof(action))) return;

    EnterCriticalSection(&g_lock);
    if (strcmp(action, "play") == 0) {
        g_sim.paused = false;
    } else if (strcmp(action, "pause") == 0) {
        g_sim.paused = true;
    } else if (strcmp(action, "step") == 0) {
        g_sim.paused = true;
        sim_step(&g_sim);
    } else if (strcmp(action, "reset") == 0) {
        int imp = g_sim.impostor_count;
        if (find_param(url, "impostors", val, sizeof(val))) {
            imp = atoi(val);
        }
        bool was_paused = g_sim.paused;
        int spd = g_sim.speed_ms;
        sim_reset(&g_sim, imp);
        g_sim.paused = was_paused;
        g_sim.speed_ms = spd;
    } else if (strcmp(action, "speed") == 0) {
        if (find_param(url, "ms", val, sizeof(val))) {
            int ms = atoi(val);
            if (ms < 35) ms = 35;
            if (ms > 1000) ms = 1000;
            g_sim.speed_ms = ms;
        }
    } else if (strcmp(action, "sabotage") == 0) {
        sim_sabotage_lights(&g_sim);
    }
    LeaveCriticalSection(&g_lock);
    broadcast_snapshot();
}

int main(void) {
    srand((unsigned int)time(NULL));
    InitializeCriticalSection(&g_lock);
    sim_init(&g_sim, 1);

    WSADATA wsa;
    if (WSAStartup(MAKEWORD(2, 2), &wsa) != 0) {
        fprintf(stderr, "WSAStartup failed\n");
        return 1;
    }

    SOCKET srv = socket(AF_INET, SOCK_STREAM, IPPROTO_TCP);
    int opt = 1;
    setsockopt(srv, SOL_SOCKET, SO_REUSEADDR, (const char *)&opt, sizeof(opt));

    struct sockaddr_in addr;
    memset(&addr, 0, sizeof(addr));
    addr.sin_family = AF_INET;
    addr.sin_addr.s_addr = htonl(INADDR_LOOPBACK);
    addr.sin_port = htons(PORT);

    if (bind(srv, (struct sockaddr *)&addr, sizeof(addr)) == SOCKET_ERROR) {
        fprintf(stderr, "Bind failed on port %d\n", PORT);
        return 1;
    }
    listen(srv, 16);

    CreateThread(NULL, 0, sim_thread, NULL, 0, NULL);

    printf("AmongMAS C11 Engine listening on http://127.0.0.1:%d\n", PORT);
    fflush(stdout);

    char req[2048];
    for (;;) {
        SOCKET client = accept(srv, NULL, NULL);
        if (client == INVALID_SOCKET) continue;

        int r = recv(client, req, sizeof(req) - 1, 0);
        if (r <= 0) {
            closesocket(client);
            continue;
        }
        req[r] = '\0';

        char method[16] = {0}, url[512] = {0};
        sscanf(req, "%15s %511s", method, url);

        if (strncmp(url, "/events", 7) == 0) {
            const char *sse_hdr =
                "HTTP/1.1 200 OK\r\n"
                "Content-Type: text/event-stream\r\n"
                "Cache-Control: no-cache\r\n"
                "Connection: keep-alive\r\n"
                "Access-Control-Allow-Origin: *\r\n\r\n";
            send(client, sse_hdr, (int)strlen(sse_hdr), 0);

            EnterCriticalSection(&g_lock);
            int jlen = sim_serialize_json(&g_sim, g_json_buf, JSON_BUF_SIZE);
            char *pkt = (char *)malloc((size_t)jlen + 32);
            if (pkt) {
                int plen = sprintf(pkt, "data: %s\n\n", g_json_buf);
                send(client, pkt, plen, 0);
                free(pkt);
            }
            if (g_sse_count < MAX_SSE_CLIENTS) {
                g_sse_clients[g_sse_count++] = client;
            } else {
                closesocket(client);
            }
            LeaveCriticalSection(&g_lock);
        } else if (strncmp(url, "/cmd", 4) == 0) {
            handle_cmd(url);
            const char *ok = "HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nOK";
            send(client, ok, (int)strlen(ok), 0);
            closesocket(client);
        } else {
            send_http_file(client, "web/index.html", "text/html");
            closesocket(client);
        }
    }

    return 0;
}
