#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Boucle d'outils et actions sur la boîte : ordre des deltas, aucune écriture, journal sans contenu.

mod ai_loop_support;
mod ai_models_support;
mod support;
mod support_api;
mod support_mirror;
mod support_read;

use ai_loop_support::fixture::LoopFixture;
use ai_loop_support::turns::{FORMATS, collect, shape, text_turn, tool_results, tool_turn};
use mira_core::api::{AiRequest, AiResponse, Request, Response};
use mira_vocabulary::{AccountId, ChatDelta, MailActionKind};

const ARCHIVE: &str = r#"{"action":"archive","messages":["m1","m2"],"reason":"RAISON-SECRETE"}"#;

/// La proposition arrive entre l'annonce et la fin de l'appel ; le modèle apprend que l'utilisateur confirmera
#[tokio::test]
async fn action_arrives_between_called_and_finished() {
    for format in FORMATS {
        let fixture = LoopFixture::start(format).await;
        fixture.queue(tool_turn(
            format,
            "",
            &[("c1", "propose_mailbox_action", ARCHIVE)],
            None,
        ));
        fixture.queue(text_turn(format, "Je te laisse confirmer.", None));
        let mut stream = fixture.ask("archive mes newsletters").await;
        let deltas = collect(&mut stream).await;
        let expected = [
            "called:propose_mailbox_action",
            "action",
            "finished:ok",
            "text",
            "done:end_turn",
        ];
        assert_eq!(shape(&deltas), expected, "{format:?}");
        let ChatDelta::ActionProposed { id, proposal } = &deltas[1] else {
            panic!("action attendue");
        };
        assert_eq!(id, "c1");
        assert_eq!(proposal.action, MailActionKind::Archive);
        assert_eq!(proposal.messages, ["m1", "m2"]);
        let results = tool_results(&fixture.bodies()[1], format);
        assert!(results[0].contains("confirm"), "{results:?}");
    }
}

/// Rien n'est modifié dans la boîte, et le journal garde le nom de l'outil sans la raison
#[tokio::test]
async fn nothing_changes_and_the_journal_has_no_content() {
    let format = FORMATS[1];
    let fixture = LoopFixture::start(format).await;
    fixture.queue(tool_turn(
        format,
        "",
        &[("c1", "propose_mailbox_action", ARCHIVE)],
        None,
    ));
    fixture.queue(text_turn(format, "Fait.", None));
    let mut stream = fixture.ask("archive").await;
    collect(&mut stream).await;
    let account = AccountId::from_raw("acc-1");
    let queued = fixture
        .db
        .read(|tx| tx.list_actions(&account, &[]))
        .unwrap();
    assert!(queued.is_empty(), "{queued:?}");
    let list = Request::Ai(AiRequest::ListAiJournal {
        limit: 10,
        before: None,
    });
    let Response::Ai(AiResponse::Journal(rows)) = fixture.engine.execute(list).await.unwrap()
    else {
        panic!("journal attendu");
    };
    let text = serde_json::to_string(&rows).unwrap();
    assert!(text.contains("propose_mailbox_action"), "{text}");
    assert!(!text.contains("RAISON-SECRETE"), "{text}");
}
