//! Corps de message pour le modèle : texte seul, borné par caractères, marqué quand il est coupé.

use mira_vocabulary::MessageId;
use mira_vocabulary::error::codes::incompatible_state::BODY_UNAVAILABLE_OFFLINE;

use super::execute::execute_read;
use crate::ai::chat::TRUNCATION_MARK;
use crate::api::{Engine, MessageRequest, Request, Response};

/// Borne d'un résultat d'outil, en caractères.
pub const RESULT_CHARS: usize = 8_000;

const OFFLINE_NOTICE: &str = "(corps non disponible hors ligne)";

/// Texte tenant dans `max` caractères, marque comprise ; un texte blanc est dit « corps vide ».
pub fn bounded(text: &str, max: usize) -> String {
    if text.trim().is_empty() {
        return "(corps vide)".to_string();
    }
    if text.chars().count() <= max {
        return text.to_string();
    }
    let keep = max.saturating_sub(TRUNCATION_MARK.chars().count());
    let mut kept: String = text.chars().take(keep).collect();
    kept.push_str(TRUNCATION_MARK);
    kept
}

/// Corps texte du message borné à `max` ; absent hors ligne, il est annoncé sans échec.
pub(super) async fn body_text(
    engine: &Engine,
    message: &MessageId,
    max: usize,
) -> Result<String, String> {
    let request = Request::Message(MessageRequest::GetMessageBody {
        message: message.clone(),
    });
    match execute_read(engine, request).await {
        Ok(Response::MessageBody(view)) => Ok(bounded(&view.text, max)),
        Ok(_) => Err("réponse inattendue du moteur".to_string()),
        Err(failure) if failure.is_code(&BODY_UNAVAILABLE_OFFLINE) => {
            Ok(OFFLINE_NOTICE.to_string())
        }
        Err(failure) => Err(failure.text()),
    }
}
