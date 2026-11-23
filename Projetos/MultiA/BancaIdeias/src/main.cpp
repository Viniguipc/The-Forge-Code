#define _WIN32_WINNT 0x0601
#include <winsock2.h>
#include <ws2tcpip.h>
#include <windows.h>

#include "gemini.hpp"
#include "json_mini.hpp"
#include "tools.hpp"

#include <atomic>
#include <chrono>
#include <cstdlib>
#include <fstream>
#include <iostream>
#include <mutex>
#include <sstream>
#include <string>
#include <thread>
#include <vector>

static const int PORT = 3050;

struct AgentDef {
    std::string name;
    std::string role;
    std::string color;
    std::string symbol;
    std::string system_prompt;
};

struct DebateTurn {
    std::string agent_name;
    std::string role_title;
    std::string color;
    std::string symbol;
    std::string model_used;
    std::string tool_badge;
    int round;
    std::string text;
    long long ms;
};

struct ModelSlot {
    std::string id;
    std::chrono::steady_clock::time_point cooldown_until;
};

static std::vector<ModelSlot> g_cascade = {
    {"gemini-3.8-flash",       std::chrono::steady_clock::time_point{}},
    {"gemini-3.7-flash",       std::chrono::steady_clock::time_point{}},
    {"gemini-3.6-flash",       std::chrono::steady_clock::time_point{}},
    {"gemini-3.5-flash",       std::chrono::steady_clock::time_point{}},
    {"gemini-3-flash-preview", std::chrono::steady_clock::time_point{}},
    {"gemini-3.1-flash-lite",  std::chrono::steady_clock::time_point{}},
    {"gemini-3.5-flash-lite",  std::chrono::steady_clock::time_point{}},
    {"gemma-4-31b-it",         std::chrono::steady_clock::time_point{}},
    {"gemma-4-26b-a4b-it",     std::chrono::steady_clock::time_point{}}
};

struct BancaState {
    bool running = false;
    bool has_key = false;
    std::string status_msg = "Pronto para iniciar.";
    std::string mode = "ideias";
    std::string idea;
    std::string model = "auto";
    int rounds = 1;
    std::vector<std::string> tool_logs;
    std::vector<DebateTurn> turns;
};

static std::mutex g_mtx;
static BancaState g_state;
static std::string g_api_key;
static std::vector<SOCKET> g_sse_clients;

static std::string trim(const std::string &s) {
    size_t a = 0;
    while (a < s.size() && (s[a] == ' ' || s[a] == '\t' || s[a] == '\r' || s[a] == '\n' || s[a] == '"' || s[a] == '\'')) a++;
    size_t b = s.size();
    while (b > a && (s[b - 1] == ' ' || s[b - 1] == '\t' || s[b - 1] == '\r' || s[b - 1] == '\n' || s[b - 1] == '"' || s[b - 1] == '\'')) b--;
    return s.substr(a, b - a);
}

static std::string load_gemini_key() {
    if (const char *env_k = std::getenv("GEMINI_API_KEY")) {
        if (std::string(env_k).size() > 10) return std::string(env_k);
    }
    const char *paths[] = {".env", "../.env", "../../.env", "../../../.env"};
    for (const char *p : paths) {
        std::ifstream f(p);
        if (!f.is_open()) continue;
        std::string line;
        while (std::getline(f, line)) {
            if (line.empty() || line[0] == '#') continue;
            size_t eq = line.find('=');
            if (eq == std::string::npos) continue;
            std::string k = trim(line.substr(0, eq));
            if (k == "GEMINI_API_KEY") {
                std::string v = trim(line.substr(eq + 1));
                if (!v.empty() && v != "coloque_sua_chave_aqui") return v;
            }
        }
    }
    return "";
}

