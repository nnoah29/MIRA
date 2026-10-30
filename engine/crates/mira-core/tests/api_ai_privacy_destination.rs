#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Changement d'adresse d'une connexion : le masquage reprend le défaut de la nouvelle destination.

mod ai_models_support;
mod support_ai_api;
mod support_api;

use mira_vocabulary::{MaskingMode, MaskingSettings, SensitiveCategory};
use support_ai_api::calls::add;
use support_ai_api::operations::update;
use support_ai_api::privacy::set_masking;
use support_ai_api::providers::{EDITABLE, started};

const LOCAL_URL: &str = "http://127.0.0.1:9/v1";
const REMOTE_URL: &str = "https://ia.example.org/v1";

/// Réglage choisi par l'utilisateur, différent des deux défauts
fn custom() -> MaskingSettings {
    MaskingSettings {
        mode: MaskingMode::Off,
        categories: vec![SensitiveCategory::Email],
    }
}

/// De la boucle locale vers un service distant, le masquage redevient actif par défaut
#[tokio::test]
async fn changing_the_url_from_loopback_to_remote_resets_masking_to_mask() {
    let f = started().await;
    let created = add(&f.engine, EDITABLE, "A", Some(LOCAL_URL))
        .await
        .unwrap();
    set_masking(&f.engine, &created.id, custom()).await.unwrap();
    let moved = update(&f.engine, &created.id, "A", Some(REMOTE_URL), None).await;
    assert_eq!(moved.unwrap().masking, MaskingSettings::remote_default());
    assert_eq!(
        f.stored(&created).masking,
        MaskingSettings::remote_default()
    );
}

/// D'un service distant vers la boucle locale, le masquage est coupé par défaut
#[tokio::test]
async fn changing_the_url_from_remote_to_loopback_sets_masking_off() {
    let f = started().await;
    let created = add(&f.engine, EDITABLE, "A", Some(REMOTE_URL))
        .await
        .unwrap();
    let moved = update(&f.engine, &created.id, "A", Some(LOCAL_URL), None).await;
    assert_eq!(moved.unwrap().masking, MaskingSettings::local_default());
    assert_eq!(f.stored(&created).masking, MaskingSettings::local_default());
}

/// La même adresse, même écrite avec une barre de fin, garde le réglage de l'utilisateur
#[tokio::test]
async fn unchanged_url_keeps_the_user_masking() {
    let f = started().await;
    let created = add(&f.engine, EDITABLE, "A", Some(REMOTE_URL))
        .await
        .unwrap();
    set_masking(&f.engine, &created.id, custom()).await.unwrap();
    let same = format!("{REMOTE_URL}/");
    let kept = update(&f.engine, &created.id, "B", Some(&same), None).await;
    assert_eq!(kept.unwrap().masking, custom());
    assert_eq!(f.stored(&created).masking, custom());
}
