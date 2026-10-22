//! Conditions sur l'objet, l'aperçu et les pièces jointes.

use super::context::MessageContext;
use super::text_match::matches_text;
use mira_vocabulary::TextMatcher;

/// L'objet, s'il existe, correspond au filtre.
pub fn subject(ctx: &MessageContext, matcher: &TextMatcher) -> bool {
    !ctx.summary.subject.is_empty() && matches_text(matcher, &ctx.summary.subject)
}

/// L'aperçu, s'il existe, correspond au filtre.
pub fn preview(ctx: &MessageContext, matcher: &TextMatcher) -> bool {
    !ctx.summary.preview.is_empty() && matches_text(matcher, &ctx.summary.preview)
}

/// Le message porte au moins une pièce jointe.
pub fn has_attachment(ctx: &MessageContext) -> bool {
    ctx.summary.has_attachments || !ctx.attachments.is_empty()
}

/// Le nom d'une pièce jointe correspond au filtre.
pub fn attachment_name(ctx: &MessageContext, matcher: &TextMatcher) -> bool {
    ctx.attachments
        .iter()
        .any(|attachment| matches_text(matcher, &attachment.name))
}

/// Le type d'une pièce jointe correspond au filtre.
pub fn attachment_type(ctx: &MessageContext, matcher: &TextMatcher) -> bool {
    ctx.attachments
        .iter()
        .any(|attachment| matches_text(matcher, &attachment.media_type))
}
