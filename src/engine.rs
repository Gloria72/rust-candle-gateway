use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct CompletionRequest {
    pub prompt: String,
    pub max_tokens: usize,
}

#[derive(Debug, Clone)]
pub struct CompletionResponse {
    pub model: String,
    pub text: String,
    pub prompt_tokens: usize,
    pub completion_tokens: usize,
    pub latency: Duration,
}

pub trait InferenceEngine: Send + Sync {
    fn complete(&self, request: CompletionRequest) -> CompletionResponse;
}

pub struct MockEngine {
    model: String,
}

impl MockEngine {
    pub fn new(model: &str) -> Self {
        Self {
            model: model.to_string(),
        }
    }
}

impl InferenceEngine for MockEngine {
    fn complete(&self, request: CompletionRequest) -> CompletionResponse {
        let started = Instant::now();
        let prompt_tokens = rough_token_count(&request.prompt);
        let mut words = vec![
            "mock",
            "candle",
            "gateway",
            "response",
            "with",
            "bounded",
            "generation",
            "and",
            "metrics",
        ];
        if request.prompt.to_lowercase().contains("kv") {
            words.extend(["kv", "cache", "reuses", "past", "attention", "states"]);
        }
        let max_tokens = request.max_tokens.max(1);
        let text = words
            .into_iter()
            .take(max_tokens)
            .collect::<Vec<&str>>()
            .join(" ");
        CompletionResponse {
            model: self.model.clone(),
            prompt_tokens,
            completion_tokens: rough_token_count(&text),
            text,
            latency: started.elapsed(),
        }
    }
}

pub fn rough_token_count(text: &str) -> usize {
    text.split_whitespace().count()
}