static std::string serialize_state_locked() {
    std::ostringstream os;
    os << "{"
       << "\"running\":" << (g_state.running ? "true" : "false") << ","
       << "\"has_key\":" << (g_state.has_key ? "true" : "false") << ","
       << "\"status_msg\":\"" << json_mini::escape(g_state.status_msg) << "\","
       << "\"mode\":\"" << json_mini::escape(g_state.mode) << "\","
       << "\"idea\":\"" << json_mini::escape(g_state.idea) << "\","
       << "\"model\":\"" << json_mini::escape(g_state.model) << "\","
       << "\"rounds\":" << g_state.rounds << ","
       << "\"tool_logs\":[";
    for (size_t i = 0; i < g_state.tool_logs.size(); i++) {
        if (i > 0) os << ",";
        os << "\"" << json_mini::escape(g_state.tool_logs[i]) << "\"";
    }
    os << "],\"turns\":[";
    for (size_t i = 0; i < g_state.turns.size(); i++) {
        const auto &t = g_state.turns[i];
        if (i > 0) os << ",";
        os << "{"
           << "\"agent_name\":\"" << json_mini::escape(t.agent_name) << "\","
           << "\"role_title\":\"" << json_mini::escape(t.role_title) << "\","
           << "\"color\":\"" << json_mini::escape(t.color) << "\","
           << "\"symbol\":\"" << json_mini::escape(t.symbol) << "\","
           << "\"model_used\":\"" << json_mini::escape(t.model_used) << "\","
           << "\"tool_badge\":\"" << json_mini::escape(t.tool_badge) << "\","
           << "\"round\":" << t.round << ","
           << "\"ms\":" << t.ms << ","
           << "\"text\":\"" << json_mini::escape(t.text) << "\""
           << "}";
    }
    os << "]}";
    return os.str();
}

static void broadcast_sse() {
    std::lock_guard<std::mutex> lk(g_mtx);
    std::string pkt = "data: " + serialize_state_locked() + "\n\n";
    for (size_t i = 0; i < g_sse_clients.size(); ) {
        int sent = send(g_sse_clients[i], pkt.data(), (int)pkt.size(), 0);
        if (sent <= 0) {
            closesocket(g_sse_clients[i]);
            g_sse_clients.erase(g_sse_clients.begin() + (long long)i);
        } else {
            i++;
        }
    }
}

