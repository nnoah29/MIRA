#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Décision de notifier : réglages, verdict du moteur, compte en sourdine et présence de l'app.

use mira_assembly::mira_vocabulary::MailboxRole;
use mira_assembly::mira_vocabulary::{AccountId, EngineSettings};
use mira_host::notifier::{Decision, decide, worth_notifying};

/// Réglages par défaut modifiés par `change`.
fn settings(change: impl FnOnce(&mut EngineSettings)) -> EngineSettings {
    let mut settings = EngineSettings::default();
    change(&mut settings);
    settings
}

/// Par défaut, un message non silencieux est notifié quand personne ne regarde.
#[test]
fn a_plain_arrival_is_shown_when_nobody_is_looking() {
    let account = AccountId::from_raw("a1");
    let verdict = decide(&EngineSettings::default(), false, &account, false, 12);
    assert_eq!(verdict, Decision::Show);
}

/// Notifications désactivées ou verdict silencieux du moteur : rien.
#[test]
fn disabled_or_silenced_arrivals_are_skipped() {
    let account = AccountId::from_raw("a1");
    let off = settings(|s| s.notifications.enabled = false);
    assert_eq!(decide(&off, false, &account, false, 12), Decision::Skip);
    let verdict = decide(&EngineSettings::default(), true, &account, false, 12);
    assert_eq!(verdict, Decision::Skip);
}

/// Un compte en sourdine n'est jamais notifié, même si le moteur n'a pas posé le drapeau.
#[test]
fn a_muted_account_is_skipped() {
    let account = AccountId::from_raw("a1");
    let muted = settings(|s| s.muted_accounts = vec![account.clone()]);
    assert_eq!(decide(&muted, false, &account, false, 12), Decision::Skip);
}

/// App au premier plan : rien, sauf si le réglage `when_focused` le demande.
#[test]
fn a_focused_app_blocks_unless_when_focused_is_set() {
    let account = AccountId::from_raw("a1");
    let default = EngineSettings::default();
    assert_eq!(decide(&default, false, &account, true, 12), Decision::Skip);
    let always = settings(|s| s.notifications.when_focused = true);
    assert_eq!(decide(&always, false, &account, true, 12), Decision::Show);
}

/// Seul un message non lu de la réception mérite une notification.
#[test]
fn only_unread_inbox_messages_are_worth_a_notification() {
    assert!(worth_notifying(MailboxRole::Inbox, false));
    assert!(!worth_notifying(MailboxRole::Inbox, true));
    for role in [
        MailboxRole::Sent,
        MailboxRole::Drafts,
        MailboxRole::Trash,
        MailboxRole::Junk,
        MailboxRole::Archive,
        MailboxRole::All,
        MailboxRole::Other,
    ] {
        assert!(!worth_notifying(role, false), "{role:?}");
    }
}
