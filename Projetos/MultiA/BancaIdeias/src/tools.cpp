#define _WIN32_WINNT 0x0601
#include "tools.hpp"
#include "json_mini.hpp"
#include <windows.h>
#include <winhttp.h>
#include <algorithm>
#include <cctype>
#include <fstream>
#include <set>
#include <sstream>

static std::string to_lower_ascii(const std::string &s) {
    std::string out = s;
    for (char &c : out) c = (char)std::tolower((unsigned char)c);
    return out;
}

static bool is_stopword(const std::string &w) {
    static const std::set<std::string> sw = {
        "para", "com", "sem", "por", "uma", "uns", "umas", "que", "como", "mais",
        "sobre", "entre", "isso", "isto", "esse", "essa", "este", "esta", "dos",
        "das", "nos", "nas", "pelo", "pela", "seu", "sua", "seus", "suas", "ser",
        "ter", "nao", "sim", "onde", "quando", "qual", "quais", "fazer", "criar",
        "ideia", "projeto", "sistema", "aplicacao", "codigo", "arquivo", "the",
        "and", "for", "with", "from", "that", "this", "are", "was", "have"
    };
    return sw.count(w) > 0;
}

static std::vector<std::string> extract_keywords(const std::string &text, size_t max_kw) {
    std::vector<std::string> words;
    std::set<std::string> seen;
    std::string cur;
    for (unsigned char c : text) {
        if (std::isalnum(c)) {
            cur += (char)std::tolower(c);
        } else {
            if (cur.size() >= 3 && !is_stopword(cur) && !seen.count(cur)) {
                seen.insert(cur);
                words.push_back(cur);
                if (words.size() >= max_kw) break;
            }
            cur.clear();
        }
    }
    if (cur.size() >= 3 && !is_stopword(cur) && !seen.count(cur) && words.size() < max_kw) {
        words.push_back(cur);
    }
    return words;
}

// ---------------- LEVEL 2: LIGHTWEIGHT TOKEN-EFFICIENT RAG ----------------

ToolResult rag_retrieve(const std::vector<RagDoc> &docs, const std::string &query,
                        int max_chunks, size_t max_chars) {
    ToolResult res{false, "RAG Local", "", ""};
    if (docs.empty()) return res;

    // If total size of all docs is already small, return directly with zero chunking overhead
    size_t total_sz = 0;
    for (const auto &d : docs) total_sz += d.content.size();
    if (total_sz <= max_chars) {
        std::ostringstream os;
        os << "[RAG - Anexos (" << docs.size() << ")]\n";
        for (const auto &d : docs) {
            os << "--- " << d.name << " ---\n" << d.content << "\n";
        }
        res.ran = true;
        res.summary = os.str();
        res.ui_badge = "RAG: " + std::to_string(docs.size()) + " anexo(s) integral (" +
                       std::to_string(total_sz) + " chars)";
        return res;
    }

    struct Chunk {
        std::string doc_name;
        int idx;
        std::string text;
        int score;
    };
    std::vector<Chunk> chunks;
    auto qwords = extract_keywords(query, 24);

    for (const auto &d : docs) {
        size_t pos = 0;
        int cidx = 1;
        while (pos < d.content.size()) {
            size_t end = std::min(pos + 460, d.content.size());
            if (end < d.content.size()) {
                size_t nl = d.content.rfind('\n', end);
                if (nl != std::string::npos && nl > pos + 150) end = nl + 1;
            }
            std::string sub = d.content.substr(pos, end - pos);
            std::string low = to_lower_ascii(sub);
            int score = 0;
            for (const auto &kw : qwords) {
                if (low.find(kw) != std::string::npos) score += 2;
            }
            // Slight boost for first chunk (headers/title)
            if (cidx == 1) score += 1;
            chunks.push_back({d.name, cidx++, sub, score});
            pos = end;
        }
    }

    std::stable_sort(chunks.begin(), chunks.end(), [](const Chunk &a, const Chunk &b) {
        return a.score > b.score;
    });

    std::ostringstream os;
    os << "[RAG - Trechos mais relevantes de " << docs.size() << " anexo(s)]\n";
    int picked = 0;
    size_t used = 0;
    for (const auto &ch : chunks) {
        if (picked >= max_chunks) break;
        if (used + ch.text.size() > max_chars && picked > 0) break;
        os << "--- " << ch.doc_name << " (trecho #" << ch.idx << ") ---\n"
           << ch.text << "\n";
        used += ch.text.size();
        picked++;
    }

    res.ran = true;
    res.summary = os.str();
    res.ui_badge = "RAG: " + std::to_string(picked) + "/" + std::to_string(chunks.size()) +
                   " trechos selecionados (" + std::to_string(used) + " chars de " +
                   std::to_string(total_sz) + ")";
    return res;
}

