//! Format OpenAI Chat : annonce des outils, appels de l'assistant et messages de résultat.

use mira_vocabulary::{ChatMessage, ChatRole, ToolCall, ToolSpec};
use serde_json::{Value, json};

use super::json_text::object_or;

/// Liste des outils comme fonctions ; absente quand il n'y en a aucun.
pub(super) fn tools_value(tools: &[ToolSpec]) -> Option<Value> {
    let wire = |tool: &ToolSpec| {
        let schema = object_or(&tool.parameters_schema, json!({"type": "object"}));
        let function =
            json!({"name": tool.name, "description": tool.description, "parameters": schema});
        json!({"type": "function", "function": function})
    };
    (!tools.is_empty()).then(|| Value::Array(tools.iter().map(wire).collect()))
}

/// Message du fil : texte seul, appels de l'assistant ou résultat d'un outil.
pub(super) fn message_value(message: &ChatMessage) -> Value {
    if let Some(outcome) = &message.tool_result {
        return json!({"role": "tool", "tool_call_id": outcome.call_id, "content": outcome.text});
    }
    let role = match message.role {
        ChatRole::User => "user",
        ChatRole::Assistant => "assistant",
        ChatRole::Tool => "tool",
    };
    if message.tool_calls.is_empty() {
        return json!({"role": role, "content": message.text});
    }
    let content = (!message.text.is_empty()).then_some(message.text.as_str());
    let calls: Vec<Value> = message.tool_calls.iter().map(call_value).collect();
    json!({"role": role, "content": content, "tool_calls": calls})
}

/// Appel de fonction ; les arguments restent la chaîne JSON du modèle.
fn call_value(call: &ToolCall) -> Value {
    let function = json!({"name": call.name, "arguments": call.arguments});
    json!({"id": call.id, "type": "function", "function": function})
}
