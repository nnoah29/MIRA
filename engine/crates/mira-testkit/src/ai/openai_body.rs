//! Flux SSE au format OpenAI Chat : deltas, arrêt, usage facultatif puis `[DONE]`.

use serde_json::{Value, json};

use super::behavior::Behavior;

/// Construit un flux OpenAI Chat ; `usage` vaut `(jetons d'entrée, jetons de sortie)`
pub fn openai_stream(texts: &[&str], usage: Option<(u32, u32)>) -> Behavior {
    let mut chunks: Vec<Vec<u8>> = texts.iter().map(|text| data_event(&delta(text))).collect();
    chunks.push(data_event(&finish()));
    if let Some((prompt, completion)) = usage {
        chunks.push(data_event(&usage_chunk(prompt, completion)));
    }
    chunks.push(b"data: [DONE]\n\n".to_vec());
    Behavior::Stream {
        chunks,
        chunk_delay_ms: 0,
    }
}

/// Événement SSE à une ligne `data:`
fn data_event(value: &Value) -> Vec<u8> {
    format!("data: {value}\n\n").into_bytes()
}

/// Morceau portant un fragment de texte
fn delta(text: &str) -> Value {
    json!({"choices": [{"delta": {"content": text}}]})
}

/// Morceau annonçant la fin normale de la génération
fn finish() -> Value {
    json!({"choices": [{"delta": {}, "finish_reason": "stop"}]})
}

/// Morceau d'usage final, sans choix
fn usage_chunk(prompt: u32, completion: u32) -> Value {
    json!({"choices": [], "usage": {"prompt_tokens": prompt, "completion_tokens": completion}})
}
