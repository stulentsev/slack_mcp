use anyhow::{anyhow, Context, Result};
use chrono::DateTime;
use regex::Regex;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::env;
use std::io::{self, BufRead, Write};
use std::path::PathBuf;

/// MCP JSON-RPC request structure
#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    #[allow(dead_code)]
    jsonrpc: String,
    id: Option<Value>,
    method: String,
    params: Option<Value>,
}

/// MCP JSON-RPC response structure
#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

/// JSON-RPC error structure
#[derive(Debug, Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
}

/// Slack thread URL components
#[derive(Debug)]
struct ThreadUrl {
    channel_id: String,
    thread_ts: String,
}

/// Slack API message structure
#[derive(Debug, Deserialize)]
struct SlackMessage {
    #[serde(rename = "type")]
    #[allow(dead_code)]
    msg_type: String,
    user: Option<String>,
    text: Option<String>,
    ts: String,
    #[allow(dead_code)]
    thread_ts: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    bot_id: Option<String>,
}

/// Slack API conversations.replies response
#[derive(Debug, Deserialize)]
struct ConversationsRepliesResponse {
    ok: bool,
    messages: Option<Vec<SlackMessage>>,
    error: Option<String>,
    has_more: Option<bool>,
    response_metadata: Option<ResponseMetadata>,
}

#[derive(Debug, Deserialize)]
struct ResponseMetadata {
    next_cursor: Option<String>,
}

/// Parse Slack thread URL to extract channel ID and thread timestamp
fn parse_slack_url(url: &str) -> Result<ThreadUrl> {
    // Pattern: https://{workspace}.slack.com/archives/{channel_id}/p{thread_ts}
    let re = Regex::new(r"https://[^/]+\.slack\.com/archives/([A-Z0-9]+)/p(\d+)")
        .context("Failed to compile regex")?;

    let captures = re
        .captures(url)
        .ok_or_else(|| anyhow!("Invalid Slack URL format"))?;

    let channel_id = captures
        .get(1)
        .ok_or_else(|| anyhow!("Channel ID not found"))?
        .as_str()
        .to_string();

    let thread_ts_raw = captures
        .get(2)
        .ok_or_else(|| anyhow!("Thread timestamp not found"))?
        .as_str();

    // Convert p1762872519417859 to 1762872519.417859
    let thread_ts = format!(
        "{}.{}",
        &thread_ts_raw[..10],
        &thread_ts_raw[10..]
    );

    Ok(ThreadUrl {
        channel_id,
        thread_ts,
    })
}

/// Load Slack token from environment or config file
fn load_slack_token() -> Result<String> {
    // First try environment variable
    if let Ok(token) = env::var("SLACK_TOKEN") {
        return Ok(token);
    }

    // Try loading from ~/.slackmcp.config
    let home_dir = env::var("HOME")
        .or_else(|_| env::var("USERPROFILE"))
        .context("Cannot determine home directory")?;

    let config_path = PathBuf::from(home_dir).join(".slackmcp.config");

    if config_path.exists() {
        dotenv::from_path(&config_path).ok();
        if let Ok(token) = env::var("SLACK_TOKEN") {
            return Ok(token);
        }
    }

    Err(anyhow!(
        "SLACK_TOKEN not found. Set it as an environment variable or in ~/.slackmcp.config"
    ))
}

/// Fetch thread messages from Slack API
async fn fetch_slack_thread(token: &str, channel_id: &str, thread_ts: &str) -> Result<Vec<SlackMessage>> {
    let client = Client::new();
    let mut all_messages = Vec::new();
    let mut cursor: Option<String> = None;

    loop {
        let mut url = format!(
            "https://slack.com/api/conversations.replies?channel={}&ts={}",
            channel_id, thread_ts
        );

        if let Some(c) = &cursor {
            url.push_str(&format!("&cursor={}", c));
        }

        let response = client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .send()
            .await
            .context("Failed to send request to Slack API")?;

        let reply: ConversationsRepliesResponse = response
            .json()
            .await
            .context("Failed to parse Slack API response")?;

        if !reply.ok {
            let error_msg = reply
                .error
                .unwrap_or_else(|| "Unknown error".to_string());
            return Err(anyhow!("Slack API error: {}", error_msg));
        }

        if let Some(mut messages) = reply.messages {
            all_messages.append(&mut messages);
        }

        // Check if there are more messages
        if let Some(true) = reply.has_more
            && let Some(metadata) = reply.response_metadata
            && let Some(next) = metadata.next_cursor
            && !next.is_empty()
        {
            cursor = Some(next);
            continue;
        }

        break;
    }

    Ok(all_messages)
}

/// Format timestamp for display
fn format_timestamp(ts: &str) -> String {
    // Parse timestamp (format: "1234567890.123456")
    if let Some(ts_str) = ts.split('.').next()
        && let Ok(timestamp) = ts_str.parse::<i64>()
        && let Some(dt) = DateTime::from_timestamp(timestamp, 0)
    {
        return dt.format("%Y-%m-%d %H:%M:%S").to_string();
    }
    ts.to_string()
}

/// Format thread messages for output
fn format_thread_messages(messages: Vec<SlackMessage>) -> String {
    let mut output = String::new();

    for (idx, msg) in messages.iter().enumerate() {
        if idx == 0 {
            // Parent message
            let user = msg.user.as_deref().unwrap_or("Unknown");
            let text = msg.text.as_deref().unwrap_or("");
            let timestamp = format_timestamp(&msg.ts);
            output.push_str(&format!("[{}] @{}\n{}\n\n", timestamp, user, text));
        } else {
            // Thread replies (indented)
            let user = msg.user.as_deref().unwrap_or("Unknown");
            let text = msg.text.as_deref().unwrap_or("");
            let timestamp = format_timestamp(&msg.ts);
            output.push_str(&format!("    [{}] @{}\n    {}\n\n", timestamp, user, text));
        }
    }

    output
}

