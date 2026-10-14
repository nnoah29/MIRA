//! Résumé d'un message construit depuis ses en-têtes bruts, et texte indexé des adresses.

use mira_vocabulary::{AddressEntry, ContentState, MessageId, MessageSummary, Timestamp};

use super::flags::server_flags;
use super::incoming::Incoming;
use crate::mime::parse_unsubscribe_headers;
use crate::sync::context::SyncContext;

/// Résumé « en-têtes seuls » d'un message qui n'existe pas encore dans le miroir.
pub fn build_summary(ctx: &SyncContext, id: &MessageId, incoming: &Incoming) -> MessageSummary {
    let (parsed, fetched) = (&incoming.parsed, &incoming.fetched);
    MessageSummary {
        id: id.clone(),
        account: ctx.account.id.clone(),
        thread: None,
        from: parsed.from.clone(),
        to: parsed.to.clone(),
        cc: parsed.cc.clone(),
        bcc: parsed.bcc.clone(),
        reply_to: parsed.reply_to.clone(),
        subject: parsed.subject.clone(),
        date: parsed.date,
        internal_date: fetched.internal_date,
        message_id_header: parsed.message_id.clone(),
        in_reply_to: parsed.in_reply_to.clone(),
        references: parsed.references.clone(),
        flags: server_flags(&fetched.flags),
        size: fetched.size,
        preview: String::new(),
        has_attachments: false,
        unsubscribe: parse_unsubscribe_headers(&fetched.header_bytes),
        content: ContentState::HeadersOnly,
        mailboxes: Vec::new(),
    }
}

/// Date de tri : l'en-tête `Date`, sinon la date interne, sinon l'instant de la synchronisation.
pub fn sort_date(ctx: &SyncContext, summary: &MessageSummary) -> Timestamp {
    summary
        .date
        .or(summary.internal_date)
        .unwrap_or_else(|| ctx.clock.now())
}

/// Texte des adresses (expéditeur, destinataires, copie) pour l'index plein texte.
pub fn address_text(summary: &MessageSummary) -> String {
    let all = [&summary.from, &summary.to, &summary.cc];
    let entries = all.into_iter().flatten();
    let words: Vec<String> = entries.map(entry_text).collect();
    words.join(" ")
}

/// Nom d'affichage et adresse d'une entrée, séparés par une espace.
fn entry_text(entry: &AddressEntry) -> String {
    let name = entry.name.as_deref().unwrap_or_default();
    format!("{name} {}", entry.address).trim().to_string()
}
