#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Consigne ajoutée par le moteur quand des outils sont annoncés : écrire par les outils, sans boucler sur des recherches.

mod ai_loop_support;
mod ai_models_support;
mod support;
mod support_api;
mod support_mirror;
mod support_read;

use ai_loop_support::fixture::LoopFixture;
use ai_loop_support::turns::{collect, text_turn};
use mira_vocabulary::{ChatMode, WireFormat};

/// Texte du message système du premier appel au modèle (format OpenAI)
async fn system_text(mode: ChatMode) -> String {
    let format = WireFormat::OpenAiChat;
    let fixture = LoopFixture::start(format).await;
    fixture.queue(text_turn(format, "Ok.", None));
    let mut start = fixture.start_of("écris", true);
    start.mode = mode;
    let mut stream = fixture.engine.chat_stream_with_tools(start).await.unwrap();
    collect(&mut stream).await;
    let body = &fixture.bodies()[0];
    body["messages"][0]["content"].as_str().unwrap().to_string()
}

/// Dans tous les modes à outils : au plus deux recherches, puis on écrit quand même en disant ce qui manque
#[tokio::test]
async fn the_notice_forbids_looping_on_searches() {
    for mode in [ChatMode::Assistant, ChatMode::Composer { selection: false }] {
        let system = system_text(mode).await;
        assert!(
            system.contains("au plus deux recherches"),
            "{mode:?}: {system}"
        );
        assert!(
            system.contains("sans tourner en boucle"),
            "{mode:?}: {system}"
        );
    }
}

/// Seul le mode assistant parle des actions sur la boîte
#[tokio::test]
async fn only_the_assistant_notice_mentions_mailbox_actions() {
    let assistant = system_text(ChatMode::Assistant).await;
    assert!(assistant.contains("propose_mailbox_action"));
    let composer = system_text(ChatMode::Composer { selection: false }).await;
    assert!(!composer.contains("propose_mailbox_action"));
}
