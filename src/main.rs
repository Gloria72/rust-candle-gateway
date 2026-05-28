mod engine;
mod http;
mod metrics;

use std::env;
use std::net::TcpListener;
use std::sync::Arc;
use std::time::Duration;

use engine::MockEngine;
use http::{serve, GatewayConfig, GatewayState};
use metrics::Metrics;

fn main() -> std::io::Result<()> {
    let config = parse_args(env::args().collect());
    let listener = TcpListener::bind(format!("{}:{}", config.host, config.port))?;
    eprintln!(
        "rust candle gateway listening on http://{}:{}",
        config.host, config.port
    );

    let state = Arc::new(GatewayState {
        engine: Arc::new(MockEngine::new("mock-candle")),
        metrics: Arc::new(Metrics::new()),
        config,
    });
    serve(listener, state)
}

fn parse_args(args: Vec<String>) -> GatewayConfig {
    let mut host = String::from("127.0.0.1");
    let mut port = 8090_u16;
    let mut timeout_ms = 30_000_u64;
    let mut max_prompt_chars = 8_192_usize;

    let mut index = 1;
    while index < args.len() {
        match args[index].as_str() {
            "--host" if index + 1 < args.len() => {
                host = args[index + 1].clone();
                index += 2;
            }
            "--port" if index + 1 < args.len() => {
                port = args[index + 1].parse().unwrap_or(port);
                index += 2;
            }
            "--timeout-ms" if index + 1 < args.len() => {
                timeout_ms = args[index + 1].parse().unwrap_or(timeout_ms);
                index += 2;
            }
            "--max-prompt-chars" if index + 1 < args.len() => {
                max_prompt_chars = args[index + 1].parse().unwrap_or(max_prompt_chars);
                index += 2;
            }
            _ => {
                index += 1;
            }
        }
    }

    GatewayConfig {
        host,
        port,
        timeout: Duration::from_millis(timeout_ms),
        max_prompt_chars,
    }
}

