#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Réglages de notification par la façade : comptes en sourdine appliqués par le moteur, heures calmes, normalisation.

mod support_api;
mod support_settings;

use mira_vocabulary::{AccountId, EngineSettings, EventKind, QuietHours};
use support_api::fixture;
use support_api::sync_calls::wait_for_events;
use support_api::synced::synced_account;
use support_api::syncing::deliver_one;
use support_settings::calls::{announced_keys, current, replace};

/// Réglages qui mettent ces comptes en sourdine
fn muting(accounts: &[&AccountId]) -> EngineSettings {
    EngineSettings {
        muted_accounts: accounts.iter().map(|id| (*id).clone()).collect(),
        ..EngineSettings::default()
    }
}

/// Verdicts « silencieux » des arrivées annoncées jusqu'ici, dans l'ordre
fn verdicts(kinds: &[EventKind]) -> Vec<bool> {
    let silenced = |kind: &EventKind| match kind {
        EventKind::NewMessageReceived { silenced, .. } => Some(*silenced),
        _ => None,
    };
    kinds.iter().filter_map(silenced).collect()
}

/// Un compte en sourdine voit ses arrivées annoncées « silencieuses » tout de suite, et plus après le rétablissement
#[tokio::test]
async fn muted_account_arrivals_are_silenced_until_unmuted() {
    let (f, account) = synced_account().await;
    replace(&f.engine, muting(&[&account])).await.unwrap();
    deliver_one(&f, "INBOX", "Pendant la sourdine");
    wait_for_events(&f.engine, |k| verdicts(k) == [true]).await;
    replace(&f.engine, EngineSettings::default()).await.unwrap();
    deliver_one(&f, "INBOX", "Après la sourdine");
    wait_for_events(&f.engine, |k| verdicts(k) == [true, false]).await;
}

/// Les heures calmes sont gardées, annoncées, et refusées hors d'une journée
#[tokio::test]
async fn quiet_hours_are_stored_announced_and_bounded() {
    let f = fixture::start().await;
    let quiet = |from, to| EngineSettings {
        quiet_hours: Some(QuietHours { from, to }),
        ..EngineSettings::default()
    };
    replace(&f.engine, quiet(22, 7)).await.unwrap();
    assert_eq!(current(&f.engine).await, quiet(22, 7));
    assert_eq!(announced_keys(&f.engine).await, [["quiet_hours"]]);
    let code = replace(&f.engine, quiet(24, 7)).await.unwrap_err();
    assert_eq!(code, "invalid_input.parameter");
    assert_eq!(current(&f.engine).await, quiet(22, 7));
}

/// Les comptes en sourdine sont rendus triés et sans doublon, et le même ensemble redemandé n'annonce rien
#[tokio::test]
async fn muted_accounts_are_sorted_deduplicated_and_announced_once() {
    let f = fixture::start().await;
    let (a, b) = (AccountId::from_raw("acc-a"), AccountId::from_raw("acc-b"));
    let stored = replace(&f.engine, muting(&[&b, &a, &b])).await.unwrap();
    assert_eq!(stored.muted_accounts, [a.clone(), b.clone()]);
    replace(&f.engine, muting(&[&a, &b])).await.unwrap();
    assert_eq!(announced_keys(&f.engine).await, [["muted_accounts"]]);
}
