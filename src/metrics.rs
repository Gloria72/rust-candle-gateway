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

#[cfg(test)]
mod tests {
    use super::Metrics;

    #[test]
    fn metrics_start_at_zero() {
        let metrics = Metrics::new();
        let rendered = metrics.render_prometheus();

        assert!(rendered.contains("rust_candle_gateway_requests_total 0"));
        assert!(rendered.contains("rust_candle_gateway_errors_total 0"));
        assert!(rendered.contains("rust_candle_gateway_generated_tokens_total 0"));
        assert!(rendered.contains("rust_candle_gateway_latency_ms_total 0"));
    }

    #[test]
    fn metrics_record_success_and_error() {
        let metrics = Metrics::new();

        metrics.record_success(7, 13);
        metrics.record_error();

        let rendered = metrics.render_prometheus();
        assert!(rendered.contains("rust_candle_gateway_requests_total 2"));
        assert!(rendered.contains("rust_candle_gateway_errors_total 1"));
        assert!(rendered.contains("rust_candle_gateway_generated_tokens_total 7"));
        assert!(rendered.contains("rust_candle_gateway_latency_ms_total 13"));
    }
}
