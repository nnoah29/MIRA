#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Boucle d'outils : un tour, plusieurs appels, enchaînement, consommation cumulée, sources, sans outils.

mod ai_loop_support;
mod ai_models_support;
mod support;
mod support_api;
mod support_mirror;
mod support_read;

use ai_loop_support::fixture::LoopFixture;
use ai_loop_support::turns::{FORMATS, collect, shape, text_turn, tool_turn};
use mira_vocabulary::{ChatDelta, ToolState};

const SEARCH: &str = r#"{"query":"devis"}"#;

/// Un appel d'outil, puis la réponse : l'ordre des événements et le résultat renvoyé au modèle
#[tokio::test]
async fn one_tool_round_then_final_text() {
    for format in FORMATS {
        let fixture = LoopFixture::start(format).await;
        fixture.queue(tool_turn(
            format,
            "Je cherche.",
            &[("c1", "search_messages", SEARCH)],
            None,
        ));
        fixture.queue(text_turn(format, "Voici le devis.", None));
        let mut stream = fixture.ask("Trouve le devis").await;
        let deltas = collect(&mut stream).await;
        let expected = [
            "text",
            "called:search_messages",
            "finished:ok",
            "text",
            "done:end_turn",
        ];
        assert_eq!(shape(&deltas), expected, "{format:?}");
        let second = &fixture.bodies()[1];
        let results = ai_loop_support::turns::tool_results(second, format);
        assert_eq!(results.len(), 1);
        assert!(results[0].starts_with("<tool_result name=\"search_messages\">"));
        assert!(results[0].contains("id=m2"));
    }
}

/// Deux appels dans un tour s'exécutent dans l'ordre et reviennent dans l'ordre
#[tokio::test]
async fn two_calls_in_one_turn_run_in_order() {
    for format in FORMATS {
        let fixture = LoopFixture::start(format).await;
        let calls = [
            ("c1", "search_messages", SEARCH),
            ("c2", "list_mailboxes", "{}"),
        ];
        fixture.queue(tool_turn(format, "", &calls, None));
        fixture.queue(text_turn(format, "Fini.", None));
        let mut stream = fixture.ask("Fais les deux").await;
        let shape = shape(&collect(&mut stream).await);
        let expected = [
            "called:search_messages",
            "finished:ok",
            "called:list_mailboxes",
            "finished:ok",
            "text",
            "done:end_turn",
        ];
        assert_eq!(shape, expected, "{format:?}");
        let results = ai_loop_support::turns::tool_results(&fixture.bodies()[1], format);
        assert!(results[0].contains("name=\"search_messages\""));
        assert!(results[1].contains("name=\"list_mailboxes\""));
    }
}

/// Une recherche puis la lecture du fil trouvé : trois requêtes au modèle
#[tokio::test]
async fn search_then_get_thread_chain() {
    for format in FORMATS {
        let fixture = LoopFixture::start(format).await;
        fixture.queue(tool_turn(
            format,
            "",
            &[("c1", "search_messages", SEARCH)],
            None,
        ));
        fixture.queue(tool_turn(
            format,
            "",
            &[("c2", "get_thread", r#"{"thread":"t-r1"}"#)],
            None,
        ));
        fixture.queue(text_turn(format, "Voilà.", None));
        let mut stream = fixture.ask("Que s'est-il dit ?").await;
        let deltas = collect(&mut stream).await;
        assert_eq!(fixture.bodies().len(), 3, "{format:?}");
        let finished = deltas
            .iter()
            .filter(|d| matches!(d, ChatDelta::ToolFinished { .. }));
        assert_eq!(finished.count(), 2);
        let last = ai_loop_support::turns::tool_results(&fixture.bodies()[2], format);
        assert!(last[1].contains("le devis de mars"));
    }
}

/// La consommation des tours s'additionne dans le total remis au lecteur
#[tokio::test]
async fn usage_is_summed_across_rounds() {
    for format in FORMATS {
        let fixture = LoopFixture::start(format).await;
        fixture.queue(tool_turn(
            format,
            "",
            &[("c1", "search_messages", SEARCH)],
            Some((10, 5)),
        ));
        fixture.queue(text_turn(format, "Fini.", Some((20, 7))));
        let mut stream = fixture.ask("Cherche").await;
        let deltas = collect(&mut stream).await;
        let usage = ChatDelta::Usage {
            input_tokens: 30,
            output_tokens: 12,
        };
        assert!(deltas.contains(&usage), "{format:?} {deltas:?}");
        let rows = fixture.db.read(|tx| tx.list_ai_journal(1, None)).unwrap();
        assert_eq!(
            (rows[0].input_tokens, rows[0].output_tokens),
            (Some(30), Some(12))
        );
    }
}

/// Un identifiant cité par le modèle mais absent de la boîte donne une erreur d'outil et aucune source
#[tokio::test]
async fn unknown_message_cited_by_the_model_has_no_source() {
    for format in FORMATS {
        let fixture = LoopFixture::start(format).await;
        let call = [("c1", "get_message", r#"{"message":"m99"}"#)];
        fixture.queue(tool_turn(format, "", &call, None));
        fixture.queue(text_turn(format, "Introuvable.", None));
        let mut stream = fixture.ask("Lis m99").await;
        let deltas = collect(&mut stream).await;
        let finished = deltas.iter().find_map(|delta| match delta {
            ChatDelta::ToolFinished { state, sources, .. } => Some((*state, sources.clone())),
            _ => None,
        });
        assert_eq!(finished, Some((ToolState::Error, Vec::new())), "{format:?}");
    }
}

/// Sans outils permis, rien n'est annoncé au modèle, même avec la méthode qui les supporte
#[tokio::test]
async fn tools_are_never_announced_unless_allowed() {
    for format in FORMATS {
        let fixture = LoopFixture::start(format).await;
        fixture.queue(text_turn(format, "Bonjour.", None));
        fixture.queue(text_turn(format, "Bonjour.", None));
        let plain = fixture.start_of("Salut", true);
        let mut stream = fixture.engine.chat_stream(plain).await.unwrap();
        collect(&mut stream).await;
        let refused = fixture.start_of("Salut", false);
        let mut stream = fixture
            .engine
            .chat_stream_with_tools(refused)
            .await
            .unwrap();
        collect(&mut stream).await;
        for body in fixture.bodies() {
            assert!(body.get("tools").is_none(), "{format:?}");
        }
    }
}
