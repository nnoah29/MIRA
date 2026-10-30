//! Requêtes et parties de contexte de test pour le rendu du contexte.

use mira_vocabulary::{ChatMessage, ChatRequest, ChatRole, ContentKind, ContextPart};

/// Partie de contexte de ce type, avec cette étiquette et ce texte
pub fn part(kind: ContentKind, label: &str, text: &str) -> ContextPart {
    ContextPart {
        kind,
        label: label.to_string(),
        text: text.to_string(),
    }
}

/// Requête de ces messages (auteur, texte) portant ce contexte
pub fn request(messages: &[(ChatRole, &str)], context: Vec<ContextPart>) -> ChatRequest {
    ChatRequest {
        model: None,
        system: None,
        messages: messages
            .iter()
            .map(|(role, text)| ChatMessage::plain(*role, (*text).to_string()))
            .collect(),
        max_output_tokens: None,
        temperature: None,
        tools: Vec::new(),
        context,
    }
}

/// Nombre d'occurrences de ce motif dans le texte, sans tenir compte de la casse
pub fn count_ignoring_case(text: &str, pattern: &str) -> usize {
    text.to_lowercase().matches(&pattern.to_lowercase()).count()
}
