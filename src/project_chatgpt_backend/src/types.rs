use candid::{CandidType, Deserialize};
use serde::Serialize;
use std::collections::HashMap;

/// Struktura analizy wiadomości z wartościami, wagami i słowami kluczowymi
#[derive(Clone, CandidType, Deserialize, Serialize, Debug)]
pub struct ChatAnalysis {
    pub goal: String,
    pub sentence_type: String,
    pub content_summary: String,
    pub intent: String,
    pub mood: String,
    pub vision: String,
    pub representation: String,
    /// Wagi poszczególnych cech
    pub weights: HashMap<String, f32>,
    /// Słowa kluczowe wyciągnięte z wiadomości
    pub keywords: Vec<String>,
}

impl ChatAnalysis {
    pub fn new() -> Self {
        ChatAnalysis {
            goal: String::new(),
            sentence_type: String::new(),
            content_summary: String::new(),
            intent: String::new(),
            mood: String::new(),
            vision: String::new(),
            representation: String::new(),
            weights: HashMap::new(),
            keywords: Vec::new(),
        }
    }

    pub fn set_weight(&mut self, feature: &str, value: f32) {
        self.weights.insert(feature.to_string(), value);
    }

    pub fn get_weight(&self, feature: &str) -> f32 {
        *self.weights.get(feature).unwrap_or(&0.0)
    }

    pub fn set_keywords(&mut self, keys: Vec<String>) {
        self.keywords = keys;
    }
}