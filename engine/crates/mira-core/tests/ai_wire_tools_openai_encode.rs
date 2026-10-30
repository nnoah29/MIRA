#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Format OpenAI Chat, encodage des outils : annonce, appels de l'assistant, messages de résultat.

mod ai_wire_support;

use ai_wire_support::tools::{assistant_with, call, search_tool, tool_turn};
use ai_wire_support::{descriptor, request};
use mira_core::ai::wire::openai_encode::encode_chat;
use mira_vocabulary::{ChatRequest, MaxTokensField};
use serde_json::{Value, json};

/// Corps JSON encodé de la requête.
fn body_of(request: &ChatRequest) -> Value {
    let descriptor = descriptor(MaxTokensField::MaxTokens);
    serde_json::from_str(&encode_chat(request, "m1", &descriptor).body).unwrap()
}

/// Les outils sont annoncés comme des fonctions avec leurs paramètres
#[test]
fn tools_are_announced_as_functions() {
    let mut req = request();
    req.tools = vec![search_tool()];
    let expected = json!([{"type": "function", "function": {
        "name": "search_messages",
        "description": "Cherche dans la boîte.",
        "parameters": {"type": "object", "properties": {"query": {"type": "string"}}, "required": ["query"]}
    }}]);
    assert_eq!(body_of(&req)["tools"], expected);
}

/// Sans outil, le champ n'existe pas
#[test]
fn no_tools_field_when_list_is_empty() {
    assert!(body_of(&request()).get("tools").is_none());
}

/// Les appels de l'assistant portent leurs arguments en chaîne JSON
#[test]
fn assistant_tool_calls_carry_argument_strings() {
    let messages = body_of(&tool_turn("Je cherche"))["messages"].clone();
    let expected = json!({"role": "assistant", "content": "Je cherche", "tool_calls": [
        {"id": "c1", "type": "function", "function": {"name": "search_messages", "arguments": "{\"query\":\"devis\"}"}},
        {"id": "c2", "type": "function", "function": {"name": "get_thread", "arguments": "{\"thread\":\"t1\"}"}}
    ]});
    assert_eq!(messages[1], expected);
}

/// Chaque résultat est un message d'outil qui nomme son appel
#[test]
fn tool_message_carries_call_id() {
    let messages = body_of(&tool_turn("Je cherche"))["messages"].clone();
    assert_eq!(
        messages[2],
        json!({"role": "tool", "tool_call_id": "c1", "content": "r1"})
    );
    assert_eq!(
        messages[3],
        json!({"role": "tool", "tool_call_id": "c2", "content": "r2"})
    );
}

/// Sans texte, le contenu de l'assistant est nul
#[test]
fn empty_assistant_text_is_null() {
    let mut req = request();
    req.system = None;
    req.messages = vec![assistant_with("", vec![call("c1", "x", "{}")])];
    assert_eq!(body_of(&req)["messages"][0]["content"], Value::Null);
}