static std::vector<AgentDef> get_agents_for_mode(const std::string &mode) {
    if (mode == "codigo_criacao") {
        return {
            {"Dev Junior", "Gerador Inicial", "#38bdf8", "💻",
             "Voce e um programador junior agil. Implemente a primeira versao funcional do que foi pedido. "
             "Entregue codigo limpo, direto e comentado apenas onde necessario, em portugues."},
            {"Revisor Cetico", "Seguranca & Edge Cases", "#ef4444", "🛡",
             "Voce e um revisor de codigo implacavel. E PROIBIDO elogiar. Analise o codigo gerado acima e aponte "
             "bugs, edge cases nao tratados, falhas de seguranca, vazamentos ou complexidade ruim. Seja direto e tecnico."},
            {"Arquiteto Senior", "Refatoracao & Otimizacao", "#10b981", "⚙",
             "Voce e um engenheiro senior. Reescreva e melhore o codigo incorporando TODAS as correcoes apontadas "
             "pelo Revisor Cetico, mantendo legibilidade, modularidade e tratamento de erros."},
            {"Tech Lead", "Juiz / Entrega Final", "#f59e0b", "⚖",
             "Voce e o Tech Lead final. Revise o debate e entregue: 1) A versao FINAL consolidada do codigo pronta para uso; "
             "2) Lista curta do que foi corrigido e garantido."}
        };
    }
    if (mode == "codigo_avaliacao") {
        return {
            {"Auditor Bugs", "Bugs & Seguranca", "#ef4444", "⚡",
             "Voce e um auditor de seguranca e bugs. E PROIBIDO elogiar. Analise o codigo submetido buscando "
             "vulnerabilidades, race conditions, null/bounds, vazamentos e falhas logicas graves. Maximo 140 palavras."},
            {"Eng. Performance", "Complexidade & Gargalos", "#38bdf8", "🚀",
             "Voce e especialista em performance. Analise complexidade de tempo/espaco, alocacoes desnecessarias, "
             "queries/I/O e gargalos de escala no codigo submetido. Reaja ao Auditor se relevante. Maximo 140 palavras."},
            {"Eng. Qualidade", "Manutenibilidade & Design", "#10b981", "✦",
             "Voce avalia arquitetura, acoplamento, legibilidade e testabilidade. Proponha refatoracoes concretas "
             "para resolver os problemas levantados sem overengineering. Maximo 140 palavras."},
            {"Tech Lead", "Veredito / Code Review", "#f59e0b", "⚖",
             "Voce e o Tech Lead. Leia o codigo e toda a revisao acima. Entregue: 1) Notas (0-10) para Seguranca, "
             "Performance e Manutenibilidade; 2) Veredito: APROVAR, REFATORAR ou REJEITAR; 3) Top 3 correcoes prioritarias."}
        };
    }
    if (mode == "texto_criacao") {
        return {
            {"Autor Criativo", "Rascunho Narrativo", "#a855f7", "✍",
             "Voce e um escritor de ficcao/literatura com voz marcante. Escreva o texto pedido (cena, conto ou trecho) "
             "com atmosfera, conflito e ritmo envolvente em portugues."},
            {"Critico Literario", "Auditor Implacavel", "#ef4444", "🔍",
             "Voce e um critico literario rigoroso. E PROIBIDO elogiar. Aponte cliches, excesso de adjetivos, "
             "furos de motivacao, 'tell don't show' e trechos arrastados no texto acima. Maximo 130 palavras."},
            {"Editor de Estilo", "Lapidacao & Ritmo", "#10b981", "✦",
             "Voce e um editor literario experiente. Reescreva e aprimore o texto aplicando os cortes, melhorias de ritmo "
             "e sugestoes do Critico, elevando a qualidade da prosa."},
            {"Editor-Chefe", "Versao Final", "#f59e0b", "⚖",
             "Voce e o Editor-Chefe. Entregue a versao FINAL polida do texto literario pronta para publicacao, "
             "seguida de uma nota editorial curta (3 linhas) explicando o refinamento feito."}
        };
    }
    if (mode == "texto_avaliacao") {
        return {
            {"Critico Estrutural", "Enredo & Coerencia", "#ef4444", "⚡",
             "Voce e um critico estrutural. E PROIBIDO elogiar de graca. Analise o texto submetido buscando furos de enredo, "
             "falhas de motivacao, conflito fraco ou resolucao forcada. Maximo 130 palavras."},
            {"Analista de Prosa", "Estilo, Voz & Ritmo", "#38bdf8", "📖",
             "Voce analisa tecnica literaria: prosa, dialogos, pacing, repeticoes, cliches e equilibrio entre mostrar e contar. "
             "Cite trechos especificos. Maximo 130 palavras."},
            {"Leitor Beta", "Impacto & Mercado", "#10b981", "✦",
             "Voce representa o leitor exigente. Avalie se o gancho inicial prende, se ha conexao emocional com os personagens "
             "e o que faria voce abandonar ou recomendar a leitura. Maximo 130 palavras."},
            {"Editor-Chefe", "Parecer Editorial", "#f59e0b", "⚖",
             "Voce e o Editor-Chefe. Leia o texto e os 3 pareceres. Entregue: 1) Notas (0-10) para Enredo, Prosa e Engajamento; "
             "2) Veredito: PUBLICAR, REVISAR ou REESCREVER; 3) 3 ajustes cirurgicos prioritarios."}
        };
    }
    // Default: ideias
    return {
        {"Dr. Kael", "Cetico / Red Team", "#ef4444", "⚡",
         "Voce e um auditor implacavel de ideias. A proposta e anonima e de terceiros. "
         "Seu UNICO trabalho e encontrar falhas fatais, custos ocultos, problemas de adocao "
         "e motivos pelos quais isso vai fracassar. E PROIBIDO elogiar ou suavizar criticas. "
         "Responda em portugues, direto ao ponto, maximo 120 palavras."},
        {"Eng. Maya", "Pragmatica Tecnica", "#38bdf8", "⚙",
         "Voce e uma engenheira senior pragmatica. Avalie exclusivamente viabilidade tecnica, "
         "complexidade de construcao, gargalos de escala, manutencao e prazo realista de um MVP. "
         "Reaja aos pontos levantados anteriormente se houver. Maximo 120 palavras em portugues."},
        {"Soren", "Estrategista de Produto", "#10b981", "✦",
         "Voce e um estrategista de produto focado em valor real e mercado. Nao elogie de graca: "
         "identifique quem realmente usaria/pagaria por isso, qual o diferencial concreto e proponha "
         "cortes de escopo ou pivos para responder as criticas feitas. Maximo 120 palavras em portugues."},
        {"Juiza Iris", "Arbitra Final", "#f59e0b", "⚖",
         "Voce e a juiza imparcial da banca. Leia a proposta anonima e TODO o debate anterior. "
         "Entregue exatamente:\n"
         "1) Notas (0 a 10): Viabilidade Tecnica, Diferencial, Risco/Execucao.\n"
         "2) Veredito: APROVAR, PIVOTAR ou DESCARTAR.\n"
         "3) 3 acoes praticas imediatas. Maximo 160 palavras em portugues."}
    };
}

