#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Boucle d'outils et brouillons : ordre des deltas, jeu d'outils du mode, outil hors mode, journal sans contenu.

mod ai_loop_support;
mod ai_models_support;
mod support;
mod support_api;
mod support_mirror;
mod support_read;

use ai_loop_support::fixture::LoopFixture;
use ai_loop_support::turns::{FORMATS, collect, shape, text_turn, tool_results, tool_turn};
use mira_core::api::{AiRequest, AiResponse, Request, Response};
use mira_vocabulary::{ChatDelta, ChatMode, DraftProposalKind, WireFormat};
use serde_json::Value;

const COMPOSE: &str = r#"{"to":["boss@example.org"],"subject":"SECRET-SUBJECT","body":"Bonjour, mon stage finit mercredi."}"#;
const EDIT: &str = r#"{"body":"Nouveau texte"}"#;

/// Noms des outils annoncés dans un corps de requête, quel que soit le format
fn announced(body: &Value, format: WireFormat) -> Vec<String> {
    let tools = body["tools"].as_array().cloned().unwrap_or_default();
    let pointer = match format {
        WireFormat::AnthropicMessages | WireFormat::OpenAiResponses => "/name",
        WireFormat::OpenAiChat => "/function/name",
    };
    let name = |tool: &Value| {
        tool.pointer(pointer)
            .and_then(Value::as_str)
            .map(str::to_string)
    };
    tools.iter().filter_map(name).collect()
}

/// Lignes du journal des envois, en texte
async fn journal_text(fixture: &LoopFixture) -> String {
    let list = Request::Ai(AiRequest::ListAiJournal {
        limit: 10,
        before: None,
    });
    let Response::Ai(AiResponse::Journal(rows)) = fixture.engine.execute(list).await.unwrap()
    else {
        panic!("journal attendu");
    };
    serde_json::to_string(&rows).unwrap()
}

/// La proposition arrive entre l'annonce et la fin de l'appel, et le modèle reçoit la phrase fixe
#[tokio::test]
async fn proposal_arrives_between_called_and_finished() {
    for format in FORMATS {
        let fixture = LoopFixture::start(format).await;
        fixture.queue(tool_turn(
            format,
            "",
            &[("c1", "compose_draft", COMPOSE)],
            None,
        ));
        fixture.queue(text_turn(format, "C'est prêt.", None));
        let mut stream = fixture.ask("écris un mail à mon boss").await;
        let deltas = collect(&mut stream).await;
        let expected = [
            "called:compose_draft",
            "proposed",
            "finished:ok",
            "text",
            "done:end_turn",
        ];
        assert_eq!(shape(&deltas), expected, "{format:?}");
        let ChatDelta::DraftProposed { id, proposal } = &deltas[1] else {
            panic!("proposition attendue");
        };
        assert_eq!(id, "c1");
        assert_eq!(proposal.kind, DraftProposalKind::New);
        assert_eq!(proposal.subject.as_deref(), Some("SECRET-SUBJECT"));
        let results = tool_results(&fixture.bodies()[1], format);
        assert!(results[0].contains("relira"), "{results:?}");
    }
}

/// Le mode composeur annonce `edit_draft` et pas `compose_draft` ; l'assistant l'inverse
#[tokio::test]
async fn announced_tools_follow_the_mode() {
    for format in FORMATS {
        let fixture = LoopFixture::start(format).await;
        fixture.queue(text_turn(format, "Ok.", None));
        let mut start = fixture.start_of("raccourcis", true);
        start.mode = ChatMode::Composer { selection: false };
        let mut stream = fixture.engine.chat_stream_with_tools(start).await.unwrap();
        collect(&mut stream).await;
        fixture.queue(text_turn(format, "Ok.", None));
        let mut stream = fixture.ask("écris un mail").await;
        collect(&mut stream).await;
        let bodies = fixture.bodies();
        let composer = announced(&bodies[0], format);
        assert!(composer.contains(&"edit_draft".to_string()), "{composer:?}");
        assert!(
            !composer.contains(&"compose_draft".to_string()),
            "{composer:?}"
        );
        let assistant = announced(&bodies[1], format);
        assert!(
            assistant.contains(&"compose_draft".to_string()),
            "{assistant:?}"
        );
        assert!(
            !assistant.contains(&"edit_draft".to_string()),
            "{assistant:?}"
        );
    }
}

/// Un outil hors de son mode échoue sans proposition, et le modèle est prévenu
#[tokio::test]
async fn tool_outside_its_mode_fails_without_a_proposal() {
    for format in FORMATS {
        let fixture = LoopFixture::start(format).await;
        fixture.queue(tool_turn(format, "", &[("c1", "edit_draft", EDIT)], None));
        fixture.queue(text_turn(format, "Je ne peux pas.", None));
        let mut stream = fixture.ask("modifie").await;
        let deltas = collect(&mut stream).await;
        let expected = [
            "called:edit_draft",
            "finished:error",
            "text",
            "done:end_turn",
        ];
        assert_eq!(shape(&deltas), expected, "{format:?}");
        let results = tool_results(&fixture.bodies()[1], format);
        assert!(results[0].contains("indisponible"), "{results:?}");
    }
}

/// Un brouillon invalide échoue sans proposition ; le journal ne garde ni objet ni corps
#[tokio::test]
async fn invalid_draft_and_journal_without_content() {
    for format in FORMATS {
        let fixture = LoopFixture::start(format).await;
        let bad = r#"{"to":["pas-une-adresse"],"subject":"SECRET-SUBJECT","body":"x"}"#;
        fixture.queue(tool_turn(format, "", &[("c1", "compose_draft", bad)], None));
        fixture.queue(tool_turn(
            format,
            "",
            &[("c2", "compose_draft", COMPOSE)],
            None,
        ));
        fixture.queue(text_turn(format, "Fait.", None));
        let mut stream = fixture.ask("écris").await;
        let found = shape(&collect(&mut stream).await);
        assert_eq!(found[1], "finished:error", "{format:?}");
        assert!(found.contains(&"proposed".to_string()), "{found:?}");
        let text = journal_text(&fixture).await;
        assert!(text.contains("compose_draft"), "{text}");
        assert!(
            !text.contains("SECRET-SUBJECT") && !text.contains("Bonjour"),
            "{text}"
        );
    }
}
