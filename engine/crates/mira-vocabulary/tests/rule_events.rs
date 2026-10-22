#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Événements des règles : catégorie, compte rattaché, forme JSON, champ `silenced` de `NewMessageReceived`.

use mira_vocabulary::{AccountId, EventCategory, EventKind, MailboxId, MessageId, RuleId};

/// Les événements de règle sont de la catégorie `rule` ; seul `RuleApplied` désigne un compte
#[test]
fn rule_events_have_rule_category_and_account_only_when_applied() {
    let changed = EventKind::RuleChanged {
        rule: RuleId::from_raw("r"),
    };
    assert_eq!(changed.category(), EventCategory::Rule);
    assert_eq!(changed.account(), None);
    let applied = EventKind::RuleApplied {
        account: AccountId::from_raw("a"),
        rule: RuleId::from_raw("r"),
        message: MessageId::from_raw("m"),
    };
    assert_eq!(applied.category(), EventCategory::Rule);
    assert_eq!(applied.account(), Some(&AccountId::from_raw("a")));
    let category = serde_json::to_value(EventCategory::Rule).unwrap();
    assert_eq!(category, "rule");
}

/// Les événements de règle portent des identifiants seulement, étiquetés en `snake_case`
#[test]
fn rule_events_json_shape() {
    let applied = EventKind::RuleApplied {
        account: AccountId::from_raw("a"),
        rule: RuleId::from_raw("r"),
        message: MessageId::from_raw("m"),
    };
    let value = serde_json::to_value(&applied).unwrap();
    let expected =
        serde_json::json!({"type":"rule_applied","account":"a","rule":"r","message":"m"});
    assert_eq!(value, expected);
    let changed = EventKind::RuleChanged {
        rule: RuleId::from_raw("r"),
    };
    let value = serde_json::to_value(&changed).unwrap();
    assert_eq!(value, serde_json::json!({"type":"rule_changed","rule":"r"}));
}

/// Un `NewMessageReceived` enregistré avant l'ajout de `silenced` se relit avec `silenced` faux
#[test]
fn new_message_received_silenced_defaults_to_false() {
    let old = r#"{"type":"new_message_received","account":"a","message":"m","mailbox":"b"}"#;
    let kind: EventKind = serde_json::from_str(old).unwrap();
    let expected = EventKind::NewMessageReceived {
        account: AccountId::from_raw("a"),
        message: MessageId::from_raw("m"),
        mailbox: MailboxId::from_raw("b"),
        silenced: false,
    };
    assert_eq!(kind, expected);
    let silenced = EventKind::NewMessageReceived {
        account: AccountId::from_raw("a"),
        message: MessageId::from_raw("m"),
        mailbox: MailboxId::from_raw("b"),
        silenced: true,
    };
    let value = serde_json::to_value(&silenced).unwrap();
    assert_eq!(value["silenced"], true);
}
