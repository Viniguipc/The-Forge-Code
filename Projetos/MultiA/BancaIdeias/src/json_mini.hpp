#pragma once
#include <string>
#include <vector>

namespace json_mini {

inline std::string escape(const std::string &s) {
    std::string out;
    out.reserve(s.size() + 16);
    for (unsigned char c : s) {
        switch (c) {
            case '"':  out += "\\\""; break;
            case '\\': out += "\\\\"; break;
            case '\b': out += "\\b"; break;
            case '\f': out += "\\f"; break;
            case '\n': out += "\\n"; break;
            case '\r': out += "\\r"; break;
            case '\t': out += "\\t"; break;
            default:
                if (c < 0x20) {
                    char buf[8];
                    snprintf(buf, sizeof(buf), "\\u%04x", c);
                    out += buf;
                } else {
                    out += static_cast<char>(c);
                }
        }
    }
    return out;
}

inline std::string unescape_json_str(const std::string &raw, size_t start_quote, size_t &end_quote) {
    std::string out;
    size_t i = start_quote + 1;
    while (i < raw.size()) {
        char c = raw[i];
        if (c == '"') {
            end_quote = i;
            return out;
        }
        if (c == '\\' && i + 1 < raw.size()) {
            char nxt = raw[i + 1];
            switch (nxt) {
                case '"':  out += '"'; i += 2; continue;
                case '\\': out += '\\'; i += 2; continue;
                case '/':  out += '/'; i += 2; continue;
                case 'b':  out += '\b'; i += 2; continue;
                case 'f':  out += '\f'; i += 2; continue;
                case 'n':  out += '\n'; i += 2; continue;
                case 'r':  out += '\r'; i += 2; continue;
                case 't':  out += '\t'; i += 2; continue;
                case 'u':
                    if (i + 5 < raw.size()) {
                        unsigned int cp = 0;
                        if (sscanf(raw.substr(i + 2, 4).c_str(), "%x", &cp) == 1) {
                            if (cp < 0x80) {
                                out += static_cast<char>(cp);
                            } else if (cp < 0x800) {
                                out += static_cast<char>(0xC0 | (cp >> 6));
                                out += static_cast<char>(0x80 | (cp & 0x3F));
                            } else {
                                out += static_cast<char>(0xE0 | (cp >> 12));
                                out += static_cast<char>(0x80 | ((cp >> 6) & 0x3F));
                                out += static_cast<char>(0x80 | (cp & 0x3F));
                            }
                        }
                        i += 6;
                        continue;
                    }
                    break;
            }
        }
        out += c;
        i++;
    }
    end_quote = raw.size();
    return out;
}

// Extract all "text": "..." inside Gemini response and concatenate
inline std::string extract_gemini_text(const std::string &json) {
    // Check for API error message first
    size_t err_pos = json.find("\"error\"");
    if (err_pos != std::string::npos) {
        size_t msg_pos = json.find("\"message\"", err_pos);
        if (msg_pos != std::string::npos) {
            size_t q1 = json.find('"', msg_pos + 9);
            if (q1 != std::string::npos) {
                size_t q2 = 0;
                return "[ERRO API] " + unescape_json_str(json, q1, q2);
            }
        }
    }

    size_t cand_pos = json.find("\"candidates\"");
    if (cand_pos == std::string::npos) {
        return "[ERRO] Resposta inesperada da API: " + json.substr(0, 220);
    }

    size_t stop_pos = json.find("\"usageMetadata\"", cand_pos);
    if (stop_pos == std::string::npos) stop_pos = json.size();

    std::string combined;
    size_t search_from = cand_pos;
    while (search_from < stop_pos) {
        size_t text_pos = json.find("\"text\"", search_from);
        if (text_pos == std::string::npos || text_pos >= stop_pos) break;

        size_t colon = json.find(':', text_pos + 6);
        if (colon == std::string::npos || colon >= stop_pos) break;
        size_t q1 = json.find('"', colon + 1);
        if (q1 == std::string::npos || q1 >= stop_pos) break;
        size_t q2 = 0;
        std::string piece = unescape_json_str(json, q1, q2);

        // Skip internal reasoning parts if marked with "thought": true nearby
        size_t window_end = (q2 + 40 < stop_pos) ? (q2 + 40) : stop_pos;
        std::string tail = json.substr(q2, window_end - q2);
        if (tail.find("\"thought\"") == std::string::npos || tail.find("true") == std::string::npos) {
            combined += piece;
        }
        search_from = q2 + 1;
    }

    if (combined.empty()) {
        return "[ERRO] Nenhum texto retornado pelo modelo.";
    }
    return combined;
}

inline std::string extract_json_field(const std::string &json, const std::string &key) {
    std::string pat = "\"" + key + "\"";
    size_t p = json.find(pat);
    if (p == std::string::npos) return "";
    size_t colon = json.find(':', p + pat.size());
    if (colon == std::string::npos) return "";
    size_t i = colon + 1;
    while (i < json.size() && (json[i] == ' ' || json[i] == '\t' || json[i] == '\n' || json[i] == '\r')) i++;
    if (i >= json.size()) return "";
    if (json[i] == '"') {
        size_t q2 = 0;
        return unescape_json_str(json, i, q2);
    }
    size_t j = i;
    while (j < json.size() && json[j] != ',' && json[j] != '}' && json[j] != ' ') j++;
    return json.substr(i, j - i);
}

struct AttachmentPair {
    std::string name;
    std::string content;
};

inline std::vector<AttachmentPair> extract_attachments(const std::string &json) {
    std::vector<AttachmentPair> out;
    size_t ap = json.find("\"attachments\"");
    if (ap == std::string::npos) return out;
    size_t arr_start = json.find('[', ap);
    if (arr_start == std::string::npos) return out;

    size_t pos = arr_start + 1;
    while (pos < json.size() && out.size() < 8) {
        size_t obj_start = json.find('{', pos);
        size_t arr_end = json.find(']', pos);
        if (obj_start == std::string::npos || (arr_end != std::string::npos && arr_end < obj_start)) break;

        size_t np = json.find("\"name\"", obj_start);
        size_t cp = json.find("\"content\"", obj_start);
        if (np == std::string::npos || cp == std::string::npos) break;

        size_t nq1 = json.find('"', json.find(':', np + 6) + 1);
        size_t nq2 = 0;
        std::string name = (nq1 != std::string::npos) ? unescape_json_str(json, nq1, nq2) : "anexo.txt";

        size_t cq1 = json.find('"', json.find(':', cp + 9) + 1);
        size_t cq2 = 0;
        std::string content = (cq1 != std::string::npos) ? unescape_json_str(json, cq1, cq2) : "";

        if (!content.empty()) {
            out.push_back({name, content});
        }
        pos = (cq2 > nq2 ? cq2 : nq2) + 1;
    }
    return out;
}

} // namespace json_mini