static std::string build_transcript(const std::string &idea, const std::string &rag_ctx,
                                    const std::vector<DebateTurn> &turns) {
    std::ostringstream os;
    os << "PROPOSTA ANONIMA PARA AUDITORIA:\n" << idea << "\n\n";
    if (!rag_ctx.empty()) {
        os << rag_ctx << "\n";
    }
    if (!turns.empty()) {
        os << "HISTORICO DO DEBATE ATE AQUI:\n";
        size_t n = turns.size();
        for (size_t i = 0; i < n; i++) {
            const auto &t = turns[i];
            std::string body = t.text;
            // Token-saving compaction: older turns beyond last 3 are capped at ~420 chars
            if (n > 3 && i + 3 < n && body.size() > 420) {
                body = body.substr(0, 417) + "...";
            }
            os << "[Rodada " << t.round << " - " << t.agent_name << " (" << t.role_title << ")]: "
               << body << "\n\n";
        }
    }
    return os.str();
}

static LlmResponse call_with_cascade(
    const std::string &requested_model,
    const std::string &system_prompt,
    const std::string &user_prompt,
    const std::string &agent_label,
    std::string &out_model_used
) {
    if (requested_model != "auto") {
        out_model_used = requested_model;
        return call_gemini_api(g_api_key, requested_model, system_prompt, user_prompt);
    }

    auto now = std::chrono::steady_clock::now();
    LlmResponse last_res{false, 0, "[ERRO] Todos os modelos da cascata estao em cooldown."};

    for (size_t i = 0; i < g_cascade.size(); i++) {
        std::string mid;
        {
            std::lock_guard<std::mutex> lk(g_mtx);
            if (g_cascade[i].cooldown_until > now) continue;
            mid = g_cascade[i].id;
            g_state.status_msg = agent_label + " usando " + mid + "...";
        }
        broadcast_sse();

        LlmResponse resp = call_gemini_api(g_api_key, mid, system_prompt, user_prompt);
        if (resp.ok) {
            out_model_used = mid;
            return resp;
        }

        last_res = resp;
        // Put model in cooldown on 429 (RPM/RPD) or 404/400 (unavailable)
        int cd_secs = (resp.status_code == 429) ? 65 : 3600;
        {
            std::lock_guard<std::mutex> lk(g_mtx);
            g_cascade[i].cooldown_until = std::chrono::steady_clock::now() + std::chrono::seconds(cd_secs);
            g_state.status_msg = mid + " esgotou (" + std::to_string(resp.status_code) + "), tentando proximo...";
        }
        broadcast_sse();
        Sleep(200);
    }

    out_model_used = "nenhum";
    return last_res;
}

