//! Rendu du contexte typé dans le dernier message de l'utilisateur, avec balises neutralisées.

use mira_vocabulary::{ChatMessage, ChatRequest, ChatRole, ContextPart};

use super::neutralize::neutralize_tag;

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
        neutralize_tag(&label, "context"),
        neutralize_tag(&part.text, "context")
    )
}