/// Handle the read_thread tool call
async fn handle_read_thread(params: &Value) -> Result<String> {
    let url = params
        .get("url")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Missing or invalid 'url' parameter"))?;

    // Parse the URL
    let thread_url = parse_slack_url(url)?;

    // Load token
    let token = load_slack_token()?;

    // Fetch thread messages
    let messages = fetch_slack_thread(&token, &thread_url.channel_id, &thread_url.thread_ts).await?;

    if messages.is_empty() {
        return Ok("No messages found in thread.".to_string());
    }

    // Format and return
    Ok(format_thread_messages(messages))
}

/// Handle MCP initialize request
fn handle_initialize(id: Option<Value>) -> JsonRpcResponse {
    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id,
        result: Some(json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {}
            },
            "serverInfo": {
                "name": "slack-mcp-server",
                "version": "0.1.0"
            }
        })),
        error: None,
    }
}

/// Handle MCP tools/list request
fn handle_tools_list(id: Option<Value>) -> JsonRpcResponse {
    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id,
        result: Some(json!({
            "tools": [
                {
                    "name": "read_thread",
                    "description": "Retrieves a complete Slack thread conversation including the original message and all replies",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "url": {
                                "type": "string",
                                "description": "Full Slack thread URL (e.g., https://workspace.slack.com/archives/C0982014L82/p1762872519417859)"
                            }
                        },
                        "required": ["url"]
                    }
                }
            ]
        })),
        error: None,
    }
}

/// Handle MCP tools/call request
async fn handle_tools_call(id: Option<Value>, params: Option<Value>) -> JsonRpcResponse {
    let params = match params {
        Some(p) => p,
        None => {
            return JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: None,
                error: Some(JsonRpcError {
                    code: -32602,
                    message: "Missing parameters".to_string(),
                    data: None,
                }),
            };
        }
    };

    let tool_name = match params.get("name").and_then(|v| v.as_str()) {
        Some(name) => name,
        None => {
            return JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: None,
                error: Some(JsonRpcError {
                    code: -32602,
                    message: "Missing tool name".to_string(),
                    data: None,
                }),
            };
        }
    };

    if tool_name != "read_thread" {
        return JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(JsonRpcError {
                code: -32601,
                message: format!("Unknown tool: {}", tool_name),
                data: None,
            }),
        };
    }

    let tool_params = params.get("arguments").cloned().unwrap_or(json!({}));

    match handle_read_thread(&tool_params).await {
        Ok(content) => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(json!({
                "content": [
                    {
                        "type": "text",
                        "text": content
                    }
                ]
            })),
            error: None,
        },
        Err(e) => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(JsonRpcError {
                code: -32000,
                message: format!("Tool execution failed: {}", e),
                data: None,
            }),
        },
    }
}

/// Process a single JSON-RPC request
async fn process_request(request: JsonRpcRequest) -> JsonRpcResponse {
    match request.method.as_str() {
        "initialize" => handle_initialize(request.id),
        "tools/list" => handle_tools_list(request.id),
        "tools/call" => handle_tools_call(request.id, request.params).await,
        "ping" => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: Some(json!({})),
            error: None,
        },
        _ => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: None,
            error: Some(JsonRpcError {
                code: -32601,
                message: format!("Method not found: {}", request.method),
                data: None,
            }),
        },
    }
}

/// Main server loop - reads JSON-RPC requests from stdin and writes responses to stdout
#[tokio::main]
async fn main() -> Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let reader = stdin.lock();

    // Write server start message to stderr (so it doesn't interfere with JSON-RPC)
    eprintln!("Slack MCP Server started");

    for line in reader.lines() {
        let line = line.context("Failed to read line from stdin")?;

        if line.trim().is_empty() {
            continue;
        }

        // Parse JSON-RPC request
        let request: JsonRpcRequest = match serde_json::from_str(&line) {
            Ok(req) => req,
            Err(e) => {
                eprintln!("Failed to parse request: {}", e);
                let error_response = JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: None,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32700,
                        message: format!("Parse error: {}", e),
                        data: None,
                    }),
                };
                let response_json = serde_json::to_string(&error_response)?;
                writeln!(stdout, "{}", response_json)?;
                stdout.flush()?;
                continue;
            }
        };

        // Process request
        let response = process_request(request).await;

        // Write response
        let response_json = serde_json::to_string(&response)?;
        writeln!(stdout, "{}", response_json)?;
        stdout.flush()?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_slack_url() {
        let url = "https://alvys.slack.com/archives/C0982014L82/p1762872519417859";
        let result = parse_slack_url(url).unwrap();
        assert_eq!(result.channel_id, "C0982014L82");
        assert_eq!(result.thread_ts, "1762872519.417859");
    }

    #[test]
    fn test_parse_invalid_slack_url() {
        let url = "https://invalid.com/something";
        assert!(parse_slack_url(url).is_err());
    }

    #[test]
    fn test_format_timestamp() {
        let ts = "1234567890.123456";
        let formatted = format_timestamp(ts);
        assert!(formatted.contains("2009-02-13"));
    }
}
