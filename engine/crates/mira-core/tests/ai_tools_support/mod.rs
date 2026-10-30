//! Appel d'outil et garnissage d'un monde de lecture pour les tests des outils d'IA.
#![allow(dead_code)]

use mira_vocabulary::{MessageId, ToolCall};
use serde_json::Value;

use crate::support_read::seed::add_body;
use crate::support_read::world::World;

/// Appel d'outil dont les arguments sont l'objet JSON donné.
pub fn call(name: &str, arguments: Value) -> ToolCall {
    ToolCall {
        id: "c1".to_string(),
        name: name.to_string(),
        arguments: arguments.to_string(),
    }
}

/// Garnit la boîte de `count` messages m1… dont le corps indexé est `text`.
pub fn index_bodies(world: &World, count: u32, text: &str) {
    world.seed_messages(count);
    for index in 1..=count {
        let id = MessageId::from_raw(format!("m{index}"));
        add_body(&world.mirror, &id, text, None);
    }
}
