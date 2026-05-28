# Rust Candle Gateway

A dependency-light Rust inference gateway with a Candle-ready engine boundary.

The default implementation uses only the Rust standard library so the gateway shape is easy to inspect, compile, and explain before adding model weights, tokenizers, or GPU-specific dependencies.

## What It Does

- Serves `GET /healthz`.
- Serves Prometheus-style `GET /metrics`.
- Serves `POST /v1/completions`.
- Enforces bounded prompt length.
- Tracks request count, error count, generated tokens, and total latency.
- Separates HTTP handling from the `InferenceEngine` trait so a Candle engine can replace the mock engine cleanly.

## Run

```bash
cargo run -- --host 127.0.0.1 --port 8090
```

Request:

```bash
curl -s http://127.0.0.1:8090/v1/completions \
  -H 'content-type: application/json' \
  -d '{"prompt":"Explain KV cache in one sentence","max_tokens":32}'
```

Metrics:

```bash
curl -s http://127.0.0.1:8090/metrics
```

## Candle Integration Plan

The gateway intentionally separates `InferenceEngine` from HTTP handling. To make this a real Candle service:

1. Add optional Candle dependencies to `Cargo.toml`.
2. Implement `InferenceEngine` for a `CandleEngine` that owns model weights, tokenizer, device, and generation config.
3. Preserve the same HTTP contract, timeouts, bounded prompt checks, and metrics.
4. Benchmark CPU, Metal, or CUDA backends with the same request payload.

## Interview Narrative

- Rust is useful at the model-serving boundary because it gives predictable memory ownership, low runtime overhead, and good safety properties.
- A production model gateway is more than generation code: it needs request bounds, timeouts, health checks, metrics, and clear API contracts.
- The mock engine keeps the service runnable; the `InferenceEngine` trait shows exactly where Candle-backed generation belongs.

