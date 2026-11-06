//! Lignes du flux de chat : `started`, texte, usage, `done`, et connexion par défaut.

use mira_assembly::api::AiRequest;
use mira_assembly::mira_vocabulary::AiConnectionId;
use mira_testkit::ai::openai_stream;

use super::chat_fixture::chat_json;
use super::chat_start::{ask_ai, collect_lines, start};
use super::driver::Driver;

/// La première ligne annonce l'opération, sous la forme d'un identifiant non vide.
pub async fn first_line_is_started_with_operation_id<D: Driver>() {
    let f = start::<D>().await;
    f.server.queue(openai_stream(&["Bon"], None));
    let lines = collect_lines(&f.driver, &chat_json(Some(&f.connection), "Salut")).await;
    assert_eq!(lines[0]["type"], "started", "{lines:?}");
    assert!(!lines[0]["operation"].as_str().unwrap().is_empty());
    assert!(f.driver.close().await.contains("\"ok\":true"));
}

/// Les lignes suivent l'ordre `started`, `text`, `usage`, `done`, et le texte se recompose.
pub async fn lines_follow_started_text_usage_done<D: Driver>() {
    let f = start::<D>().await;
    f.server
        .queue(openai_stream(&["Bon", "jour"], Some((12, 5))));
    let lines = collect_lines(&f.driver, &chat_json(Some(&f.connection), "Salut")).await;
    let kinds: Vec<&str> = lines.iter().map(|l| l["type"].as_str().unwrap()).collect();
    assert_eq!(
        kinds,
        ["started", "text", "text", "usage", "done"],
        "{lines:?}"
    );
    assert_eq!(lines[1]["text"], "Bon");
    assert_eq!(lines[3]["input_tokens"], 12);
    assert_eq!(lines[3]["output_tokens"], 5);
    assert_eq!(lines[4]["reason"], "end_turn");
    assert!(f.driver.close().await.contains("\"ok\":true"));
}

/// Sans connexion désignée, la connexion par défaut sert.
pub async fn default_connection_serves_when_none_is_named<D: Driver>() {
    let f = start::<D>().await;
    let connection = AiConnectionId::from_raw(f.connection.clone());
    ask_ai(&f.driver, AiRequest::SetDefaultAiConnection { connection }).await;
    f.server.queue(openai_stream(&["ok"], None));
    let lines = collect_lines(&f.driver, &chat_json(None, "Salut")).await;
    assert_eq!(lines.last().unwrap()["type"], "done", "{lines:?}");
    assert!(f.driver.close().await.contains("\"ok\":true"));
}
