#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use mira_vocabulary::{
    AccountId, ActionId, Event, EventCategory, EventKind, EventSeq, LogLevel, MailboxId, MessageId,
    OperationId, ThreadId, Timestamp,
};

/// Fabrique le compte `a1` utilisé par les cas de test.
fn acc() -> AccountId {
    AccountId::from_raw("a1")
}

/// Les événements du moteur, des comptes, de la synchronisation et des boîtes ont leur catégorie.
#[test]
fn category_of_engine_account_sync_and_mailbox_events() {
    assert_eq!(EventKind::EngineStarted.category(), EventCategory::Engine);
    let op = EventKind::OperationFinished {
        operation: OperationId::from_raw("o"),
    };
    assert_eq!(op.category(), EventCategory::Engine);
    let added = EventKind::AccountAdded { account: acc() };
    assert_eq!(added.category(), EventCategory::Account);
    let offline = EventKind::WentOffline { account: acc() };
    assert_eq!(offline.category(), EventCategory::Sync);
    let mailbox = MailboxId::from_raw("m");
    let created = EventKind::MailboxCreated {
        account: acc(),
        mailbox,
    };
    assert_eq!(created.category(), EventCategory::Mailbox);
}

/// Les événements de message, de fil et d'action ont leur catégorie.
#[test]
fn category_of_message_thread_and_action_events() {
    let message = MessageId::from_raw("x");
    let body = EventKind::BodyAvailable {
        account: acc(),
        message,
    };
    assert_eq!(body.category(), EventCategory::Message);
    let thread = EventKind::ThreadUpdated {
        account: acc(),
        thread: ThreadId::from_raw("t"),
    };
    assert_eq!(thread.category(), EventCategory::Thread);
    let action = ActionId::from_raw("q");
    let queued = EventKind::ActionQueued {
        account: acc(),
        action,
    };
    assert_eq!(queued.category(), EventCategory::Action);
}

/// Les événements d'envoi, d'alerte et de journal ont leur catégorie.
#[test]
fn category_of_send_alert_and_log_events() {
    let action = ActionId::from_raw("q");
    let uncertain = EventKind::SendUncertain {
        account: acc(),
        action,
    };
    assert_eq!(uncertain.category(), EventCategory::Send);
    let quota = EventKind::QuotaFull { account: acc() };
    assert_eq!(quota.category(), EventCategory::Alert);
    let level = LogLevel::Warn;
    let log = EventKind::Diagnostic {
        level,
        target: "sync".into(),
        text: "lent".into(),
    };
    assert_eq!(log.category(), EventCategory::Log);
}

/// Seuls les événements rattachés à un compte rendent ce compte.
#[test]
fn account_is_present_only_for_account_events() {
    let limited = EventKind::RateLimited { account: acc() };
    assert_eq!(limited.account(), Some(&acc()));
    assert_eq!(EventKind::EngineStopping.account(), None);
    let operation = EventKind::OperationCancelled {
        operation: OperationId::from_raw("o"),
    };
    assert_eq!(operation.account(), None);
}

/// La sérialisation JSON est taguée par `type` en `snake_case` et ne change pas.
#[test]
fn event_json_is_tagged_and_stable() {
    let kind = EventKind::SyncFailed {
        account: acc(),
        operation: OperationId::from_raw("op1"),
        code: "network.timeout".to_string(),
    };
    let event = Event {
        seq: EventSeq(7),
        at: Timestamp::from_millis(0),
        kind,
    };
    let json = serde_json::to_value(&event).unwrap();
    let kind_json = serde_json::json!({
        "type": "sync_failed", "account": "a1", "operation": "op1", "code": "network.timeout"
    });
    assert_eq!(
        json,
        serde_json::json!({"seq": 7, "at": 0, "kind": kind_json})
    );
    assert_eq!(serde_json::from_value::<Event>(json).unwrap(), event);
}
