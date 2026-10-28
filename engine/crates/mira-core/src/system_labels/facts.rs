//! Faits d'un fil lus en base : son dernier message vivant, hors brouillons, corbeille et spam.

use std::collections::HashMap;

use mira_vocabulary::{MailboxId, MailboxRole, MessageSummary, MiraError, ThreadId};

use super::automated::is_automated_address;
use super::classify::LastMessage;
use super::me::Me;
use crate::storage::Tx;

/// Dernier message vivant d'un fil (le plus récent, à égalité l'identifiant le plus grand), ou rien
pub fn last_live_message(
    tx: &Tx<'_>,
    thread: &ThreadId,
    me: &Me,
) -> Result<Option<(i64, LastMessage)>, MiraError> {
    let mut roles: HashMap<MailboxId, Option<MailboxRole>> = HashMap::new();
    let mut live = Vec::new();
    for summary in tx.messages_of_thread(thread)? {
        if is_live(tx, &summary, &mut roles)? {
            live.push(summary);
        }
    }
    let last = live
        .into_iter()
        .max_by_key(|m| (when(m), m.id.as_str().to_string()));
    let Some(summary) = last else {
        return Ok(None);
    };
    let text = tx.load_parsed(&summary.id)?.map(|parsed| parsed.text);
    Ok(Some((when(&summary), describe(&summary, me, text))))
}

/// Instant du message : sa date, sinon sa date de réception
fn when(summary: &MessageSummary) -> i64 {
    let stamp = summary.date.or(summary.internal_date);
    stamp.map_or(0, |t| t.millis)
}

/// Le message est-il dans au moins une boîte qui ne soit ni brouillons, ni corbeille, ni spam ?
fn is_live(
    tx: &Tx<'_>,
    summary: &MessageSummary,
    roles: &mut HashMap<MailboxId, Option<MailboxRole>>,
) -> Result<bool, MiraError> {
    for mailbox in &summary.mailboxes {
        if !roles.contains_key(mailbox) {
            let role = tx.get_mailbox(mailbox)?.map(|m| m.role);
            roles.insert(mailbox.clone(), role);
        }
        let hidden = matches!(
            roles.get(mailbox),
            Some(Some(
                MailboxRole::Drafts | MailboxRole::Trash | MailboxRole::Junk
            )) | Some(None)
        );
        if !hidden {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Ce qui compte dans un message : qui l'envoie, à qui, quand, et son texte (à défaut, son aperçu)
fn describe(summary: &MessageSummary, me: &Me, text: Option<String>) -> LastMessage {
    let sender = summary.from.first().map(|a| a.address.as_str());
    let recipients = summary.to.iter().chain(&summary.cc);
    LastMessage {
        from_own: sender.is_some_and(|a| me.owns(a)),
        sender_automated: summary.unsubscribe.is_some() || sender.is_some_and(is_automated_address),
        awaits_human: recipients
            .into_iter()
            .any(|r| !me.owns(&r.address) && !is_automated_address(&r.address)),
        date: when(summary),
        text: text.unwrap_or_else(|| summary.preview.clone()),
    }
}
