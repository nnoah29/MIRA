//! Réévaluation des libellés système d'un compte, dans la transaction de l'appelant.

use mira_vocabulary::{AccountId, MiraError, SystemLabelKind, ThreadId};

use super::classify::classify;
use super::facts::last_live_message;
use super::me::{Me, load_me};
use crate::storage::{SystemState, Tx};

const BATCH: u32 = 500;

/// Réévalue les conversations du compte qui en ont besoin ; vrai si un libellé a changé
pub fn refresh_account_labels(
    tx: &Tx<'_>,
    account: &AccountId,
    now: i64,
) -> Result<bool, MiraError> {
    let me = load_me(tx, account)?;
    let mut changed = false;
    for thread in tx.threads_needing_labels(account, now, BATCH)? {
        changed |= refresh_thread(tx, &thread, &me, now)?;
    }
    Ok(changed)
}

/// Réévalue une conversation et note quand son verdict pourra changer sans nouveau message
fn refresh_thread(tx: &Tx<'_>, thread: &ThreadId, me: &Me, now: i64) -> Result<bool, MiraError> {
    let Some((account, key)) = tx.thread_root_key(thread)? else {
        return Ok(false);
    };
    let found = last_live_message(tx, thread, me)?;
    let verdict = classify(found.as_ref().map(|(_, last)| last), me, now);
    let latest = found.map_or(0, |(date, _)| date);
    let waiting = apply_detected(
        tx,
        &account,
        &key,
        SystemLabelKind::Waiting,
        (verdict.waiting, latest),
    )?;
    let mention = apply_detected(
        tx,
        &account,
        &key,
        SystemLabelKind::Mention,
        (verdict.mention, latest),
    )?;
    tx.settle_thread_labels(thread, verdict.recheck_at)?;
    Ok(waiting || mention)
}

/// Écrit le verdict d'une sorte sauf si un choix manuel plus récent que le dernier message le protège
fn apply_detected(
    tx: &Tx<'_>,
    account: &AccountId,
    key: &str,
    kind: SystemLabelKind,
    (detected, latest): (bool, i64),
) -> Result<bool, MiraError> {
    let state = tx.system_label_state(account, key, kind)?;
    if state.is_some_and(|s| s.manual && s.as_of >= latest) {
        return Ok(false);
    }
    let was_on = state.is_some_and(|s| s.on);
    if detected {
        let auto = SystemState {
            on: true,
            manual: false,
            as_of: 0,
        };
        tx.put_system_label_state(account, key, kind, auto)?;
    } else {
        tx.delete_system_label_state(account, key, kind)?;
    }
    Ok(was_on != detected)
}
