# Rust Candle Gateway

A small Rust HTTP gateway shaped like an inference service.

The default engine is a mock implementation. That is intentional: the HTTP surface, request bounds, metrics, and engine trait can be checked before pulling in Candle, tokenizers, model weights, or GPU-specific setup.

## What is here

- `GET /healthz`
- `GET /metrics` in a simple Prometheus-style text format
- `POST /v1/completions`
- Enforces bounded prompt length.
- Tracks request count, error count, generated tokens, and total latency.
- Keeps HTTP handling separate from the `InferenceEngine` trait, so the mock engine can be replaced by a Candle-backed engine later.

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

## Test

```bash
cargo test
cargo check
```

The current unit tests cover the mock engine token limits, rough token counting, metrics rendering, and the small HTTP parsing/JSON escaping helpers. They do not start a socket server yet, so they run quickly and do not need a model download.

## Candle Integration Plan

To make this a real Candle service:

1. Add optional Candle dependencies to `Cargo.toml`.
2. Implement `InferenceEngine` for a `CandleEngine` that owns model weights, tokenizer, device, and generation config.
3. Preserve the same HTTP contract, timeouts, bounded prompt checks, and metrics.
4. Benchmark CPU, Metal, or CUDA backends with the same request payload.

## Current status

- This repo does not include model weights or a tokenizer yet.
- The mock engine is there to keep the service runnable while the gateway code is being shaped.
- The next real step is adding a `CandleEngine` behind the existing trait and then comparing CPU/Metal/CUDA runs with the same request payloads.
