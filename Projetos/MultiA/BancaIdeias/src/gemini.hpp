#pragma once
#include <string>

struct LlmResponse {
    bool ok;
    int status_code;
    std::string text;
};

LlmResponse call_gemini_api(
    const std::string &api_key,
    const std::string &model,
    const std::string &system_prompt,
    const std::string &user_prompt
);
