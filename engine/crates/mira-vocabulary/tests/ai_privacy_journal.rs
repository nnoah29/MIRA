#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Ligne du journal des envois d'IA : uniquement des comptes et des codes, jamais de contenu.

use mira_vocabulary::{
    AiConnectionId, AiJournalEntry, AiJournalFinish, AiProviderId, ContentKind, OperationId,
    Timestamp,
};

/// Une ligne `running` d'une génération distante avec toutes ses mesures
fn sample_entry() -> AiJournalEntry {
    AiJournalEntry {
        id: "j-1".into(),
        at: Timestamp::from_millis(5),
        operation: OperationId::from_raw("op-1"),
        connection: AiConnectionId::from_raw("c-1"),
        provider: AiProviderId("openai".into()),
        model: "m1".into(),
        local: false,
        kinds: vec![ContentKind::MailMessage],
        parts: 1,
        chars: 120,
        masked: 2,
        input_tokens: Some(3),
        output_tokens: None,
        outcome: "running".into(),
        tools: Vec::new(),
    }
}

/// Une ligne de journal n'a que des comptes et des codes : aucun champ de texte libre
#[test]
fn journal_entry_has_no_content_field() {
    let value = serde_json::to_value(sample_entry()).unwrap();
    let mut names: Vec<&str> = value
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    names.sort_unstable();
    let expected = [
        "at",
        "chars",
        "connection",
        "id",
        "input_tokens",
        "kinds",
        "local",
        "masked",
        "model",
        "operation",
        "outcome",
        "output_tokens",
        "parts",
        "provider",
        "tools",
    ];
    assert_eq!(names, expected);
    let back = serde_json::from_value::<AiJournalEntry>(value).unwrap();
    assert_eq!(back, sample_entry());
}

const FINISH_FIELDS: [&str; 7] = [
    "extra_chars",
    "extra_kinds",
    "input_tokens",
    "masked",
    "outcome",
    "output_tokens",
    "tools",
];

/// La complétion d'une ligne ne porte que des comptes et l'issue
#[test]
fn journal_finish_has_only_counts_and_outcome() {
    let finish = AiJournalFinish {
        masked: 1,
        input_tokens: Some(2),
        output_tokens: Some(3),
        outcome: "end_turn".into(),
        tools: Vec::new(),
        extra_kinds: Vec::new(),
        extra_chars: 0,
    };
    let value = serde_json::to_value(&finish).unwrap();
    let mut names: Vec<&str> = value
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    names.sort_unstable();
    assert_eq!(names, FINISH_FIELDS);
}
