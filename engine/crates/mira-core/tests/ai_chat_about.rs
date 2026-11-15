#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Champ « ce que tu dois savoir de moi » dans la discussion : ajouté à la consigne, masqué, absent s'il est vide.

mod ai_loop_support;
mod ai_models_support;
mod support;
mod support_api;
mod support_mirror;
mod support_read;

use ai_loop_support::fixture::LoopFixture;
use ai_loop_support::turns::{FORMATS, collect, text_turn};
use mira_vocabulary::ChatMode;

/// Texte du champ, enregistré dans la base du profil
fn set_about(f: &LoopFixture, text: &str) {
    f.db.write(|tx| tx.set_ai_about(text)).unwrap();
}

/// Corps de la dernière requête envoyée au service, en texte
fn sent(f: &LoopFixture) -> String {
    f.bodies().last().unwrap().to_string()
}

/// Le texte arrive dans la consigne de l'assistant, avec son en-tête
#[tokio::test]
async fn the_text_is_added_to_the_assistant_prompt() {
    for format in FORMATS {
        let f = LoopFixture::start(format).await;
        set_about(&f, "Je suis médecin à Cotonou.");
        f.queue(text_turn(format, "Bonjour.", None));
        collect(&mut f.ask("bonjour").await).await;
        assert!(
            sent(&f).contains("Je suis médecin à Cotonou."),
            "{format:?}"
        );
        assert!(sent(&f).contains("veut que tu saches"), "{format:?}");
    }
}

/// Vide, le champ n'ajoute rien
#[tokio::test]
async fn an_empty_field_adds_nothing() {
    let format = FORMATS[1];
    let f = LoopFixture::start(format).await;
    f.queue(text_turn(format, "Bonjour.", None));
    collect(&mut f.ask("bonjour").await).await;
    assert!(!sent(&f).contains("veut que tu saches"));
}

/// Le masquage s'applique : un IBAN écrit dans le champ ne part pas en clair
#[tokio::test]
async fn sensitive_values_in_the_text_are_masked() {
    let format = FORMATS[1];
    let f = LoopFixture::start(format).await;
    set_about(&f, "Mon IBAN est FR7630006000011234567890189.");
    f.queue(text_turn(format, "Ok.", None));
    collect(&mut f.ask("bonjour").await).await;
    assert!(!sent(&f).contains("FR7630006000011234567890189"));
}

/// En mode composeur aussi, le texte accompagne la rédaction
#[tokio::test]
async fn the_composer_gets_the_text_too() {
    let format = FORMATS[0];
    let f = LoopFixture::start(format).await;
    set_about(&f, "Je signe toujours « Noah ».");
    f.queue(text_turn(format, "Voici.", None));
    let mut start = f.start_of("écris un mail", true);
    start.mode = ChatMode::Composer { selection: false };
    collect(&mut f.engine.chat_stream_with_tools(start).await.unwrap()).await;
    assert!(sent(&f).contains("Je signe toujours"));
}
