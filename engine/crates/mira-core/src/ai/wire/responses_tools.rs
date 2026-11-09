//! Format Responses : annonce des outils, éléments d'appel de fonction et de résultat.

use mira_vocabulary::{ChatMessage, ChatRole, ToolCall, ToolSpec};
use serde_json::{Value, json};

use super::json_text::object_or;

/// Liste des outils comme fonctions à plat ; absente quand il n'y en a aucun.
pub(super) fn tools_value(tools: &[ToolSpec]) -> Option<Value> {
    let wire = |tool: &ToolSpec| {
        let schema = object_or(&tool.parameters_schema, json!({"type": "object"}));
        json!({"type": "function", "name": tool.name, "description": tool.description, "parameters": schema})
    };
    (!tools.is_empty()).then(|| Value::Array(tools.iter().map(wire).collect()))
}

/// Éléments d'entrée d'un message : résultat d'outil, ou texte puis appels de l'assistant.
pub(super) fn message_items(message: &ChatMessage) -> Vec<Value> {
    if let Some(outcome) = &message.tool_result {
        let output = json!({"type": "function_call_output", "call_id": outcome.call_id, "output": outcome.text});
        return vec![output];
    }
    let mut items: Vec<Value> = text_item(message).into_iter().collect();
    items.extend(message.tool_calls.iter().map(call_item));
    items
}

/// Message de texte : l'utilisateur écrit `input_text`, l'assistant `output_text` ; un texte vide ou un autre rôle est écarté.
fn text_item(message: &ChatMessage) -> Option<Value> {
    if message.text.is_empty() && !message.tool_calls.is_empty() {
        return None;
    }
    let (role, kind) = match message.role {
        ChatRole::User => ("user", "input_text"),
        ChatRole::Assistant => ("assistant", "output_text"),
        ChatRole::Tool => return None,
    };
    Some(json!({"role": role, "content": [{"type": kind, "text": message.text}]}))
}

/// Appel de fonction ; les arguments restent la chaîne JSON du modèle.
fn call_item(call: &ToolCall) -> Value {
    json!({"type": "function_call", "call_id": call.id, "name": call.name, "arguments": call.arguments})
}
