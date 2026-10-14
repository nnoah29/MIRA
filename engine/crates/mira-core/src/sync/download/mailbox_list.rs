//! Étape 1 de la descente : la liste des boîtes du compte, avec leurs rôles (le serveur gagne, D-23).

use std::collections::HashMap;

use mira_vocabulary::{EventKind, Mailbox, MailboxId, MiraError, RawMailbox, SyncStep};

use super::names::{is_selectable, split_name};
use super::progress::{count, report_progress};
use crate::mailboxes::infer_mailbox_role;
use crate::storage::Tx;
use crate::sync::context::SyncContext;
use crate::sync::session::SyncSession;

/// Identifiant de chaque nom de boîte du serveur : celui du miroir s'il existe, sinon un neuf.
fn assign_ids(
    ctx: &SyncContext,
    known: &[Mailbox],
    raws: &[&RawMailbox],
) -> HashMap<String, MailboxId> {
    let existing: HashMap<&str, &MailboxId> =
        known.iter().map(|m| (m.name.as_str(), &m.id)).collect();
    let mut ids = HashMap::new();
    for raw in raws {
        let id = existing.get(raw.name.as_str()).map(|id| (*id).clone());
        ids.insert(
            raw.name.clone(),
            id.unwrap_or_else(|| MailboxId::from_raw(ctx.new_id())),
        );
    }
    ids
}

/// Une boîte est synchronisée selon le choix du compte, sinon selon son état précédent (par défaut oui).
fn enabled_for(ctx: &SyncContext, id: &MailboxId, previous: Option<&Mailbox>) -> bool {
    match &ctx.account.sync.synced_mailboxes {
        Some(chosen) => chosen.contains(id),
        None => previous.is_none_or(|old| old.sync_enabled),
    }
}

/// Boîte du miroir décrite par la réponse LIST, en gardant repère et compteurs existants.
fn mailbox_of(
    ctx: &SyncContext,
    raw: &RawMailbox,
    ids: &HashMap<String, MailboxId>,
    previous: Option<&Mailbox>,
) -> Mailbox {
    let (display_name, parent_name) = split_name(raw);
    let id = ids[&raw.name].clone();
    Mailbox {
        sync_enabled: enabled_for(ctx, &id, previous),
        id,
        account: ctx.account.id.clone(),
        name: raw.name.clone(),
        display_name,
        delimiter: raw.delimiter.clone(),
        parent: parent_name.and_then(|name| ids.get(&name).cloned()),
        role: infer_mailbox_role(&*ctx.provider, raw),
        marker: previous.and_then(|old| old.marker),
        total: previous.map_or(0, |old| old.total),
        unread: previous.map_or(0, |old| old.unread),
    }
}

/// Aligne les boîtes du miroir sur le serveur ; rend les événements de création et de suppression.
fn reconcile(
    tx: &Tx<'_>,
    ctx: &SyncContext,
    raws: &[RawMailbox],
) -> Result<Vec<EventKind>, MiraError> {
    let known = tx.list_mailboxes(&ctx.account.id)?;
    let listed: Vec<&RawMailbox> = raws.iter().filter(|raw| is_selectable(raw)).collect();
    let ids = assign_ids(ctx, &known, &listed);
    let mut events = Vec::new();
    for raw in &listed {
        let previous = known.iter().find(|m| m.name == raw.name);
        tx.upsert_mailbox(&mailbox_of(ctx, raw, &ids, previous))?;
        if previous.is_none() {
            let (account, mailbox) = (ctx.account.id.clone(), ids[&raw.name].clone());
            events.push(EventKind::MailboxCreated { account, mailbox });
        }
    }
    for old in known.iter().filter(|m| !ids.contains_key(&m.name)) {
        tx.delete_mailbox(&old.id)?;
        let (account, mailbox) = (ctx.account.id.clone(), old.id.clone());
        events.push(EventKind::MailboxDeleted { account, mailbox });
    }
    Ok(events)
}

/// Liste les boîtes du serveur, met le miroir à jour et rend toutes les boîtes du compte.
pub async fn sync_mailbox_list(session: &mut SyncSession) -> Result<Vec<Mailbox>, MiraError> {
    report_progress(session, SyncStep::Mailboxes, 0, 1);
    let raws = session.conn.client.list_mailboxes().await?;
    let ctx = session.ctx.clone();
    let events = ctx.db.write(|tx| reconcile(tx, &ctx, &raws))?;
    for event in events {
        ctx.emit(event);
    }
    let all = ctx.db.read(|tx| tx.list_mailboxes(&ctx.account.id))?;
    let total = count(all.len());
    report_progress(session, SyncStep::Mailboxes, total, total);
    Ok(all)
}
