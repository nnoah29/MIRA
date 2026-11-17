#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Format Responses, outils : annonce, éléments d'appel et de résultat, fragments décodés.

mod ai_wire_support;

use ai_wire_support::tools::{search_tool, tool_turn};
use ai_wire_support::{descriptor, request};
use mira_core::ai::sse::SseEvent;
use mira_core::ai::wire::dispatch::{decode_event, encode};
use mira_vocabulary::{AiProviderDescriptor, ChatDelta, ChatRequest, MaxTokensField, WireFormat};
use serde_json::{Value, json};

/// Corps JSON encodé de la requête au format Responses.
fn body_of(request: &ChatRequest) -> Value {
    let descriptor = AiProviderDescriptor {
        format: WireFormat::OpenAiResponses,
        ..descriptor(MaxTokensField::MaxTokens)
    };
    serde_json::from_str(&encode(request, "m1", &descriptor).body).unwrap()
}

/// Les outils sont annoncés comme des fonctions à plat
#[test]
fn tools_are_flat_functions() {
    let mut req = request();
    req.tools = vec![search_tool()];
    let expected = json!([{"type": "function", "name": "search_messages",
        "description": "Cherche dans la boîte.",
        "parameters": {"type": "object", "properties": {"query": {"type": "string"}}, "required": ["query"]}}]);
    assert_eq!(body_of(&req)["tools"], expected);
}

/// Sans outil, le champ n'existe pas
#[test]
fn tools_absent_when_empty() {
    assert!(body_of(&request()).get("tools").is_none());
}

/// Les appels de l'assistant et leurs résultats deviennent des éléments `function_call` et `function_call_output`
#[test]
fn assistant_calls_and_results_become_items() {
    let input = body_of(&tool_turn("Je cherche"))["input"].clone();
    let items = input.as_array().unwrap();
    assert_eq!(items.len(), 6);
    assert_eq!(
        items[1],
        json!({"role": "assistant", "content": [{"type": "output_text", "text": "Je cherche"}]})
    );
    assert_eq!(
        items[2],
        json!({"type": "function_call", "call_id": "c1", "name": "search_messages", "arguments": "{\"query\":\"devis\"}"})
    );
    assert_eq!(items[3]["call_id"], "c2");
    assert_eq!(
        items[4],
        json!({"type": "function_call_output", "call_id": "c1", "output": "r1"})
    );
    assert_eq!(
        items[5],
        json!({"type": "function_call_output", "call_id": "c2", "output": "r2"})
    );
}

/// Décode des données SSE au format Responses.
fn decode(data: &str) -> Vec<ChatDelta> {
    let event = SseEvent {
        event: None,
        data: data.to_string(),
    };
    decode_event(WireFormat::OpenAiResponses, &event).unwrap()
}

/// L'ajout d'un élément `function_call` porte l'identifiant et le nom, les arguments suivent par morceaux
#[test]
fn function_call_fragments_decode() {
    let added = r#"{"type":"response.output_item.added","output_index":1,"item":{"type":"function_call","call_id":"c9","name":"search_messages","arguments":""}}"#;
    let first = ChatDelta::ToolFragment {
        index: 1,
        id: Some("c9".into()),
        name: Some("search_messages".into()),
        arguments: String::new(),
    };
    assert_eq!(decode(added), vec![first]);
    let more =
        r#"{"type":"response.function_call_arguments.delta","output_index":1,"delta":"{\"q\":"}"#;
    let next = ChatDelta::ToolFragment {
        index: 1,
        id: None,
        name: None,
        arguments: "{\"q\":".into(),
    };
    assert_eq!(decode(more), vec![next]);
    let message =
        r#"{"type":"response.output_item.added","output_index":0,"item":{"type":"message"}}"#;
    assert!(decode(message).is_empty());
}
