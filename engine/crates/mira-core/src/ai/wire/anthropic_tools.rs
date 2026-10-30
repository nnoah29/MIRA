//! Format Messages : annonce des outils, appels de l'assistant et résultats regroupés en un message.

use mira_vocabulary::{ChatMessage, ChatRole, ToolCall, ToolOutcome, ToolSpec};
use serde_json::{Value, json};

use super::json_text::object_or;

/// Liste des outils au format du service ; absente quand il n'y en a aucun.
pub(super) fn tools_value(tools: &[ToolSpec]) -> Option<Value> {
    let wire = |tool: &ToolSpec| {
        let schema = object_or(&tool.parameters_schema, json!({"type": "object"}));
        json!({"name": tool.name, "description": tool.description, "input_schema": schema})
    };
    (!tools.is_empty()).then(|| Value::Array(tools.iter().map(wire).collect()))
}

/// Messages du fil ; les résultats d'outils consécutifs partagent un seul message de l'utilisateur.
pub(super) fn message_values(messages: &[ChatMessage]) -> Vec<Value> {
    let mut values: Vec<Value> = Vec::new();
    for message in messages {
        match &message.tool_result {
            Some(outcome) => push_result(&mut values, outcome),
            None if message.tool_calls.is_empty() => values.push(plain_value(message)),
            None => values.push(assistant_value(message)),
        }
    }
    values
}

/// Ajoute le résultat au message de résultats précédent, ou en ouvre un.
fn push_result(values: &mut Vec<Value>, outcome: &ToolOutcome) {
    let block = json!({
        "type": "tool_result",
        "tool_use_id": outcome.call_id,
        "content": outcome.text,
        "is_error": outcome.is_error
    });
    let open = values
        .last_mut()
        .filter(|last| last["content"][0]["type"] == "tool_result")
        .and_then(|last| last["content"].as_array_mut());
    match open {
        Some(blocks) => blocks.push(block),
        None => values.push(json!({"role": "user", "content": [block]})),
    }
}

/// Message de l'assistant : son texte éventuel puis un bloc par appel.
fn assistant_value(message: &ChatMessage) -> Value {
    let text = (!message.text.is_empty()).then(|| json!({"type": "text", "text": message.text}));
    let calls = message.tool_calls.iter().map(call_block);
    let blocks: Vec<Value> = text.into_iter().chain(calls).collect();
    json!({"role": "assistant", "content": blocks})
}

/// Bloc d'appel ; des arguments illisibles deviennent un objet vide.
fn call_block(call: &ToolCall) -> Value {
    let input = object_or(&call.arguments, json!({}));
    json!({"type": "tool_use", "id": call.id, "name": call.name, "input": input})
}

/// Message de texte seul.
fn plain_value(message: &ChatMessage) -> Value {
    let role = match message.role {
        ChatRole::Assistant => "assistant",
        ChatRole::User | ChatRole::Tool => "user",
    };
    json!({"role": role, "content": message.text})
}
