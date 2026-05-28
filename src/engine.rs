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

#[cfg(test)]
mod tests {
    use super::{rough_token_count, CompletionRequest, InferenceEngine, MockEngine};

    #[test]
    fn rough_token_count_ignores_extra_whitespace() {
        assert_eq!(rough_token_count("  one   two\nthree\t"), 3);
        assert_eq!(rough_token_count(""), 0);
    }

    #[test]
    fn mock_engine_respects_max_tokens() {
        let engine = MockEngine::new("unit-model");
        let response = engine.complete(CompletionRequest {
            prompt: "hello kv cache".to_string(),
            max_tokens: 4,
        });

        assert_eq!(response.model, "unit-model");
        assert_eq!(response.prompt_tokens, 3);
        assert_eq!(response.completion_tokens, 4);
        assert_eq!(rough_token_count(&response.text), 4);
    }

    #[test]
    fn mock_engine_uses_at_least_one_output_token() {
        let engine = MockEngine::new("unit-model");
        let response = engine.complete(CompletionRequest {
            prompt: "hello".to_string(),
            max_tokens: 0,
        });

        assert_eq!(response.completion_tokens, 1);
        assert!(!response.text.is_empty());
    }
}
