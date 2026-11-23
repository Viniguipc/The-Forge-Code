#pragma once
#include <string>
#include <vector>

struct RagDoc {
    std::string name;
    std::string content;
};

struct ToolResult {
    bool ran;
    std::string tool_name;
    std::string summary;   // compact text injected into agent prompt
    std::string ui_badge;  // short status shown in UI turn card
};

// Level 2: Lightweight token-efficient RAG
ToolResult rag_retrieve(const std::vector<RagDoc> &docs, const std::string &query,
                        int max_chunks = 3, size_t max_chars = 1400);

// Level 3: Zero-key Web Search (GitHub Repos + Wikipedia PT)
ToolResult web_search_competitors(const std::string &idea_text);

// Level 3: Isolated Code Compilation & Execution Sandbox (C, C++, JS, Python)
ToolResult sandbox_execute_code(const std::string &text_with_code);
