//! Réponses `result` : une enveloppe qui ne tient pas dans une trame est remplacée par un échec fixe.

use mira_host::envelope::internal_failure;
use mira_protocol::{Frame, encode};
use serde_json::Value;

/// Marge qui couvre `type` et `id` autour de l'enveloppe dans la trame.
const FRAME_OVERHEAD: usize = 64;

/// Fabrique la trame `result` de ce texte d'enveloppe ; trop grande pour `max`, elle devient `réponse trop grande`.
pub fn result_frame(id: u64, text: &str, max: u32) -> Frame {
    let envelope = serde_json::from_str(text).unwrap_or(Value::Null);
    let frame = Frame::Result { id, envelope };
    if text.len() + FRAME_OVERHEAD <= max as usize || encode(&frame, max).is_ok() {
        return frame;
    }
    let envelope =
        serde_json::from_str(&internal_failure("réponse trop grande")).unwrap_or(Value::Null);
    Frame::Result { id, envelope }
}