static void run_debate_worker(std::string mode, std::string idea, std::string model,
                              int rounds, std::vector<RagDoc> docs) {
    if (rounds < 1) rounds = 1;
    if (rounds > 2) rounds = 2;

    std::vector<AgentDef> agents = get_agents_for_mode(mode);
    std::vector<DebateTurn> local_turns;

    // Level 2: Lightweight token-efficient RAG
    std::string rag_ctx;
    if (!docs.empty()) {
        {
            std::lock_guard<std::mutex> lk(g_mtx);
            g_state.status_msg = "Executando RAG local nos anexos...";
        }
        broadcast_sse();
        ToolResult rag = rag_retrieve(docs, idea, 3, 1400);
        if (rag.ran) {
            rag_ctx = rag.summary;
            std::lock_guard<std::mutex> lk(g_mtx);
            g_state.tool_logs.push_back(rag.ui_badge);
        }
        broadcast_sse();
    }

    // Level 3: Pre-turn tool outputs (Web Search for Ideias / Sandbox for Code Review)
    std::string pending_tool_ctx;
    std::string pending_tool_badge;

    if (mode == "ideias") {
        {
            std::lock_guard<std::mutex> lk(g_mtx);
            g_state.status_msg = "Dr. Kael consultando Busca Web (concorrentes)...";
        }
        broadcast_sse();
        ToolResult ws = web_search_competitors(idea);
        if (ws.ran) {
            pending_tool_ctx = ws.summary + "\n";
            pending_tool_badge = ws.ui_badge;
            std::lock_guard<std::mutex> lk(g_mtx);
            g_state.tool_logs.push_back(ws.ui_badge);
        }
        broadcast_sse();
    } else if (mode == "codigo_avaliacao") {
        {
            std::lock_guard<std::mutex> lk(g_mtx);
            g_state.status_msg = "Auditor executando codigo na Sandbox...";
        }
        broadcast_sse();
        std::string code_src = idea + "\n" + rag_ctx;
        ToolResult sb = sandbox_execute_code(code_src);
        if (sb.ran) {
            pending_tool_ctx = sb.summary + "\n";
            pending_tool_badge = sb.ui_badge;
            std::lock_guard<std::mutex> lk(g_mtx);
            g_state.tool_logs.push_back(sb.ui_badge);
        }
        broadcast_sse();
    }

    for (int r = 1; r <= rounds; r++) {
        for (int idx = 0; idx < 3; idx++) {
            const auto &ag = agents[idx];
            {
                std::lock_guard<std::mutex> lk(g_mtx);
                g_state.status_msg = ag.name + " (" + ag.role + ") - Rodada " + std::to_string(r) + "...";
            }
            broadcast_sse();

            std::string prompt = build_transcript(idea, rag_ctx, local_turns);
            std::string used_badge = pending_tool_badge;
            if (!pending_tool_ctx.empty()) {
                prompt += pending_tool_ctx + "\n";
                pending_tool_ctx.clear();
                pending_tool_badge.clear();
            }

            if (r == 1) {
                prompt += "De o seu parecer inicial seguindo rigorosamente seu papel.";
            } else {
                prompt += "Rodada de replica: responda diretamente aos argumentos dos outros membros da banca e refine sua posicao.";
            }

            auto t0 = std::chrono::steady_clock::now();
            std::string model_used;
            LlmResponse resp = call_with_cascade(model, ag.system_prompt, prompt, ag.name, model_used);
            long long ms = std::chrono::duration_cast<std::chrono::milliseconds>(
                               std::chrono::steady_clock::now() - t0).count();

            DebateTurn turn{ag.name, ag.role, ag.color, ag.symbol, model_used, used_badge, r, resp.text, ms};
            local_turns.push_back(turn);

            {
                std::lock_guard<std::mutex> lk(g_mtx);
                g_state.turns.push_back(turn);
            }
            broadcast_sse();

            // Reflexion Sandbox in codigo_criacao: test code after Dev Junior (idx 0) & Arquiteto (idx 2)
            if (mode == "codigo_criacao" && (idx == 0 || idx == 2)) {
                {
                    std::lock_guard<std::mutex> lk(g_mtx);
                    g_state.status_msg = "Sandbox compilando/testando codigo de " + ag.name + "...";
                }
                broadcast_sse();
                ToolResult sb = sandbox_execute_code(resp.text);
                if (sb.ran) {
                    pending_tool_ctx = sb.summary + "\n";
                    pending_tool_badge = sb.ui_badge;
                    std::lock_guard<std::mutex> lk(g_mtx);
                    g_state.tool_logs.push_back(ag.name + " -> " + sb.ui_badge);
                }
                broadcast_sse();
            }

            Sleep(350);
        }
    }

    // Final Judge Verdict
    const auto &judge = agents[3];
    {
        std::lock_guard<std::mutex> lk(g_mtx);
        g_state.status_msg = judge.name + " formulando veredito final...";
    }
    broadcast_sse();

    std::string j_prompt = build_transcript(idea, rag_ctx, local_turns);
    std::string j_badge = pending_tool_badge;
    if (!pending_tool_ctx.empty()) {
        j_prompt += pending_tool_ctx + "\n";
    }
    j_prompt += "Com base em todo o debate acima, emita seu veredito final.";
    auto t0 = std::chrono::steady_clock::now();
    std::string j_model_used;
    LlmResponse j_resp = call_with_cascade(model, judge.system_prompt, j_prompt, judge.name, j_model_used);
    long long ms = std::chrono::duration_cast<std::chrono::milliseconds>(
                       std::chrono::steady_clock::now() - t0).count();

    DebateTurn j_turn{judge.name, judge.role, judge.color, judge.symbol, j_model_used, j_badge, rounds, j_resp.text, ms};
    {
        std::lock_guard<std::mutex> lk(g_mtx);
        g_state.turns.push_back(j_turn);
        g_state.running = false;
        g_state.status_msg = "Avaliacao concluida!";
    }
    broadcast_sse();
}

