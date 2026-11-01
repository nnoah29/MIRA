#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Pont `chat_stream` et brouillons : la ligne `draft_proposed` suit le contrat, le mode composeur est lu depuis le JSON.

mod support;

use mira_assembly::mira_vocabulary::ChatMode;
use mira_bridge::runtime;
use mira_testkit::ai::{openai_stream, openai_tool_turn};
use support::chat_fixture::{chat_json_mode, collect_lines, start};

const COMPOSE: &str = r#"{"to":["Boss <boss@example.org>"],"subject":"Fin de stage","body":"Bonjour, mon stage finit mercredi."}"#;

/// Types des lignes reçues, dans l'ordre.
fn kinds(lines: &[serde_json::Value]) -> Vec<&str> {
    lines.iter().map(|l| l["type"].as_str().unwrap()).collect()
}

/// En mode assistant, la proposition arrive entre l'annonce et la fin de l'appel
#[tokio::test(flavor = "multi_thread")]
async fn draft_proposed_line_reaches_the_sink() {
    let f = start().await;
    f.server.queue(openai_tool_turn(
        "",
        &[("c1", "compose_draft", COMPOSE)],
        None,
    ));
    f.server.queue(openai_stream(&["Voilà."], None));
    let json = chat_json_mode(Some(&f.connection), "écris à mon boss", ChatMode::Assistant);
    let lines = collect_lines(&json).await;
    let expected = [
        "started",
        "tool_called",
        "draft_proposed",
        "tool_finished",
        "text",
        "done",
    ];
    assert_eq!(kinds(&lines), expected, "{lines:?}");
    assert_eq!(lines[2]["id"], "c1");
    assert_eq!(lines[2]["proposal"]["kind"], "new");
    assert_eq!(lines[2]["proposal"]["subject"], "Fin de stage");
    assert_eq!(lines[2]["proposal"]["to"][0]["address"], "boss@example.org");
    assert!(runtime::close().await.contains("\"ok\":true"));
}

/// Le mode composeur arrive du JSON de l'app : `edit_draft` est annoncé, `compose_draft` non
#[tokio::test(flavor = "multi_thread")]
async fn composer_mode_comes_from_the_json() {
    let f = start().await;
    f.server.queue(openai_tool_turn(
        "",
        &[("c1", "edit_draft", r#"{"body":"Nouveau"}"#)],
        None,
    ));
    f.server.queue(openai_stream(&["Fait."], None));
    let mode = ChatMode::Composer { selection: false };
    let json = chat_json_mode(Some(&f.connection), "raccourcis", mode);
    assert!(
        json.contains(r#""mode":{"kind":"composer","selection":false}"#),
        "{json}"
    );
    let lines = collect_lines(&json).await;
    assert_eq!(lines[2]["type"], "draft_proposed", "{lines:?}");
    assert_eq!(lines[2]["proposal"]["kind"], "edit");
    let body = &f.server.requests()[0].body;
    assert!(
        body.contains("edit_draft") && !body.contains("compose_draft"),
        "{body}"
    );
    assert!(runtime::close().await.contains("\"ok\":true"));
}

/// Une action sur la boîte proposée arrive comme ligne `action_proposed`, sans rien modifier
#[tokio::test(flavor = "multi_thread")]
async fn action_proposed_line_reaches_the_sink() {
    let f = start().await;
    let args = r#"{"action":"archive","messages":["m1","m2"],"reason":"Newsletters lues"}"#;
    f.server.queue(openai_tool_turn(
        "",
        &[("c1", "propose_mailbox_action", args)],
        None,
    ));
    f.server
        .queue(openai_stream(&["À toi de confirmer."], None));
    let json = chat_json_mode(
        Some(&f.connection),
        "archive mes newsletters",
        ChatMode::Assistant,
    );
    let lines = collect_lines(&json).await;
    let expected = [
        "started",
        "tool_called",
        "action_proposed",
        "tool_finished",
        "text",
        "done",
    ];
    assert_eq!(kinds(&lines), expected, "{lines:?}");
    assert_eq!(lines[2]["id"], "c1");
    assert_eq!(lines[2]["proposal"]["action"], "archive");
    assert_eq!(
        lines[2]["proposal"]["messages"],
        serde_json::json!(["m1", "m2"])
    );
    assert!(runtime::close().await.contains("\"ok\":true"));
}
