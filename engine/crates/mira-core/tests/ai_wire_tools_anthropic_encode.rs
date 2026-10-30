#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Format Messages, encodage des outils : annonce, appels de l'assistant, résultats regroupés.

mod ai_wire_support;

use ai_wire_support::request;
use ai_wire_support::tools::{assistant_with, call, messages_descriptor, search_tool, tool_turn};
use mira_core::ai::wire::anthropic_encode::encode_chat;
use mira_vocabulary::ChatRequest;
use serde_json::{Value, json};

/// Corps JSON encodé de la requête.
fn body_of(request: &ChatRequest) -> Value {
    serde_json::from_str(&encode_chat(request, "m1", &messages_descriptor()).body).unwrap()
}

/// Les outils sont annoncés avec leur schéma d'arguments
#[test]
fn tools_are_announced_with_schema() {
    let mut req = request();
    req.tools = vec![search_tool()];
    let expected = json!([{
        "name": "search_messages",
        "description": "Cherche dans la boîte.",
        "input_schema": {"type": "object", "properties": {"query": {"type": "string"}}, "required": ["query"]}
    }]);
    assert_eq!(body_of(&req)["tools"], expected);
}

/// Sans outil, le champ n'existe pas
#[test]
fn no_tools_field_when_list_is_empty() {
    assert!(body_of(&request()).get("tools").is_none());
}

/// Le texte puis les appels de l'assistant deviennent des blocs, dans l'ordre
#[test]
fn assistant_tool_calls_become_wire_blocks() {
    let messages = body_of(&tool_turn("Je cherche"))["messages"].clone();
    let expected = json!({"role": "assistant", "content": [
        {"type": "text", "text": "Je cherche"},
        {"type": "tool_use", "id": "c1", "name": "search_messages", "input": {"query": "devis"}},
        {"type": "tool_use", "id": "c2", "name": "get_thread", "input": {"thread": "t1"}}
    ]});
    assert_eq!(messages[1], expected);
}

/// Les résultats consécutifs forment un seul message de l'utilisateur
#[test]
fn consecutive_tool_results_share_one_user_message() {
    let messages = body_of(&tool_turn("Je cherche"))["messages"].clone();
    assert_eq!(messages.as_array().unwrap().len(), 3);
    let expected = json!({"role": "user", "content": [
        {"type": "tool_result", "tool_use_id": "c1", "content": "r1", "is_error": false},
        {"type": "tool_result", "tool_use_id": "c2", "content": "r2", "is_error": true}
    ]});
    assert_eq!(messages[2], expected);
}

/// Sans texte, pas de bloc de texte ; des arguments illisibles donnent un objet vide
#[test]
fn empty_text_and_unreadable_arguments_are_handled() {
    let mut req = request();
    req.messages = vec![assistant_with("", vec![call("c1", "x", "{pas du json")])];
    let content = body_of(&req)["messages"][0]["content"].clone();
    let expected = json!([{"type": "tool_use", "id": "c1", "name": "x", "input": {}}]);
    assert_eq!(content, expected);
}