static void send_http_file(SOCKET client, const char *path, const char *ctype) {
    std::ifstream f(path, std::ios::binary);
    if (!f.is_open()) {
        const char *nf = "HTTP/1.1 404 Not Found\r\nContent-Length: 9\r\n\r\nNot Found";
        send(client, nf, (int)strlen(nf), 0);
        return;
    }
    std::ostringstream ss;
    ss << f.rdbuf();
    std::string body = ss.str();
    std::ostringstream hdr;
    hdr << "HTTP/1.1 200 OK\r\n"
        << "Content-Type: " << ctype << "; charset=utf-8\r\n"
        << "Content-Length: " << body.size() << "\r\n"
        << "Cache-Control: no-cache\r\nConnection: close\r\n\r\n";
    std::string h = hdr.str();
    send(client, h.data(), (int)h.size(), 0);
    send(client, body.data(), (int)body.size(), 0);
}

int main() {
    g_api_key = load_gemini_key();
    {
        std::lock_guard<std::mutex> lk(g_mtx);
        g_state.has_key = !g_api_key.empty();
        if (!g_state.has_key) {
            g_state.status_msg = "GEMINI_API_KEY nao encontrada no .env!";
        }
    }

    WSADATA wsa;
    if (WSAStartup(MAKEWORD(2, 2), &wsa) != 0) {
        std::cerr << "WSAStartup failed\n";
        return 1;
    }

    SOCKET srv = socket(AF_INET, SOCK_STREAM, IPPROTO_TCP);
    int opt = 1;
    setsockopt(srv, SOL_SOCKET, SO_REUSEADDR, (const char *)&opt, sizeof(opt));

    sockaddr_in addr{};
    addr.sin_family = AF_INET;
    addr.sin_addr.s_addr = htonl(INADDR_LOOPBACK);
    addr.sin_port = htons(PORT);

    if (bind(srv, (sockaddr *)&addr, sizeof(addr)) == SOCKET_ERROR) {
        std::cerr << "Bind failed on port " << PORT << "\n";
        return 1;
    }
    listen(srv, 16);

    std::cout << "BancaIdeias C++20/WinHTTP server listening on http://127.0.0.1:" << PORT << "\n";
    std::cout << (g_api_key.empty() ? "[AVISO] GEMINI_API_KEY nao encontrada no .env\n"
                                    : "[OK] GEMINI_API_KEY carregada.\n");

    for (;;) {
        SOCKET client = accept(srv, nullptr, nullptr);
        if (client == INVALID_SOCKET) continue;

        char buf[8192];
        int r = recv(client, buf, sizeof(buf) - 1, 0);
        if (r <= 0) {
            closesocket(client);
            continue;
        }
        buf[r] = '\0';
        std::string req(buf, (size_t)r);

        std::string method, url;
        {
            std::istringstream iss(req);
            iss >> method >> url;
        }

        if (url == "/events") {
            const char *sse_hdr =
                "HTTP/1.1 200 OK\r\n"
                "Content-Type: text/event-stream\r\n"
                "Cache-Control: no-cache\r\n"
                "Connection: keep-alive\r\n\r\n";
            send(client, sse_hdr, (int)strlen(sse_hdr), 0);

            std::lock_guard<std::mutex> lk(g_mtx);
            std::string pkt = "data: " + serialize_state_locked() + "\n\n";
            send(client, pkt.data(), (int)pkt.size(), 0);
            g_sse_clients.push_back(client);
        } else if (method == "POST" && url == "/evaluate") {
            // Ensure we read the full Content-Length body across TCP packets
            size_t hdr_end = req.find("\r\n\r\n");
            while (hdr_end == std::string::npos && req.size() < 32768) {
                int r2 = recv(client, buf, sizeof(buf) - 1, 0);
                if (r2 <= 0) break;
                req.append(buf, (size_t)r2);
                hdr_end = req.find("\r\n\r\n");
            }
            std::string body;
            if (hdr_end != std::string::npos) {
                int content_len = 0;
                std::string hdrs = req.substr(0, hdr_end);
                for (char &c : hdrs) c = (char)tolower((unsigned char)c);
                size_t cl_pos = hdrs.find("content-length:");
                if (cl_pos != std::string::npos) {
                    content_len = std::atoi(hdrs.c_str() + cl_pos + 15);
                }
                body = req.substr(hdr_end + 4);
                if (content_len > 262144) content_len = 262144;
                while ((int)body.size() < content_len) {
                    int r2 = recv(client, buf, sizeof(buf) - 1, 0);
                    if (r2 <= 0) break;
                    body.append(buf, (size_t)r2);
                }
            }

            std::string mode = json_mini::extract_json_field(body, "mode");
            std::string idea = json_mini::extract_json_field(body, "idea");
            std::string model = json_mini::extract_json_field(body, "model");
            std::string rstr = json_mini::extract_json_field(body, "rounds");
            auto raw_atts = json_mini::extract_attachments(body);
            std::vector<RagDoc> docs;
            for (const auto &a : raw_atts) {
                docs.push_back({a.name, a.content});
            }
            if (mode.empty()) mode = "ideias";
            if (model.empty()) model = "auto";
            int rounds = rstr.empty() ? 1 : std::atoi(rstr.c_str());

            std::cout << "[POST /evaluate] mode=" << mode << " idea_len=" << idea.size()
                      << " docs=" << docs.size() << " model=" << model << " rounds=" << rounds << "\n";

            bool started = false;
            {
                std::lock_guard<std::mutex> lk(g_mtx);
                if (g_api_key.empty()) {
                    g_api_key = load_gemini_key();
                    g_state.has_key = !g_api_key.empty();
                }
                if (g_api_key.empty()) {
                    g_state.status_msg = "Erro: GEMINI_API_KEY nao encontrada no .env!";
                } else if (idea.empty() && docs.empty()) {
                    g_state.status_msg = "Erro: texto e anexos vazios.";
                } else if (!g_state.running) {
                    if (idea.empty()) idea = "Analise o(s) arquivo(s) anexado(s).";
                    g_state.running = true;
                    g_state.mode = mode;
                    g_state.idea = idea;
                    g_state.model = model;
                    g_state.rounds = rounds;
                    g_state.tool_logs.clear();
                    g_state.turns.clear();
                    g_state.status_msg = "Iniciando banca...";
                    started = true;
                }
            }
            broadcast_sse();
            if (started) {
                std::thread(run_debate_worker, mode, idea, model, rounds, std::move(docs)).detach();
            }
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
