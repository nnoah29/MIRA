#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Contrat 1.2 : réglage du masquage, valeur par défaut selon la destination, oubli des types à un changement d'adresse.

mod ai_models_support;
mod support_ai_api;
mod support_api;

use mira_vocabulary::{ContentKind, MaskingMode, MaskingSettings, SensitiveCategory};
use support_ai_api::calls::{add, list};
use support_ai_api::operations::update;
use support_ai_api::privacy::{acknowledge, set_masking};
use support_ai_api::providers::{EDITABLE, started};

const LOCAL_URL: &str = "http://127.0.0.1:9/v1";
const REMOTE_URL: &str = "https://ia.example.org/v1";

/// Le réglage posé est rendu, relu en base et listé
#[tokio::test]
async fn set_masking_roundtrip_and_listed() {
    let f = started().await;
    let created = add(&f.engine, EDITABLE, "A", Some(REMOTE_URL))
        .await
        .unwrap();
    let wanted = MaskingSettings {
        mode: MaskingMode::Mask,
        categories: vec![SensitiveCategory::Iban, SensitiveCategory::Email],
    };
    let after = set_masking(&f.engine, &created.id, wanted.clone()).await;
    assert_eq!(after.unwrap().masking, wanted);
    assert_eq!(f.stored(&created).masking, wanted);
    assert_eq!(list(&f.engine).await[0].masking, wanted);
}

/// Une connexion distante est créée avec le masquage actif sur les catégories par défaut
#[tokio::test]
async fn add_remote_connection_defaults_masking_to_mask() {
    let f = started().await;
    let created = add(&f.engine, EDITABLE, "A", Some(REMOTE_URL))
        .await
        .unwrap();
    assert_eq!(created.masking, MaskingSettings::remote_default());
    assert_eq!(created.masking.mode, MaskingMode::Mask);
    assert_eq!(
        f.stored(&created).masking,
        MaskingSettings::remote_default()
    );
}

/// Une connexion vers la boucle locale est créée avec le masquage coupé
#[tokio::test]
async fn add_loopback_connection_defaults_masking_to_off() {
    let f = started().await;
    let created = add(&f.engine, EDITABLE, "A", Some(LOCAL_URL))
        .await
        .unwrap();
    assert_eq!(created.masking, MaskingSettings::local_default());
    assert_eq!(created.masking.mode, MaskingMode::Off);
    let fixed = add(&f.engine, support_ai_api::providers::FIXED, "B", None).await;
    assert_eq!(fixed.unwrap().masking, MaskingSettings::local_default());
}

/// Changer l'adresse oublie les types autorisés avec le consentement ; la même adresse les garde
#[tokio::test]
async fn changing_the_url_clears_allowed_kinds() {
    let f = started().await;
    let created = add(&f.engine, EDITABLE, "A", Some(LOCAL_URL))
        .await
        .unwrap();
    let kinds = vec![ContentKind::MailMessage, ContentKind::MailList];
    acknowledge(&f.engine, &created.id, kinds.clone())
        .await
        .unwrap();
    let same = update(&f.engine, &created.id, "B", Some(LOCAL_URL), None).await;
    assert_eq!(same.unwrap().allowed_kinds, kinds);
    let moved = update(&f.engine, &created.id, "B", Some(REMOTE_URL), None).await;
    let moved = moved.unwrap();
    assert!(moved.allowed_kinds.is_empty() && moved.consent_at.is_none());
    assert!(f.stored(&created).allowed_kinds.is_empty());
}
