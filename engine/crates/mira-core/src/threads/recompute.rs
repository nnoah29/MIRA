//! Recalcul complet des fils d'un compte : la table est dérivée, rien n'est perdu (invariant 13).

use std::collections::HashMap;

use mira_vocabulary::{AccountId, MessageSummary, MiraError, ThreadId};

use super::assign::assign_thread;
use super::key::thread_key;
use crate::ports::IdGenerator;
use crate::storage::{Database, Tx};

const BATCH: u32 = 500;

/// Efface et recalcule les fils d'un compte en une transaction ; rend le nombre de messages rattachés
pub fn recompute_threads(
    db: &Database,
    ids: &dyn IdGenerator,
    account: &AccountId,
) -> Result<usize, MiraError> {
    db.write(|tx| {
        let known: HashMap<String, ThreadId> =
            tx.thread_ids_by_root_key(account)?.into_iter().collect();
        let hints: HashMap<String, String> =
            tx.thread_hints_of_messages(account)?.into_iter().collect();
        tx.clear_threads(account)?;
        reassign_all(tx, ids, account, &known, &hints)
    })
}

/// Rattache tous les messages du compte, du plus ancien au plus récent, par lots
fn reassign_all(
    tx: &Tx<'_>,
    ids: &dyn IdGenerator,
    account: &AccountId,
    known: &HashMap<String, ThreadId>,
    hints: &HashMap<String, String>,
) -> Result<usize, MiraError> {
    let mut after: Option<(i64, String)> = None;
    let mut count = 0;
    loop {
        let position = after.as_ref().map(|(date, id)| (*date, id.as_str()));
        let batch = tx.messages_for_threading(account, position, BATCH)?;
        let Some((date, id, _)) = batch.last() else {
            return Ok(count);
        };
        after = Some((*date, id.clone()));
        count += batch.len();
        for (_, _, summary) in &batch {
            reassign_one(tx, ids, summary, known, hints)?;
        }
    }
}

/// Rattache un message en réutilisant l'ancien identifiant de son fil quand la clé est connue
fn reassign_one(
    tx: &Tx<'_>,
    ids: &dyn IdGenerator,
    summary: &MessageSummary,
    known: &HashMap<String, ThreadId>,
    hints: &HashMap<String, String>,
) -> Result<(), MiraError> {
    let hint = hints.get(summary.id.as_str()).map(String::as_str);
    let candidate = known
        .get(&thread_key(summary, hint))
        .cloned()
        .unwrap_or_else(|| ThreadId::from_raw(ids.next_id()));
    assign_thread(tx, &candidate, summary, hint)?;
    Ok(())
}
