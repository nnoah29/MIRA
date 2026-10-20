//! Fusion de fin de passe : un message déplacé ailleurs, arrivé avant que sa source soit vue disparaître, reprend son identité.

use mira_vocabulary::{AccountId, EventKind, MessageId, MiraError};

use crate::storage::Tx;
use crate::sync::context::SyncContext;
use crate::sync::identity::apply_server_flags;
use crate::sync::strategy::updated;

/// Fusionne chaque orphelin de la passe avec son jumeau créé pendant la passe, et annonce le résultat
pub fn merge_moved_twins(ctx: &SyncContext, created: &[MessageId]) -> Result<(), MiraError> {
    if created.is_empty() {
        return Ok(());
    }
    let merged = ctx
        .db
        .write(|tx| merge_all(tx, ctx.account_id(), created))?;
    for (kept, twin) in merged {
        let account = ctx.account.id.clone();
        ctx.emit(EventKind::MessageDeleted {
            account,
            message: twin,
        });
        ctx.emit(updated(&ctx.account.id, &kept));
    }
    Ok(())
}

/// Fusionne tous les couples (orphelin, jumeau unique créé pendant la passe) ; rend (conservé, jumeau)
fn merge_all(
    tx: &Tx<'_>,
    account: &AccountId,
    created: &[MessageId],
) -> Result<Vec<(MessageId, MessageId)>, MiraError> {
    let mut available = created.to_vec();
    let mut merged = Vec::new();
    for orphan in tx.orphan_messages(account)? {
        let Some(twin) = twin_of(tx, account, &orphan, &available)? else {
            continue;
        };
        let flags = tx.get_message(&twin)?.map(|summary| summary.flags);
        if let Some(flags) = flags {
            apply_server_flags(tx, &orphan, flags)?;
        }
        tx.merge_twin_into(&twin, &orphan)?;
        available.retain(|id| id != &twin);
        merged.push((orphan, twin));
    }
    Ok(merged)
}

/// Le seul message créé pendant la passe qui a l'empreinte (Message-ID, date, taille) de l'orphelin
fn twin_of(
    tx: &Tx<'_>,
    account: &AccountId,
    orphan: &MessageId,
    created: &[MessageId],
) -> Result<Option<MessageId>, MiraError> {
    let Some(summary) = tx.get_message(orphan)? else {
        return Ok(None);
    };
    let Some(header) = summary.message_id_header.as_deref() else {
        return Ok(None);
    };
    let candidates = tx.find_by_fingerprint(account, header, summary.date, summary.size)?;
    let mut twins = candidates
        .into_iter()
        .filter(|id| id != orphan && created.contains(id));
    Ok(match (twins.next(), twins.next()) {
        (Some(only), None) => Some(only),
        _ => None,
    })
}
