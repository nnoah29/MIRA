#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Journal des envois : une ligne par génération, des comptes et jamais de contenu.

mod ai_chat_support;
mod ai_models_support;
#[allow(dead_code)]
mod support_ai_context;
mod support_api;

use ai_chat_support::{
    add_chat_connection, allow_kinds, chat_start_with, collect, give_consent, journal_dump,
    journal_rows, loopback_stream,
};
use ai_models_support::{ModelsFixture, TestProvider};
use mira_testkit::ai::openai_stream;
use mira_vocabulary::ContentKind;
use support_ai_context::part;

/// Une génération laisse une seule ligne : opération, modèle, comptes, jetons, issue et date de l'horloge
#[tokio::test]
async fn a_generation_leaves_one_journal_row_with_counts() {
    let fixture = ModelsFixture::start(TestProvider::new("test-provider")).await;
    let behavior = openai_stream(&["Bon", "jour"], Some((12, 5)));
    let mut stream = loopback_stream(&fixture, behavior, "Salut").await;
    collect(&mut stream).await;
    let rows = journal_rows(&fixture);
    assert_eq!(rows.len(), 1);
    let row = &rows[0];
    assert_eq!(row.operation, stream.operation);
    assert_eq!(
        (row.provider.0.as_str(), row.model.as_str()),
        ("test-provider", "model-a")
    );
    assert_eq!(row.connection.as_str(), "conn-1");
    assert_eq!((row.input_tokens, row.output_tokens), (Some(12), Some(5)));
    assert_eq!(
        (row.outcome.as_str(), row.chars, row.masked),
        ("end_turn", 5, 0)
    );
    assert_eq!(row.at.as_millis(), 1_700_000_000_000);
}

/// Ni le prompt, ni le contexte, ni la consigne, ni la réponse ne se retrouvent dans la table
#[tokio::test]
async fn journal_row_has_no_prompt_or_answer_text() {
    let fixture = ModelsFixture::start(TestProvider::new("test-provider")).await;
    let base = Some(fixture.server.base_url());
    let connection = add_chat_connection(&fixture, "conn-1", base, Some("model-a"));
    fixture
        .server
        .queue(openai_stream(&["REPONSE-SECRETE"], Some((1, 1))));
    let parts = vec![part(
        ContentKind::MailMessage,
        "ETIQUETTE-SECRETE",
        "CONTEXTE-SECRET",
    )];
    let mut start = chat_start_with(&connection, "PROMPT-SECRET", parts);
    start.request.system = Some("SYSTEME-SECRET".to_string());
    let mut stream = fixture.engine.chat_stream(start).await.unwrap();
    collect(&mut stream).await;
    let stored = serde_json::to_string(&journal_rows(&fixture)).unwrap() + &journal_dump(&fixture);
    for secret in [
        "PROMPT-SECRET",
        "CONTEXTE-SECRET",
        "SYSTEME-SECRET",
        "REPONSE-SECRETE",
        "ETIQUETTE-SECRETE",
    ] {
        assert!(!stored.contains(secret), "{secret}");
    }
}

/// La ligne dit si la destination est locale : vrai pour la boucle locale, faux pour un service distant
#[tokio::test]
async fn local_flag_is_set_for_loopback() {
    let fixture = ModelsFixture::start(TestProvider::new("test-provider")).await;
    let base = Some(fixture.server.base_url());
    let local = add_chat_connection(&fixture, "local", base, Some("m"));
    fixture.server.queue(openai_stream(&["ok"], None));
    let start = chat_start_with(&local, "q", vec![]);
    collect(&mut fixture.engine.chat_stream(start).await.unwrap()).await;
    let remote = add_chat_connection(
        &fixture,
        "remote",
        Some("http://ai.provider.test/v1".into()),
        Some("m"),
    );
    fixture.store_key("remote", "sk-test");
    give_consent(&fixture, &remote);
    let start = chat_start_with(&remote, "q", vec![]);
    collect(&mut fixture.engine.chat_stream(start).await.unwrap()).await;
    let flags: Vec<(String, bool)> = journal_rows(&fixture)
        .into_iter()
        .map(|row| (row.connection.as_str().to_string(), row.local))
        .collect();
    assert_eq!(
        flags,
        [("remote".to_string(), false), ("local".to_string(), true)]
    );
}

/// Les types envoyés sont distincts, dans l'ordre de première apparition ; les parties sont comptées
#[tokio::test]
async fn kinds_and_parts_are_recorded() {
    let fixture = ModelsFixture::start(TestProvider::new("test-provider")).await;
    let base = Some(fixture.server.base_url());
    let connection = add_chat_connection(&fixture, "conn-1", base, Some("m"));
    allow_kinds(&fixture, &connection, &[]);
    fixture.server.queue(openai_stream(&["ok"], None));
    let parts = vec![
        part(ContentKind::MailMessage, "A", "un"),
        part(ContentKind::Attachment, "B", "deux"),
        part(ContentKind::MailMessage, "C", "trois"),
    ];
    let start = chat_start_with(&connection, "q", parts);
    collect(&mut fixture.engine.chat_stream(start).await.unwrap()).await;
    let row = journal_rows(&fixture).remove(0);
    assert_eq!(
        row.kinds,
        [ContentKind::MailMessage, ContentKind::Attachment]
    );
    assert_eq!(row.parts, 3);
    assert!(row.chars > 3 * "<context></context>".len() as u32);
}
