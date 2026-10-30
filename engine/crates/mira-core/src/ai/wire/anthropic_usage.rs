//! Lecture de la consommation de jetons portée par un événement du format Messages.

use mira_vocabulary::ChatDelta;
use serde_json::Value;

/// Extrait la consommation portée par l'événement, un champ absent valant 0.
pub(super) fn usage_delta(usage: Option<&Value>) -> Option<ChatDelta> {
    let usage = usage?.as_object()?;
    let count = |name: &str| token_count(usage.get(name));
    Some(ChatDelta::Usage {
        input_tokens: count("input_tokens"),
        output_tokens: count("output_tokens"),
    })
}

/// Convertit un compteur JSON en u32, 0 s'il est absent ou invalide, saturé s'il est trop grand.
fn token_count(value: Option<&Value>) -> u32 {
    let count = value.and_then(Value::as_u64).unwrap_or(0);
    u32::try_from(count).unwrap_or(u32::MAX)
}
