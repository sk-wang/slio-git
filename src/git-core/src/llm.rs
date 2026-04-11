//! LLM-powered commit message generation via OpenAI-compatible API.

use serde::{Deserialize, Serialize};

/// Configuration for an OpenAI-compatible LLM endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    pub api_url: String,
    pub api_key: String,
    pub model: String,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            api_url: "https://api.deepseek.com/v1/chat/completions".to_string(),
            api_key: String::new(),
            model: "deepseek-chat".to_string(),
        }
    }
}

impl LlmConfig {
    pub fn is_configured(&self) -> bool {
        !self.api_key.is_empty() && !self.api_url.is_empty() && !self.model.is_empty()
    }
}

fn truncate_utf8(input: &str, max_bytes: usize) -> &str {
    if input.len() <= max_bytes {
        return input;
    }

    let mut end = max_bytes;
    while !input.is_char_boundary(end) {
        end -= 1;
    }

    &input[..end]
}

#[derive(Serialize)]
struct ChatRequest<'a> {
    model: &'a str,
    messages: Vec<ChatMessage<'a>>,
    temperature: f32,
    max_tokens: u32,
}

#[derive(Serialize)]
struct ChatMessage<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: ResponseMessage,
}

#[derive(Deserialize)]
struct ResponseMessage {
    content: String,
}

/// Build a prompt from branch name, staged diff, and recent commit logs.
fn build_prompt(branch_name: &str, diff_summary: &str, recent_logs: &[String]) -> String {
    let mut prompt = String::from(
        "You are a commit message generator. Based on the current branch, staged changes (diff), \
         and recent commit history, write a concise, conventional commit message.\n\
         Rules:\n\
         - Use conventional commits format: type(scope): description\n\
         - First line under 72 characters\n\
         - Optionally add a blank line and body for complex changes\n\
         - The branch name hints at the feature/fix being worked on — use it for context\n\
         - Match the style and language of recent commits (Chinese or English)\n\
         - Output ONLY the commit message, no explanation\n\n",
    );

    prompt.push_str(&format!("Current branch: {}\n\n", branch_name));

    if !recent_logs.is_empty() {
        prompt.push_str("Recent commits (newest first):\n");
        for log in recent_logs.iter().take(15) {
            prompt.push_str("- ");
            prompt.push_str(log);
            prompt.push('\n');
        }
        prompt.push('\n');
    }

    prompt.push_str("Staged changes (diff):\n```\n");
    // Truncate large diffs to stay within token limits
    const MAX_DIFF_CHARS: usize = 8000;
    if diff_summary.len() > MAX_DIFF_CHARS {
        prompt.push_str(truncate_utf8(diff_summary, MAX_DIFF_CHARS));
        prompt.push_str("\n... (truncated)\n");
    } else {
        prompt.push_str(diff_summary);
    }
    prompt.push_str("```\n");

    prompt
}

/// Generate a commit message using an OpenAI-compatible API.
pub async fn generate_commit_message(
    config: &LlmConfig,
    branch_name: &str,
    diff_summary: &str,
    recent_logs: &[String],
) -> Result<String, String> {
    if !config.is_configured() {
        return Err("LLM 未配置，请在设置中填写 API 地址、密钥和模型。".to_string());
    }

    let prompt = build_prompt(branch_name, diff_summary, recent_logs);

    let request = ChatRequest {
        model: &config.model,
        messages: vec![
            ChatMessage {
                role: "system",
                content: "You are a helpful assistant that generates git commit messages.",
            },
            ChatMessage {
                role: "user",
                content: &prompt,
            },
        ],
        temperature: 0.3,
        max_tokens: 256,
    };

    let client = reqwest::Client::new();
    let response = client
        .post(&config.api_url)
        .header("Authorization", format!("Bearer {}", config.api_key))
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await
        .map_err(|e| format!("LLM 请求失败: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("LLM 返回错误 {}: {}", status, body));
    }

    let chat_response: ChatResponse = response
        .json()
        .await
        .map_err(|e| format!("LLM 响应解析失败: {}", e))?;

    chat_response
        .choices
        .into_iter()
        .next()
        .map(|c| c.message.content.trim().to_string())
        .ok_or_else(|| "LLM 返回空响应。".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_not_configured() {
        let config = LlmConfig::default();
        assert!(!config.is_configured());
    }

    #[test]
    fn configured_when_all_fields_set() {
        let config = LlmConfig {
            api_url: "https://api.example.com/v1/chat/completions".to_string(),
            api_key: "sk-test".to_string(),
            model: "gpt-4o-mini".to_string(),
        };
        assert!(config.is_configured());
    }

    #[test]
    fn build_prompt_includes_branch_diff_and_logs() {
        let diff = "diff --git a/foo.rs\n+fn bar() {}";
        let logs = vec!["feat: add foo".to_string(), "fix: bar bug".to_string()];
        let prompt = build_prompt("feature/login", diff, &logs);
        assert!(prompt.contains("feature/login"));
        assert!(prompt.contains("feat: add foo"));
        assert!(prompt.contains("+fn bar() {}"));
    }

    #[test]
    fn build_prompt_truncates_large_diff() {
        let diff = "x".repeat(10000);
        let prompt = build_prompt("main", &diff, &[]);
        assert!(prompt.contains("(truncated)"));
        assert!(prompt.contains("main"));
    }

    #[test]
    fn build_prompt_truncates_large_multibyte_diff_without_panicking() {
        let diff = "你好，世界\n".repeat(2000);
        let prompt = build_prompt("main", &diff, &[]);
        assert!(prompt.contains("(truncated)"));
        assert!(prompt.contains("你好"));
    }
}
