use crate::context::Context;
use crate::helpers::colors::Colorize;
use log::debug;
use serde_json::{Value, json};

const DEFAULT_BASE_URL: &str = "https://api.openai.com";
const DEFAULT_MODEL: &str = "gpt-4o-mini";

fn get_env_or_config(env_var: &str, config_val: &str) -> Option<String> {
    std::env::var(env_var)
        .ok()
        .filter(|v| !v.is_empty())
        .or_else(|| if config_val.is_empty() { None } else { Some(config_val.to_string()) })
}

fn build_repo_list(c: &Context) -> String {
    let repos = c.database().get_all_items();
    repos
        .iter()
        .map(|r| format!("{}/{}/{}", r.host, r.owner, r.repo))
        .collect::<Vec<_>>()
        .join("\n")
}

fn call_chat_api(
    base_url: &str,
    api_key: &str,
    model: &str,
    messages: &[Value],
) -> anyhow::Result<String> {
    let url = format!("{}/v1/chat/completions", base_url.trim_end_matches('/'));

    let body = json!({
        "model": model,
        "messages": messages,
    });

    debug!("AI request URL: {}", url);
    debug!("AI request model: {}", model);

    let client = reqwest::blocking::Client::new();
    let response = client
        .post(&url)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&body)
        .send()?;

    let status = response.status();
    let response_text = response.text()?;

    if !status.is_success() {
        anyhow::bail!("API request failed ({}): {}", status, response_text);
    }

    let response_json: Value = serde_json::from_str(&response_text)?;

    let content = response_json
        .get("choices")
        .and_then(|c| c.get(0))
        .and_then(|c| c.get("message"))
        .and_then(|m| m.get("content"))
        .and_then(|c| c.as_str())
        .unwrap_or("")
        .to_string();

    Ok(content)
}

/// Use AI to resolve a natural language intent into a repository search keyword.
///
/// Given a user query like "I want to work on the blog" or "open the auth service",
/// this function calls an LLM with the list of managed repositories and asks it
/// to return the best matching keyword for the `find` command.
///
/// Returns `None` if AI is not configured or the API call fails.
pub fn resolve_intent(c: &Context, intent: &str) -> Option<String> {
    let ai_config = &c.config().ai;

    let api_key = match get_env_or_config("PROG_AI_API_KEY", &ai_config.api_key) {
        Some(key) => key,
        None => {
            eprintln!(
                "{}",
                "AI API key not configured. Set PROG_AI_API_KEY environment variable or add to config:"
                    .red()
            );
            eprintln!();
            eprintln!("  [ai]");
            eprintln!("  api_key = \"your-api-key\"");
            eprintln!("  model = \"gpt-4o-mini\"              # optional");
            eprintln!("  base_url = \"https://api.openai.com\" # optional");
            return None;
        }
    };

    let base_url = get_env_or_config("PROG_AI_BASE_URL", &ai_config.base_url)
        .unwrap_or_else(|| DEFAULT_BASE_URL.to_string());
    let model = get_env_or_config("PROG_AI_MODEL", &ai_config.model)
        .unwrap_or_else(|| DEFAULT_MODEL.to_string());

    let repo_list = build_repo_list(c);

    if repo_list.is_empty() {
        eprintln!("{}", "No repositories found. Run 'prog sync' first.".red());
        return None;
    }

    let messages = vec![
        json!({
            "role": "system",
            "content": format!(
                "You are a repository search assistant. The user manages the following repositories:\n\n\
                 {}\n\n\
                 Given a natural language query, determine which repository the user wants to navigate to. \
                 Reply with ONLY the repository name (the last component of the path, e.g. \"prog\" not \"github.com/bytemain/prog\"). \
                 If multiple repositories match, return them separated by commas. \
                 Do NOT include any explanation, just the repository name(s).",
                repo_list
            )
        }),
        json!({
            "role": "user",
            "content": intent
        }),
    ];

    match call_chat_api(&base_url, &api_key, &model, &messages) {
        Ok(response) => {
            let keyword = response.trim().to_string();
            debug!("AI resolved intent '{}' to keyword '{}'", intent, keyword);
            if keyword.is_empty() { None } else { Some(keyword) }
        }
        Err(e) => {
            eprintln!("{}", format!("AI query failed: {}", e).red());
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_env_or_config_falls_back_to_config() {
        let result = get_env_or_config("PROG_AI_TEST_NONEXISTENT_KEY", "config_value");
        assert_eq!(result, Some("config_value".to_string()));
    }

    #[test]
    fn test_get_env_or_config_both_empty_returns_none() {
        let result = get_env_or_config("PROG_AI_TEST_NONEXISTENT_KEY_2", "");
        assert_eq!(result, None);
    }

    #[test]
    fn test_default_constants() {
        assert_eq!(DEFAULT_BASE_URL, "https://api.openai.com");
        assert_eq!(DEFAULT_MODEL, "gpt-4o-mini");
    }
}
