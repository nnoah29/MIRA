#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Contrat 1.2 : lecture paginée du journal des envois et vidage, sans aucun texte dans les lignes.

mod ai_chat_support;
mod ai_models_support;
mod support_ai_api;
mod support_api;

use ai_chat_support::{chat_start, connection_with, run_chat};
use ai_models_support::{ModelsFixture, TestProvider};
use mira_core::api::{AiRequest, AiResponse, Request, Response};
use mira_testkit::ai::openai_stream;
use mira_vocabulary::MaskingSettings;
use support_ai_api::privacy::journal;

/// Fait trois générations et rend le moteur prêt à lire le journal
async fn after_three_chats() -> ModelsFixture {
    let fixture = ModelsFixture::start(TestProvider::new("test-provider")).await;
    let connection = connection_with(&fixture, &MaskingSettings::remote_default());
    for text in ["un", "deux", "trois"] {
        fixture.server.queue(openai_stream(&["ok"], Some((3, 1))));
        run_chat(&fixture, chat_start(&connection, text)).await;
    }
    fixture
}

/// Après des discussions, le journal les liste, la plus récente d'abord, sans texte
#[tokio::test]
async fn list_journal_after_a_chat() {
    let f = after_three_chats().await;
    let rows = journal(&f.engine, 50, None).await.unwrap();
    assert_eq!(rows.len(), 3);
    assert!(rows.iter().all(|row| row.outcome == "end_turn"));
    assert!(rows.windows(2).all(|pair| pair[0].at >= pair[1].at));
    let text = serde_json::to_string(&rows).unwrap();
    for prompt in ["un", "deux", "trois", "ok"] {
        assert!(!text.contains(&format!("\"{prompt}\"")), "{text}");
    }
}

/// Une limite nulle ou supérieure à 200 est refusée, 1 et 200 passent
#[tokio::test]
async fn limit_zero_or_over_200_is_refused() {
    let f = after_three_chats().await;
    for limit in [0, 201, u32::MAX] {
        let refused = journal(&f.engine, limit, None).await;
        assert_eq!(refused.unwrap_err(), "invalid_input.ai_journal_limit");
    }
    assert_eq!(journal(&f.engine, 1, None).await.unwrap().len(), 1);
    assert_eq!(journal(&f.engine, 200, None).await.unwrap().len(), 3);
}

/// Le curseur `before` reprend après la ligne donnée, page par page
#[tokio::test]
async fn before_cursor_pages() {
    let f = after_three_chats().await;
    let first = journal(&f.engine, 2, None).await.unwrap();
    assert_eq!(first.len(), 2);
    let second = journal(&f.engine, 2, Some(&first[1].id)).await.unwrap();
    assert_eq!(second.len(), 1);
    let all = journal(&f.engine, 10, None).await.unwrap();
    assert_eq!(all[2].id, second[0].id);
    let unknown = journal(&f.engine, 2, Some("inconnu")).await.unwrap();
    assert!(unknown.is_empty());
}

/// Vider le journal répond `Done` et ne laisse aucune ligne
#[tokio::test]
async fn clear_journal_empties_it() {
    let f = after_three_chats().await;
    let answer = f
        .engine
        .execute(Request::Ai(AiRequest::ClearAiJournal))
        .await;
    assert_eq!(answer.unwrap(), Response::Ai(AiResponse::Done));
    assert!(journal(&f.engine, 50, None).await.unwrap().is_empty());
}
