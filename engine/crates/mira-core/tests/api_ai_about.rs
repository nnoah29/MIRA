#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Champ « ce que tu dois savoir de moi » par la façade : vide au départ, enregistré rogné, borné à 1 000 caractères.

mod local_chat_support;
mod local_support;
mod support_ai_about;
mod support_api;

use local_chat_support::fixture::LocalFixture;
use support_ai_about::{read, save};

/// Un profil neuf n'a rien à dire de l'utilisateur
#[tokio::test]
async fn a_new_profile_has_an_empty_text() {
    let f = LocalFixture::start(300);
    assert_eq!(read(&f).await, "");
}

/// Le texte est rogné, les fins de ligne Windows normalisées, et il se relit à l'identique
#[tokio::test]
async fn the_text_is_trimmed_normalized_and_kept() {
    let f = LocalFixture::start(300);
    let kept = save(&f, "  Je suis médecin.\r\nJe signe « Noah ».  ")
        .await
        .unwrap();
    assert_eq!(kept, "Je suis médecin.\nJe signe « Noah ».");
    assert_eq!(read(&f).await, kept);
}

/// Un texte vide efface le champ
#[tokio::test]
async fn an_empty_text_clears_the_field() {
    let f = LocalFixture::start(300);
    save(&f, "quelque chose").await.unwrap();
    assert_eq!(save(&f, "   ").await.unwrap(), "");
    assert_eq!(read(&f).await, "");
}

/// Mille caractères passent, mille un sont refusés et l'ancien texte reste
#[tokio::test]
async fn the_limit_is_one_thousand_characters() {
    let f = LocalFixture::start(300);
    assert_eq!(
        save(&f, &"é".repeat(1_000)).await.unwrap().chars().count(),
        1_000
    );
    let error = save(&f, &"é".repeat(1_001)).await.unwrap_err();
    assert_eq!(error.code, "invalid_input.ai_about_text");
    assert_eq!(read(&f).await.chars().count(), 1_000);
}
