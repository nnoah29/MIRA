//! Rendu du contexte typé dans le dernier message de l'utilisateur, avec balises neutralisées.

use mira_vocabulary::{ChatMessage, ChatRequest, ChatRole, ContextPart};

/// Place les parties de contexte avant le texte du dernier message de l'utilisateur, puis vide `context`.
pub fn render_context(mut request: ChatRequest) -> ChatRequest {
    if request.context.is_empty() {
        return request;
    }
    let block: String = request.context.drain(..).map(render_part).collect();
    match request
        .messages
        .iter_mut()
        .rev()
        .find(|message| message.role == ChatRole::User)
    {
        Some(message) => message.text.insert_str(0, &block),
        None => request.messages.push(ChatMessage::plain(
            ChatRole::User,
            block.trim_end().to_string(),
        )),
    }
    request
}

/// Une partie entre ses balises, étiquette et texte rendus inoffensifs.
fn render_part(part: ContextPart) -> String {
    let label = part.label.replace(['\n', '\r'], " ").replace('"', "&quot;");
    format!(
        "<context kind=\"{}\" label=\"{}\">\n{}\n</context>\n\n",
        part.kind.as_str(),
        neutralize(&label),
        neutralize(&part.text)
    )
}

/// Remplace le `<` de toute balise `context` (ouvrante ou fermante, casse et blancs quelconques) par `&lt;`.
fn neutralize(text: &str) -> String {
    let mut neutral = String::with_capacity(text.len());
    for (index, character) in text.char_indices() {
        let rest = text.get(index + 1..).unwrap_or_default();
        match character == '<' && is_context_tag(rest) {
            true => neutral.push_str("&lt;"),
            false => neutral.push(character),
        }
    }
    neutral
}

/// Dit si le texte qui suit un `<` nomme la balise `context`, avec ou sans `/`, blancs tolérés.
fn is_context_tag(after_bracket: &str) -> bool {
    let rest = after_bracket.trim_start();
    let rest = rest.strip_prefix('/').unwrap_or(rest).trim_start();
    rest.get(.."context".len())
        .is_some_and(|word| word.eq_ignore_ascii_case("context"))
}
