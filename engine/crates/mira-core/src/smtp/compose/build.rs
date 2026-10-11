//! Assemblage du message avec `mail-builder` : en-têtes, corps texte et HTML, pièces jointes.

use mail_builder::MessageBuilder;
use mail_builder::headers::message_id::MessageId;
use mira_vocabulary::error::codes::internal;
use mira_vocabulary::{ErrorCategory, MiraError};

use super::addresses::{checked_address, envelope_recipients, header_address, header_list};
use super::built::BuiltMessage;
use super::input::ComposeInput;
use super::sanitize::{bare_message_id, single_line};

/// Compose le message MIME ; les Bcc restent dans l'enveloppe, jamais dans les en-têtes
pub fn build_message(input: &ComposeInput) -> Result<BuiltMessage, MiraError> {
    let recipients = envelope_recipients(input)?;
    let sender = checked_address(&input.from)?;
    let id = bare_message_id(&input.message_id)?;
    let builder = MessageBuilder::new()
        .from(header_address(&input.from)?)
        .subject(single_line(&input.subject))
        .date(input.date.as_millis().div_euclid(1000))
        .message_id(id.clone())
        .text_body(input.body_text.clone());
    let builder = add_recipients(builder, input)?;
    let builder = add_threading(builder, input)?;
    let raw = add_content(builder, input).write_to_vec().map_err(|_| {
        MiraError::new(
            ErrorCategory::Internal,
            internal::BUG,
            "composition du message",
        )
    })?;
    Ok(BuiltMessage {
        raw,
        message_id: format!("<{id}>"),
        envelope_from: sender.as_str().to_string(),
        envelope_recipients: recipients,
    })
}

/// Ajoute les en-têtes To et Cc (jamais Bcc)
fn add_recipients<'x>(
    builder: MessageBuilder<'x>,
    input: &ComposeInput,
) -> Result<MessageBuilder<'x>, MiraError> {
    let mut builder = builder;
    if let Some(to) = header_list(&input.to)? {
        builder = builder.to(to);
    }
    if let Some(cc) = header_list(&input.cc)? {
        builder = builder.cc(cc);
    }
    Ok(builder)
}

/// Ajoute In-Reply-To et References quand ils sont fournis
fn add_threading<'x>(
    builder: MessageBuilder<'x>,
    input: &ComposeInput,
) -> Result<MessageBuilder<'x>, MiraError> {
    let mut builder = builder;
    if let Some(parent) = &input.in_reply_to {
        builder = builder.in_reply_to(bare_message_id(parent)?);
    }
    if !input.references.is_empty() {
        let ids = input
            .references
            .iter()
            .map(|id| bare_message_id(id))
            .collect::<Result<Vec<_>, _>>()?;
        builder = builder.references(MessageId::new_list(ids.into_iter()));
    }
    Ok(builder)
}

/// Ajoute la partie HTML et les pièces jointes
fn add_content<'x>(builder: MessageBuilder<'x>, input: &ComposeInput) -> MessageBuilder<'x> {
    let mut builder = builder;
    if let Some(html) = &input.body_html {
        builder = builder.html_body(html.clone());
    }
    for attachment in &input.attachments {
        builder = builder.attachment(
            single_line(&attachment.mime_type),
            single_line(&attachment.filename),
            attachment.data.clone(),
        );
    }
    builder
}
