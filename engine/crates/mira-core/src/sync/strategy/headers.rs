//! Téléchargement des en-têtes par lots de 100, du plus récent au plus ancien, avec progression.

use mira_vocabulary::{AccountId, EventKind, Mailbox, MessageId, MiraError, SyncStep};

use crate::imap::FetchedHeaders;
use crate::rules::apply::rules_apply_in;
use crate::sync::context::SyncContext;
use crate::sync::download::{count, report_progress};
use crate::sync::identity::{Incoming, Ingested, ingest_header};
use crate::sync::rule_hook::silenced_by_rules;
use crate::sync::session::SyncSession;

/// Taille d'un lot d'en-têtes, et donc fréquence minimale des événements de progression.
pub const HEADER_BATCH: usize = 100;

/// Faut-il annoncer les messages intégrés (nouveau message, mise à jour) ou rester silencieux ?
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Announce {
    Silent,
    Live,
}

/// UID triés du plus récent au plus ancien.
pub fn newest_first(uids: &[u32]) -> Vec<u32> {
    let mut sorted = uids.to_vec();
    sorted.sort_unstable_by(|a, b| b.cmp(a));
    sorted
}

/// Télécharge et intègre les en-têtes des UID donnés, un lot à la fois, puis publie l'avancement.
pub async fn ingest_headers(
    session: &mut SyncSession,
    mailbox: &Mailbox,
    validity: u32,
    uids: &[u32],
    announce: Announce,
) -> Result<(), MiraError> {
    let ordered = newest_first(uids);
    let total = count(ordered.len());
    report_progress(session, SyncStep::Headers, 0, total);
    let mut done = 0;
    for batch in ordered.chunks(HEADER_BATCH) {
        ingest_batch(session, mailbox, validity, batch, announce).await?;
        done += count(batch.len());
        report_progress(session, SyncStep::Headers, done, total);
    }
    Ok(())
}

/// Télécharge et intègre un seul lot d'en-têtes dans une transaction.
pub async fn ingest_batch(
    session: &mut SyncSession,
    mailbox: &Mailbox,
    validity: u32,
    batch: &[u32],
    announce: Announce,
) -> Result<(), MiraError> {
    let extra = session
        .ctx
        .provider
        .descriptor()
        .quirks
        .extra_fetch_attributes;
    let fetched = session.conn.client.fetch_headers(batch, &extra).await?;
    let outcomes = store_batch(&session.ctx, mailbox, validity, fetched, announce)?;
    let created = outcomes.iter().filter_map(Ingested::created_id);
    session.created_in_pass.extend(created.cloned());
    announce_batch(&session.ctx, mailbox, &outcomes, announce);
    Ok(())
}

/// Intègre un lot d'en-têtes dans une seule transaction ; en direct, inscrit les messages créés en attente des règles.
pub fn store_batch(
    ctx: &SyncContext,
    mailbox: &Mailbox,
    validity: u32,
    fetched: Vec<FetchedHeaders>,
    announce: Announce,
) -> Result<Vec<Ingested>, MiraError> {
    let evaluates = announce == Announce::Live && rules_apply_in(mailbox.role);
    ctx.db.write(|tx| {
        let mut outcomes = Vec::new();
        for headers in fetched {
            let incoming = Incoming::new(&*ctx.provider, &mailbox.id, validity, headers);
            let outcome = ingest_header(tx, ctx, &incoming)?;
            if let (true, Ingested::Created(message)) = (evaluates, &outcome) {
                tx.add_pending(message)?;
            }
            outcomes.push(outcome);
        }
        Ok(outcomes)
    })
}

/// Émet les événements des messages créés ou modifiés, sauf en mode silencieux.
fn announce_batch(ctx: &SyncContext, mailbox: &Mailbox, outcomes: &[Ingested], mode: Announce) {
    if mode == Announce::Silent {
        return;
    }
    for outcome in outcomes {
        match outcome {
            Ingested::Created(message) => {
                let by_rules = rules_apply_in(mailbox.role) && silenced_by_rules(ctx, message);
                let silenced = by_rules || silenced_by_labels(ctx, message);
                ctx.emit(EventKind::NewMessageReceived {
                    account: ctx.account.id.clone(),
                    message: message.clone(),
                    mailbox: mailbox.id.clone(),
                    silenced,
                });
            }
            Ingested::Updated(message) => ctx.emit(updated(&ctx.account.id, message)),
            Ingested::Unchanged(_) => {}
        }
    }
}

/// Événement « message modifié ».
pub fn updated(account: &AccountId, message: &MessageId) -> EventKind {
    EventKind::MessageUpdated {
        account: account.clone(),
        message: message.clone(),
    }
}

/// Vrai si un libellé du message le masque de la Réception ou coupe ses notifications
fn silenced_by_labels(ctx: &SyncContext, message: &MessageId) -> bool {
    ctx.db
        .read(|tx| tx.label_silences(message))
        .unwrap_or(false)
}
