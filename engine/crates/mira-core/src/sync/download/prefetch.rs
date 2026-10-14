//! Étape 3 de la descente : préchargement des corps des messages les plus récents de la réception.

use mira_vocabulary::{
    ContentState, ErrorCategory, EventKind, LogLevel, Mailbox, MailboxId, MessageId, MiraError,
    PageRequest, SyncStep,
};

use super::body::store_body;
use super::progress::{count, report_progress};
use crate::sync::context::SyncContext;
use crate::sync::session::SyncSession;

const MAX_PREFETCH_BYTES: u64 = 10 * 1024 * 1024;

/// Les `wanted` messages les plus récents de la boîte qui n'ont encore que leurs en-têtes, avec leur UID.
fn bodies_to_fetch(
    ctx: &SyncContext,
    mailbox: &MailboxId,
    wanted: u32,
) -> Result<Vec<(MessageId, u32)>, MiraError> {
    ctx.db.read(|tx| {
        let request = PageRequest {
            cursor: None,
            limit: wanted,
        };
        let page = tx.list_messages_in_mailbox(mailbox, &request)?;
        let mut targets = Vec::new();
        for summary in page.items {
            let appearance = tx.get_appearance(&summary.id, mailbox)?;
            let uid = appearance.map_or(0, |found| found.uid);
            let light = summary.size <= MAX_PREFETCH_BYTES;
            if summary.content == ContentState::HeadersOnly && uid > 0 && light {
                targets.push((summary.id, uid));
            }
        }
        Ok(targets)
    })
}

/// Télécharge la source d'un message, l'enregistre, puis annonce que le corps est disponible.
async fn fetch_one(
    session: &mut SyncSession,
    message: &MessageId,
    uid: u32,
) -> Result<(), MiraError> {
    let Some(raw) = session.conn.client.fetch_source(uid).await? else {
        return Ok(());
    };
    let ctx = session.ctx.clone();
    if ctx.db.write(|tx| store_body(tx, &ctx, message, &raw))? {
        ctx.emit(EventKind::BodyAvailable {
            account: ctx.account.id.clone(),
            message: message.clone(),
        });
    }
    Ok(())
}

/// Un refus du serveur pour un message est journalisé et ignoré (il ne bloque pas les autres) ; toute autre erreur interrompt la passe.
fn tolerate_refusal(
    session: &SyncSession,
    outcome: Result<(), MiraError>,
) -> Result<(), MiraError> {
    match outcome {
        Err(error) if error.category == ErrorCategory::Server => {
            let text = format!("corps non préchargé : {}", error.code);
            session.ctx.diagnose(LogLevel::Warn, "sync.prefetch", &text);
            Ok(())
        }
        other => other,
    }
}

/// Précharge les corps des plus récents messages de la réception (50 par défaut).
pub async fn prefetch_bodies(session: &mut SyncSession, inbox: &Mailbox) -> Result<(), MiraError> {
    let wanted = session.ctx.account.sync.prefetch_bodies;
    if wanted == 0 {
        return Ok(());
    }
    let targets = bodies_to_fetch(&session.ctx, &inbox.id, wanted)?;
    if targets.is_empty() {
        return Ok(());
    }
    session.conn.client.select(&inbox.name, None).await?;
    let total = count(targets.len());
    report_progress(session, SyncStep::Bodies, 0, total);
    for (index, (message, uid)) in targets.iter().enumerate() {
        let outcome = fetch_one(session, message, *uid).await;
        tolerate_refusal(session, outcome)?;
        let done = count(index + 1);
        if done.is_multiple_of(10) || done == total {
            report_progress(session, SyncStep::Bodies, done, total);
        }
    }
    Ok(())
}