// ---------------- LEVEL 3: ZERO-KEY WEB SEARCH TOOL ----------------

static std::wstring to_wide_str(const std::string &s) {
    if (s.empty()) return L"";
    int n = MultiByteToWideChar(CP_UTF8, 0, s.c_str(), (int)s.size(), nullptr, 0);
    std::wstring w(n, L'\0');
    MultiByteToWideChar(CP_UTF8, 0, s.c_str(), (int)s.size(), &w[0], n);
    return w;
}

static std::string http_get_https(const wchar_t *host, const std::string &path) {
    HINTERNET hSession = WinHttpOpen(L"BancaIdeias-Tool/1.0",
                                     WINHTTP_ACCESS_TYPE_DEFAULT_PROXY,
                                     WINHTTP_NO_PROXY_NAME,
                                     WINHTTP_NO_PROXY_BYPASS, 0);
    if (!hSession) return "";
    WinHttpSetTimeouts(hSession, 5000, 5000, 5000, 8000);

    HINTERNET hConnect = WinHttpConnect(hSession, host, INTERNET_DEFAULT_HTTPS_PORT, 0);
    if (!hConnect) { WinHttpCloseHandle(hSession); return ""; }

    std::wstring wpath = to_wide_str(path);
    HINTERNET hReq = WinHttpOpenRequest(hConnect, L"GET", wpath.c_str(),
                                        nullptr, WINHTTP_NO_REFERER,
                                        WINHTTP_DEFAULT_ACCEPT_TYPES,
                                        WINHTTP_FLAG_SECURE);
    if (!hReq) { WinHttpCloseHandle(hConnect); WinHttpCloseHandle(hSession); return ""; }

    const wchar_t *hdrs = L"User-Agent: BancaIdeias-MAS/1.0\r\nAccept: application/json\r\n";
    BOOL ok = WinHttpSendRequest(hReq, hdrs, (DWORD)-1L, nullptr, 0, 0, 0) &&
              WinHttpReceiveResponse(hReq, nullptr);
    std::string raw;
    if (ok) {
        DWORD avail = 0;
        while (WinHttpQueryDataAvailable(hReq, &avail) && avail > 0 && raw.size() < 65536) {
            std::string chunk(avail, '\0');
            DWORD rd = 0;
            if (WinHttpReadData(hReq, &chunk[0], avail, &rd) && rd > 0) {
                raw.append(chunk.data(), rd);
            } else break;
        }
    }
    WinHttpCloseHandle(hReq);
    WinHttpCloseHandle(hConnect);
    WinHttpCloseHandle(hSession);
    return raw;
}

static std::string strip_html_tags(const std::string &s) {
    std::string out;
    bool in_tag = false;
    for (char c : s) {
        if (c == '<') { in_tag = true; continue; }
        if (c == '>') { in_tag = false; continue; }
        if (!in_tag) out += c;
    }
    return out;
}

