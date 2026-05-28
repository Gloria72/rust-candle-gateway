use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use crate::engine::{CompletionRequest, InferenceEngine};
use crate::metrics::Metrics;

pub struct GatewayConfig {
    pub host: String,
    pub port: u16,
    pub timeout: Duration,
    pub max_prompt_chars: usize,
}

pub struct GatewayState {
    pub engine: Arc<dyn InferenceEngine>,
    pub metrics: Arc<Metrics>,
    pub config: GatewayConfig,
}

pub fn serve(listener: TcpListener, state: Arc<GatewayState>) -> std::io::Result<()> {
    for stream in listener.incoming() {
        let stream = stream?;
        let state = Arc::clone(&state);
        thread::spawn(move || {
            if let Err(error) = handle_stream(stream, state) {
                eprintln!("request error: {}", error);
            }
        });
    }
    Ok(())
}

fn handle_stream(mut stream: TcpStream, state: Arc<GatewayState>) -> std::io::Result<()> {
    stream.set_read_timeout(Some(state.config.timeout))?;
    stream.set_write_timeout(Some(state.config.timeout))?;
    let mut buffer = vec![0_u8; 64 * 1024];
    let bytes = stream.read(&mut buffer)?;
    let request = String::from_utf8_lossy(&buffer[..bytes]).to_string();
    let path = parse_path(&request);

    match (parse_method(&request).as_str(), path.as_str()) {
        ("GET", "/healthz") => write_json(&mut stream, 200, "{\"ok\":true}"),
        ("GET", "/metrics") => write_text(&mut stream, 200, &state.metrics.render_prometheus()),
        ("POST", "/v1/completions") => handle_completion(&mut stream, request, state),
        _ => write_json(&mut stream, 404, "{\"error\":\"not found\"}"),
    }
}

fn handle_completion(
    stream: &mut TcpStream,
    raw_request: String,
    state: Arc<GatewayState>,
) -> std::io::Result<()> {
    let started = Instant::now();
    let body = raw_request.split("\r\n\r\n").nth(1).unwrap_or("");
    let prompt = extract_json_string(body, "prompt").unwrap_or_default();
    let max_tokens = extract_json_number(body, "max_tokens").unwrap_or(64);

    if prompt.len() > state.config.max_prompt_chars {
        state.metrics.record_error();
        return write_json(stream, 413, "{\"error\":\"prompt too large\"}");
    }

    let response = state.engine.complete(CompletionRequest { prompt, max_tokens });
    let latency_ms = started.elapsed().as_millis() as u64;
    state
        .metrics
        .record_success(response.completion_tokens as u64, latency_ms);

    let payload = format!(
        "{{\"model\":\"{}\",\"choices\":[{{\"text\":\"{}\",\"finish_reason\":\"stop\"}}],\
         \"usage\":{{\"prompt_tokens\":{},\"completion_tokens\":{},\"total_tokens\":{}}},\
         \"latency_ms\":{}}}",
        escape_json(&response.model),
        escape_json(&response.text),
        response.prompt_tokens,
        response.completion_tokens,
        response.prompt_tokens + response.completion_tokens,
        response.latency.as_millis()
    );
    write_json(stream, 200, &payload)
}

fn parse_method(request: &str) -> String {
    request
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().next())
        .unwrap_or("")
        .to_string()
}

fn parse_path(request: &str) -> String {
    request
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .unwrap_or("/")
        .to_string()
}

fn extract_json_string(body: &str, key: &str) -> Option<String> {
    let needle = format!("\"{}\"", key);
    let start = body.find(&needle)?;
    let after_key = &body[start + needle.len()..];
    let colon = after_key.find(':')?;
    let mut chars = after_key[colon + 1..].trim_start().chars();
    if chars.next()? != '"' {
        return None;
    }
    let mut value = String::new();
    let mut escaped = false;
    for ch in chars {
        if escaped {
            value.push(ch);
            escaped = false;
        } else if ch == '\\' {
            escaped = true;
        } else if ch == '"' {
            return Some(value);
        } else {
            value.push(ch);
        }
    }
    None
}

fn extract_json_number(body: &str, key: &str) -> Option<usize> {
    let needle = format!("\"{}\"", key);
    let start = body.find(&needle)?;
    let after_key = &body[start + needle.len()..];
    let colon = after_key.find(':')?;
    let digits = after_key[colon + 1..]
        .trim_start()
        .chars()
        .take_while(|ch| ch.is_ascii_digit())
        .collect::<String>();
    digits.parse().ok()
}

fn write_json(stream: &mut TcpStream, status: u16, body: &str) -> std::io::Result<()> {
    write_response(stream, status, "application/json", body)
}

fn write_text(stream: &mut TcpStream, status: u16, body: &str) -> std::io::Result<()> {
    write_response(stream, status, "text/plain; charset=utf-8", body)
}

fn write_response(
    stream: &mut TcpStream,
    status: u16,
    content_type: &str,
    body: &str,
) -> std::io::Result<()> {
    let reason = match status {
        200 => "OK",
        404 => "Not Found",
        413 => "Payload Too Large",
        _ => "OK",
    };
    let response = format!(
        "HTTP/1.1 {} {}\r\ncontent-type: {}\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
        status,
        reason,
        content_type,
        body.len(),
        body
    );
    stream.write_all(response.as_bytes())
}

fn escape_json(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}

#[cfg(test)]
mod tests {
    use super::{
        escape_json, extract_json_number, extract_json_string, parse_method, parse_path,
    };

    #[test]
    fn parses_method_and_path() {
        let raw = "POST /v1/completions HTTP/1.1\r\nhost: localhost\r\n\r\n{}";

        assert_eq!(parse_method(raw), "POST");
        assert_eq!(parse_path(raw), "/v1/completions");
    }

    #[test]
    fn parses_missing_request_line_safely() {
        assert_eq!(parse_method(""), "");
        assert_eq!(parse_path(""), "/");
    }

    #[test]
    fn extracts_basic_json_fields() {
        let body = r#"{"prompt":"Explain KV cache","max_tokens":32}"#;

        assert_eq!(
            extract_json_string(body, "prompt"),
            Some("Explain KV cache".to_string())
        );
        assert_eq!(extract_json_number(body, "max_tokens"), Some(32));
    }

    #[test]
    fn extracts_escaped_json_string() {
        let body = r#"{"prompt":"say \"hello\" now"}"#;

        assert_eq!(
            extract_json_string(body, "prompt"),
            Some("say \"hello\" now".to_string())
        );
    }

    #[test]
    fn escapes_json_response_text() {
        assert_eq!(
            escape_json("a \"quote\" and \\ slash\n"),
            "a \\\"quote\\\" and \\\\ slash\\n"
        );
    }
}
