//! Troncature de l'entrée avant envoi : plafond compté en caractères, consigne et messages compris.

use mira_vocabulary::ChatRequest;

/// Plafond de l'entrée envoyée au service, en caractères.
pub const MAX_INPUT_CHARS: usize = 120_000;

pub(crate) const TRUNCATION_MARK: &str = "…[tronqué]";

/// Retire les plus anciens messages jusqu'à tenir ; un dernier message trop long est coupé et marqué.
pub fn truncate_request(mut request: ChatRequest, max_chars: usize) -> ChatRequest {
    let system = request.system.as_deref().map_or(0, char_count);
    let lengths: Vec<usize> = request
        .messages
        .iter()
        .map(|m| char_count(&m.text))
        .collect();
    let mut total = system + lengths.iter().sum::<usize>();
    if total <= max_chars {
        return request;
    }
    let mut dropped = 0;
    while total > max_chars && lengths.len() - dropped > 1 {
        total -= lengths[dropped];
        dropped += 1;
    }
    request.messages.drain(..dropped);
    if let (true, Some(last)) = (total > max_chars, request.messages.last_mut()) {
        last.text = cut(&last.text, max_chars.saturating_sub(system));
    }
    request
}

/// Garde le début du texte, caractère par caractère, puis ajoute la marque de troncature.
fn cut(text: &str, budget: usize) -> String {
    let keep = budget.saturating_sub(char_count(TRUNCATION_MARK));
    let mut kept: String = text.chars().take(keep).collect();
    kept.push_str(TRUNCATION_MARK);
    kept
}

/// Nombre de caractères envoyés (consigne et messages), pour le journal.
pub(super) fn request_chars(request: &ChatRequest) -> u32 {
    let system = request.system.as_deref().map_or(0, char_count);
    let messages: usize = request.messages.iter().map(|m| char_count(&m.text)).sum();
    u32::try_from(system + messages).unwrap_or(u32::MAX)
}

/// Compte les caractères (et non les octets) d'un texte.
fn char_count(text: &str) -> usize {
    text.chars().count()
}