ToolResult web_search_competitors(const std::string &idea_text) {
    ToolResult res{true, "Busca Web", "", ""};
    auto kws = extract_keywords(idea_text, 4);
    if (kws.empty()) {
        res.ran = false;
        return res;
    }

    std::string q;
    for (size_t i = 0; i < std::min<size_t>(kws.size(), 3); i++) {
        if (i > 0) q += "+";
        q += kws[i];
    }
    std::string q2 = (kws.size() >= 2) ? (kws[0] + "+" + kws[1]) : q;

    std::ostringstream out;
    out << "[BUSCA WEB EXTERNA (query: \"" << q2 << "\")]\n";
    int hits = 0;

    // 1. GitHub Open-Source Competitors Search (try 3 keywords, fallback to 2)
    std::string gh_raw = http_get_https(L"api.github.com",
        "/search/repositories?q=" + q + "&sort=stars&per_page=3");
    if (gh_raw.find("\"full_name\"") == std::string::npos && q2 != q) {
        gh_raw = http_get_https(L"api.github.com",
            "/search/repositories?q=" + q2 + "&sort=stars&per_page=3");
    }
    if (!gh_raw.empty()) {
        size_t pos = gh_raw.find("\"items\"");
        int count = 0;
        while (pos != std::string::npos && count < 3) {
            size_t fn = gh_raw.find("\"full_name\"", pos);
            if (fn == std::string::npos) break;
            size_t nq1 = gh_raw.find('"', fn + 11);
            size_t nq2 = 0;
            std::string name = (nq1 != std::string::npos) ? json_mini::unescape_json_str(gh_raw, nq1, nq2) : "";

            std::string desc;
            size_t dp = gh_raw.find("\"description\"", nq2);
            if (dp != std::string::npos && dp < nq2 + 800) {
                size_t dq1 = gh_raw.find('"', dp + 13);
                size_t nnull = gh_raw.find("null", dp + 13);
                if (dq1 != std::string::npos && (nnull == std::string::npos || dq1 < nnull)) {
                    size_t dq2 = 0;
                    desc = json_mini::unescape_json_str(gh_raw, dq1, dq2);
                    if (desc.size() > 90) desc = desc.substr(0, 87) + "...";
                }
            }
            if (!name.empty()) {
                out << "- GitHub: " << name << (desc.empty() ? "" : " (" + desc + ")") << "\n";
                hits++;
                count++;
            }
            pos = nq2 + 1;
        }
    }

    // 2. Wikipedia PT Domain Context
    std::string wk_q = (kws.size() >= 2) ? (kws[0] + "%20" + kws[1]) : kws[0];
    std::string wk_raw = http_get_https(L"pt.wikipedia.org",
        "/w/api.php?action=query&list=search&srsearch=" + wk_q + "&format=json&srlimit=2");
    if (!wk_raw.empty()) {
        size_t pos = wk_raw.find("\"search\"");
        int count = 0;
        while (pos != std::string::npos && count < 2) {
            size_t tp = wk_raw.find("\"title\"", pos);
            if (tp == std::string::npos) break;
            size_t tq1 = wk_raw.find('"', tp + 7);
            size_t tq2 = 0;
            std::string title = (tq1 != std::string::npos) ? json_mini::unescape_json_str(wk_raw, tq1, tq2) : "";

            std::string snip;
            size_t sp = wk_raw.find("\"snippet\"", tq2);
            if (sp != std::string::npos && sp < tq2 + 400) {
                size_t sq1 = wk_raw.find('"', sp + 9);
                if (sq1 != std::string::npos) {
                    size_t sq2 = 0;
                    snip = strip_html_tags(json_mini::unescape_json_str(wk_raw, sq1, sq2));
                    if (snip.size() > 100) snip = snip.substr(0, 97) + "...";
                }
            }
            if (!title.empty()) {
                out << "- Wiki: " << title << (snip.empty() ? "" : " - " + snip) << "\n";
                hits++;
                count++;
            }
            pos = tq2 + 1;
        }
    }

    if (hits == 0) {
        out << "- Nenhum concorrente direto encontrado para esses termos.\n";
    }

    res.summary = out.str();
    if (res.summary.size() > 680) res.summary = res.summary.substr(0, 677) + "...";
    res.ui_badge = "Busca Web (" + q2 + "): " + std::to_string(hits) + " resultado(s)";
    return res;
}

// ---------------- LEVEL 3: CODE EXECUTION SANDBOX ----------------

static std::string run_cmd_capture(const std::string &cmd_line, DWORD timeout_ms, DWORD &exit_code) {
    SECURITY_ATTRIBUTES sa{};
    sa.nLength = sizeof(sa);
    sa.bInheritHandle = TRUE;

    HANDLE hRead = nullptr, hWrite = nullptr;
    if (!CreatePipe(&hRead, &hWrite, &sa, 0)) {
        exit_code = 1;
        return "[Erro ao criar pipe]";
    }
    SetHandleInformation(hRead, HANDLE_FLAG_INHERIT, 0);

    STARTUPINFOA si{};
    si.cb = sizeof(si);
    si.dwFlags = STARTF_USESTDHANDLES | STARTF_USESHOWWINDOW;
    si.wShowWindow = SW_HIDE;
    si.hStdOutput = hWrite;
    si.hStdError = hWrite;

    PROCESS_INFORMATION pi{};
    std::string cmd = "cmd.exe /C " + cmd_line;
    std::string output;
    exit_code = 1;

    if (CreateProcessA(nullptr, &cmd[0], nullptr, nullptr, TRUE, CREATE_NO_WINDOW,
                       nullptr, nullptr, &si, &pi)) {
        CloseHandle(hWrite);
        hWrite = nullptr;

        DWORD wait_res = WaitForSingleObject(pi.hProcess, timeout_ms);
        if (wait_res == WAIT_TIMEOUT) {
            TerminateProcess(pi.hProcess, 124);
            output = "[TIMEOUT: execucao excedeu o limite]\n";
        }

        char buf[1024];
        DWORD rd = 0;
        while (ReadFile(hRead, buf, sizeof(buf) - 1, &rd, nullptr) && rd > 0) {
            output.append(buf, rd);
            if (output.size() > 2048) break;
        }

        GetExitCodeProcess(pi.hProcess, &exit_code);
        CloseHandle(pi.hProcess);
        CloseHandle(pi.hThread);
    } else {
        output = "[Erro ao iniciar processo]";
    }

    if (hWrite) CloseHandle(hWrite);
    CloseHandle(hRead);
    return output;
}

