//! Flux de test au format Responses.

use mira_testkit::ai::Behavior;
use mira_vocabulary::ChatDelta;

/// Flux Responses : deux morceaux de texte puis la clôture avec la consommation
pub fn responses_stream() -> Behavior {
    let events = [
        r#"{"type":"response.output_text.delta","output_index":0,"delta":"Bon"}"#,
        r#"{"type":"response.output_text.delta","output_index":0,"delta":"jour"}"#,
        r#"{"type":"response.completed","response":{"output":[{"type":"message"}],"usage":{"input_tokens":12,"output_tokens":5}}}"#,
    ];
    let chunks = events
        .iter()
        .map(|data| format!("data: {data}\n\n").into_bytes())
        .collect();
    Behavior::Stream {
        chunks,
        chunk_delay_ms: 0,
    }
}

/// Texte reconstitué des deltas
pub fn text_of(deltas: &[ChatDelta]) -> String {
    deltas
        .iter()
        .filter_map(|delta| match delta {
            ChatDelta::Text { text } => Some(text.as_str()),
            _ => None,
        })
        .collect()
}
