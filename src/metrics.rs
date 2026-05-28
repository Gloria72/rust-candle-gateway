use std::sync::atomic::{AtomicU64, Ordering};

pub struct Metrics {
    requests: AtomicU64,
    errors: AtomicU64,
    generated_tokens: AtomicU64,
    total_latency_ms: AtomicU64,
}

impl Metrics {
    pub fn new() -> Self {
        Self {
            requests: AtomicU64::new(0),
            errors: AtomicU64::new(0),
            generated_tokens: AtomicU64::new(0),
            total_latency_ms: AtomicU64::new(0),
        }
    }

    pub fn record_success(&self, generated_tokens: u64, latency_ms: u64) {
        self.requests.fetch_add(1, Ordering::Relaxed);
        self.generated_tokens
            .fetch_add(generated_tokens, Ordering::Relaxed);
        self.total_latency_ms
            .fetch_add(latency_ms, Ordering::Relaxed);
    }

    pub fn record_error(&self) {
        self.requests.fetch_add(1, Ordering::Relaxed);
        self.errors.fetch_add(1, Ordering::Relaxed);
    }

    pub fn render_prometheus(&self) -> String {
        let requests = self.requests.load(Ordering::Relaxed);
        let errors = self.errors.load(Ordering::Relaxed);
        let tokens = self.generated_tokens.load(Ordering::Relaxed);
        let latency = self.total_latency_ms.load(Ordering::Relaxed);
        format!(
            "rust_candle_gateway_requests_total {}\n\
             rust_candle_gateway_errors_total {}\n\
             rust_candle_gateway_generated_tokens_total {}\n\
             rust_candle_gateway_latency_ms_total {}\n",
            requests, errors, tokens, latency
        )
    }
}