static bool extract_code_block(const std::string &text, std::string &lang, std::string &code) {
    size_t pos = text.find("```");
    if (pos == std::string::npos) return false;
    size_t nl = text.find('\n', pos + 3);
    if (nl == std::string::npos) return false;
    lang = to_lower_ascii(text.substr(pos + 3, nl - (pos + 3)));
    while (!lang.empty() && (lang.back() == '\r' || lang.back() == ' ')) lang.pop_back();

    size_t end = text.find("```", nl + 1);
    if (end == std::string::npos) return false;
    code = text.substr(nl + 1, end - (nl + 1));
    return !code.empty();
}

ToolResult sandbox_execute_code(const std::string &text_with_code) {
    ToolResult res{false, "Sandbox", "", ""};
    std::string lang, code;
    if (!extract_code_block(text_with_code, lang, code)) {
        // Fallback: if raw code without fences looks like C++/Python/JS
        if (text_with_code.find("#include") != std::string::npos) {
            lang = "cpp";
            code = text_with_code;
        } else if (text_with_code.find("def ") != std::string::npos || text_with_code.find("print(") != std::string::npos) {
            lang = "python";
            code = text_with_code;
        } else if (text_with_code.find("function ") != std::string::npos || text_with_code.find("console.log") != std::string::npos) {
            lang = "js";
            code = text_with_code;
        } else {
            return res;
        }
    }

    CreateDirectoryA("sandbox_tmp", nullptr);
    DWORD exit_code = 0;
    std::string out_log;
    std::string mode_label;

    if (lang == "cpp" || lang == "c++" || lang == "cc") {
        mode_label = "C++ (g++)";
        std::ofstream f("sandbox_tmp\\main.cpp", std::ios::binary);
        f << code;
        f.close();
        out_log = run_cmd_capture("g++ -std=c++17 -O2 -Wall sandbox_tmp\\main.cpp -o sandbox_tmp\\app.exe", 10000, exit_code);
        if (exit_code == 0) {
            std::string run_out = run_cmd_capture("sandbox_tmp\\app.exe", 3000, exit_code);
            out_log = (out_log.empty() ? "" : "[Warnings]\n" + out_log + "\n") + "[Stdout]\n" + run_out;
        } else {
            out_log = "[Erro de Compilacao]\n" + out_log;
        }
    } else if (lang == "c") {
        mode_label = "C (gcc)";
        std::ofstream f("sandbox_tmp\\main.c", std::ios::binary);
        f << code;
        f.close();
        out_log = run_cmd_capture("gcc -O2 -Wall sandbox_tmp\\main.c -o sandbox_tmp\\app.exe", 10000, exit_code);
        if (exit_code == 0) {
            std::string run_out = run_cmd_capture("sandbox_tmp\\app.exe", 3000, exit_code);
            out_log = (out_log.empty() ? "" : "[Warnings]\n" + out_log + "\n") + "[Stdout]\n" + run_out;
        } else {
            out_log = "[Erro de Compilacao]\n" + out_log;
        }
    } else if (lang == "js" || lang == "javascript" || lang == "node") {
        mode_label = "Node.js";
        std::ofstream f("sandbox_tmp\\main.js", std::ios::binary);
        f << code;
        f.close();
        out_log = run_cmd_capture("node sandbox_tmp\\main.js", 3000, exit_code);
    } else if (lang == "py" || lang == "python") {
        mode_label = "Python";
        std::ofstream f("sandbox_tmp\\main.py", std::ios::binary);
        f << code;
        f.close();
        out_log = run_cmd_capture("python sandbox_tmp\\main.py", 3000, exit_code);
    } else {
        res.ran = true;
        res.ui_badge = "Sandbox: linguagem '" + lang + "' nao executavel localmente";
        res.summary = "[SANDBOX] Linguagem '" + lang + "' nao suportada para execucao direta.";
        return res;
    }

    if (out_log.size() > 750) out_log = out_log.substr(0, 747) + "...";
    if (out_log.empty()) out_log = "(sem saida)";

    res.ran = true;
    res.summary = "[SANDBOX " + mode_label + " | exit=" + std::to_string(exit_code) + "]\n" + out_log;
    res.ui_badge = "Sandbox " + mode_label + " (exit=" + std::to_string(exit_code) + ")";
    return res;
}
