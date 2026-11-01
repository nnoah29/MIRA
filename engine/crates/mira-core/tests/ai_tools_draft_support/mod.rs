//! Appels d'outils de brouillon et noms d'outils annoncés, pour les tests des propositions.
#![allow(dead_code)]

use mira_core::ai::tools::{ToolRun, propose, specs_for};
use mira_vocabulary::{ChatMode, ToolCall};
use serde_json::Value;

/// Noms des outils annoncés dans ce mode.
pub fn names(mode: ChatMode) -> Vec<String> {
    specs_for(mode).into_iter().map(|spec| spec.name).collect()
}

/// Appelle `compose_draft` en mode assistant.
pub fn compose(arguments: Value) -> ToolRun {
    let call = ToolCall {
        id: "c1".to_string(),
        name: "compose_draft".to_string(),
        arguments: arguments.to_string(),
    };
    propose(ChatMode::Assistant, &call)
}

/// Appelle `edit_draft` dans ce mode.
pub fn edit(mode: ChatMode, arguments: Value) -> ToolRun {
    let call = ToolCall {
        id: "c1".to_string(),
        name: "edit_draft".to_string(),
        arguments: arguments.to_string(),
    };
    propose(mode, &call)
}
