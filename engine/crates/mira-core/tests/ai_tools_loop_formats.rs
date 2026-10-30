#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Boucle d'outils, formats et journal : arguments en morceaux, outils annoncés, support absent, journal sans contenu.

mod ai_loop_support;
mod ai_models_support;
mod support;
mod support_api;
mod support_mirror;
mod support_read;

use ai_loop_support::fixture::LoopFixture;
use ai_loop_support::turns::{FORMATS, collect, text_turn, tool_turn};
use mira_vocabulary::{ChatDelta, ContentKind};

/// Des arguments reçus en une dizaine de morceaux sont assemblés : l'extrait montré est la requête entière
#[tokio::test]
async fn arguments_split_in_many_fragments_are_assembled() {
    for format in FORMATS {
        let fixture = LoopFixture::start(format).await;
        let call = [("c1", "search_messages", r#"{"query":"le devis de mars"}"#)];
        fixture.queue(tool_turn(format, "", &call, None));
        fixture.queue(text_turn(format, "Fini.", None));
        let deltas = collect(&mut fixture.ask("Cherche").await).await;
        let summary = deltas.iter().find_map(|delta| match delta {
            ChatDelta::ToolCalled { summary, .. } => Some(summary.clone()),
            _ => None,
        });
        assert_eq!(summary.as_deref(), Some("le devis de mars"), "{format:?}");
    }
}

/// Les cinq outils du catalogue sont annoncés, et seulement eux, même si le client en propose d'autres
#[tokio::test]
async fn announced_tools_are_ours_only() {
    for format in FORMATS {
        let fixture = LoopFixture::start(format).await;
        fixture.queue(text_turn(format, "Salut.", None));
        let mut start = fixture.start_of("Salut", true);
        start.request.tools = vec![mira_vocabulary::ToolSpec {
            name: "rm_rf".to_string(),
            description: "x".to_string(),
            parameters_schema: "{}".to_string(),
        }];
        collect(&mut fixture.engine.chat_stream_with_tools(start).await.unwrap()).await;
        let body = fixture.bodies()[0].to_string();
        assert!(!body.contains("rm_rf"), "{format:?}");
        for name in [
            "search_messages",
            "list_messages",
            "get_message",
            "get_thread",
            "list_mailboxes",
        ] {
            assert!(body.contains(name), "{name}");
        }
    }
}

/// Un fournisseur qui ne supporte pas les outils n'en reçoit aucun
#[tokio::test]
async fn provider_without_support_announces_no_tools() {
    for format in FORMATS {
        let fixture = LoopFixture::start_with(format, false).await;
        fixture.queue(text_turn(format, "Salut.", None));
        collect(&mut fixture.ask("Salut").await).await;
        assert!(fixture.bodies()[0].get("tools").is_none(), "{format:?}");
    }
}

/// Le journal garde les noms d'outils, les types envoyés et des nombres, jamais d'argument ni de contenu
#[tokio::test]
async fn journal_row_lists_tools_and_no_content() {
    for format in FORMATS {
        let fixture = LoopFixture::start(format).await;
        let call = [("c1", "search_messages", r#"{"query":"devis"}"#)];
        fixture.queue(tool_turn(format, "", &call, None));
        fixture.queue(text_turn(format, "Fini.", None));
        collect(&mut fixture.ask("Cherche").await).await;
        let row = fixture
            .db
            .read(|tx| tx.list_ai_journal(1, None))
            .unwrap()
            .remove(0);
        assert_eq!(row.tools.len(), 1);
        assert_eq!(
            (row.tools[0].name.as_str(), row.tools[0].count),
            ("search_messages", 1)
        );
        assert!(row.kinds.contains(&ContentKind::MailList));
        let dump = serde_json::to_string(&row).unwrap();
        assert!(
            !dump.contains("devis") && !dump.contains("mars"),
            "{format:?}"
        );
    }
}

/// Un outil sans paramètre dont le modèle n'envoie aucun argument s'exécute comme avec `{}`
#[tokio::test]
async fn empty_arguments_mean_no_arguments() {
    for format in FORMATS {
        let fixture = LoopFixture::start(format).await;
        fixture.queue(tool_turn(format, "", &[("c1", "list_mailboxes", "")], None));
        fixture.queue(text_turn(format, "Voilà.", None));
        let deltas = collect(&mut fixture.ask("Liste").await).await;
        let state = deltas.iter().find_map(|delta| match delta {
            ChatDelta::ToolFinished { state, .. } => Some(*state),
            _ => None,
        });
        assert_eq!(state, Some(mira_vocabulary::ToolState::Ok), "{format:?}");
    }
}
