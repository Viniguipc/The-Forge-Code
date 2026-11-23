#define _WIN32_WINNT 0x0601
#include "gemini.hpp"
#include "json_mini.hpp"
#include <windows.h>
#include <winhttp.h>
#include <string>

static std::wstring to_wide(const std::string &s) {
    if (s.empty()) return L"";
    int n = MultiByteToWideChar(CP_UTF8, 0, s.c_str(), (int)s.size(), nullptr, 0);
    std::wstring w(n, L'\0');
    MultiByteToWideChar(CP_UTF8, 0, s.c_str(), (int)s.size(), &w[0], n);
    return w;
}

LlmResponse call_gemini_api(
    const std::string &api_key,
    const std::string &model,
    const std::string &system_prompt,
    const std::string &user_prompt
) {
    LlmResponse res{false, 0, ""};

    std::string body;
    if (model.rfind("gemma", 0) == 0) {
        std::string combined = "[INSTRUCOES DO SEU PAPEL]\n" + system_prompt + "\n\n[DEBATE]\n" + user_prompt;
        body = "{\"contents\":[{\"role\":\"user\",\"parts\":[{\"text\":\"" +
               json_mini::escape(combined) +
               "\"}]}],\"generationConfig\":{\"temperature\":0.85,\"maxOutputTokens\":4096}}";
    } else {
        body = "{\"system_instruction\":{\"parts\":[{\"text\":\"" +
               json_mini::escape(system_prompt) +
               "\"}]},\"contents\":[{\"role\":\"user\",\"parts\":[{\"text\":\"" +
               json_mini::escape(user_prompt) +
               "\"}]}],\"generationConfig\":{\"temperature\":0.85,\"maxOutputTokens\":4096}}";
    }

    HINTERNET hSession = WinHttpOpen(L"BancaIdeias-CPP20/1.0",
                                     WINHTTP_ACCESS_TYPE_DEFAULT_PROXY,
                                     WINHTTP_NO_PROXY_NAME,
                                     WINHTTP_NO_PROXY_BYPASS, 0);
    if (!hSession) {
        res.text = "[ERRO] WinHttpOpen falhou.";
        return res;
    }

    // 45s timeout
    WinHttpSetTimeouts(hSession, 10000, 10000, 15000, 45000);

    HINTERNET hConnect = WinHttpConnect(hSession, L"generativelanguage.googleapis.com",
                                        INTERNET_DEFAULT_HTTPS_PORT, 0);
    if (!hConnect) {
        WinHttpCloseHandle(hSession);
        res.text = "[ERRO] WinHttpConnect falhou.";
        return res;
    }

    std::string path_str = "/v1beta/models/" + model + ":generateContent?key=" + api_key;
    std::wstring wpath = to_wide(path_str);

    HINTERNET hRequest = WinHttpOpenRequest(hConnect, L"POST", wpath.c_str(),
                                            nullptr, WINHTTP_NO_REFERER,
                                            WINHTTP_DEFAULT_ACCEPT_TYPES,
                                            WINHTTP_FLAG_SECURE);
    if (!hRequest) {
        WinHttpCloseHandle(hConnect);
        WinHttpCloseHandle(hSession);
        res.text = "[ERRO] WinHttpOpenRequest falhou.";
        return res;
    }

    const wchar_t *headers = L"Content-Type: application/json\r\n";
    BOOL sent = WinHttpSendRequest(hRequest, headers, (DWORD)-1L,
                                   (LPVOID)body.data(), (DWORD)body.size(),
                                   (DWORD)body.size(), 0);
    if (!sent || !WinHttpReceiveResponse(hRequest, nullptr)) {
        WinHttpCloseHandle(hRequest);
        WinHttpCloseHandle(hConnect);
        WinHttpCloseHandle(hSession);
        res.text = "[ERRO] Falha na requisicao HTTPS para o Gemini.";
        return res;
    }

    DWORD status = 0, sz = sizeof(status);
    WinHttpQueryHeaders(hRequest, WINHTTP_QUERY_STATUS_CODE | WINHTTP_QUERY_FLAG_NUMBER,
                        WINHTTP_HEADER_NAME_BY_INDEX, &status, &sz, WINHTTP_NO_HEADER_INDEX);
    res.status_code = (int)status;

    std::string raw;
    DWORD avail = 0;
    while (WinHttpQueryDataAvailable(hRequest, &avail) && avail > 0) {
        std::string chunk(avail, '\0');
        DWORD read_bytes = 0;
        if (WinHttpReadData(hRequest, &chunk[0], avail, &read_bytes) && read_bytes > 0) {
            raw.append(chunk.data(), read_bytes);
        } else {
            break;
        }
    }

    WinHttpCloseHandle(hRequest);
    WinHttpCloseHandle(hConnect);
    WinHttpCloseHandle(hSession);

    res.text = json_mini::extract_gemini_text(raw);
    res.ok = (status >= 200 && status < 300);
    return res;
}
