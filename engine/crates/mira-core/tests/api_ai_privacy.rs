#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Contrat 1.2 : consentement par type de contenu, liste des types autorisés, connexion inconnue.

mod ai_models_support;
mod support_ai_api;
mod support_api;

use mira_vocabulary::{AiConnectionId, ContentKind};
use support_ai_api::calls::add;
use support_ai_api::privacy::{acknowledge, acknowledge_without_kinds, set_allowed};
use support_ai_api::providers::{EDITABLE, started};

const URL: &str = "http://127.0.0.1:9/v1";

/// Un client du contrat 1.1, qui n'envoie pas `kinds`, autorise le mail ouvert et rien d'autre
#[tokio::test]
async fn acknowledge_without_kinds_allows_mail_message() {
    let f = started().await;
    let created = add(&f.engine, EDITABLE, "A", Some(URL)).await.unwrap();
    assert!(created.allowed_kinds.is_empty());
    let after = acknowledge_without_kinds(&f.engine, &created.id).await;
    let after = after.unwrap();
    assert!(after.consent_at.is_some());
    assert_eq!(after.allowed_kinds, [ContentKind::MailMessage]);
}

/// Les types donnés sont exactement ceux qui deviennent autorisés, sans doublon
#[tokio::test]
async fn acknowledge_with_kinds_allows_exactly_those() {
    let f = started().await;
    let created = add(&f.engine, EDITABLE, "A", Some(URL)).await.unwrap();
    let kinds = vec![
        ContentKind::MailList,
        ContentKind::MailList,
        ContentKind::Attachment,
    ];
    let after = acknowledge(&f.engine, &created.id, kinds).await.unwrap();
    assert_eq!(
        after.allowed_kinds,
        [ContentKind::MailList, ContentKind::Attachment]
    );
    assert_eq!(f.stored(&created).allowed_kinds, after.allowed_kinds);
}

/// Un second consentement garde la première date et ajoute les types sans retirer les anciens
#[tokio::test]
async fn second_acknowledge_keeps_the_first_consent_date() {
    let f = started().await;
    let created = add(&f.engine, EDITABLE, "A", Some(URL)).await.unwrap();
    let first = acknowledge(&f.engine, &created.id, vec![ContentKind::MailMessage]).await;
    let first = first.unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    let second = acknowledge(&f.engine, &created.id, vec![ContentKind::ToolResult]).await;
    let second = second.unwrap();
    assert_eq!(second.consent_at, first.consent_at);
    assert_eq!(
        second.allowed_kinds,
        [ContentKind::MailMessage, ContentKind::ToolResult]
    );
}

/// Définir la liste remplace l'ancienne, et la vide si on le demande
#[tokio::test]
async fn set_allowed_content_replaces_the_list() {
    let f = started().await;
    let created = add(&f.engine, EDITABLE, "A", Some(URL)).await.unwrap();
    acknowledge(
        &f.engine,
        &created.id,
        vec![ContentKind::MailMessage, ContentKind::MailList],
    )
    .await
    .unwrap();
    let kinds = vec![ContentKind::Attachment];
    let replaced = set_allowed(&f.engine, &created.id, kinds).await.unwrap();
    assert_eq!(replaced.allowed_kinds, [ContentKind::Attachment]);
    assert!(replaced.consent_at.is_some());
    let emptied = set_allowed(&f.engine, &created.id, Vec::new()).await;
    assert!(emptied.unwrap().allowed_kinds.is_empty());
}

/// Une connexion inconnue donne `not_found.ai_connection` pour chacune des trois opérations
#[tokio::test]
async fn unknown_connection_is_not_found() {
    let f = started().await;
    let unknown = AiConnectionId::from_raw("nope");
    let code = "not_found.ai_connection";
    let kinds = vec![ContentKind::MailMessage];
    let consent = acknowledge(&f.engine, &unknown, kinds.clone()).await;
    assert_eq!(consent.unwrap_err(), code);
    let allowed = set_allowed(&f.engine, &unknown, kinds).await;
    assert_eq!(allowed.unwrap_err(), code);
    let masking = support_ai_api::privacy::set_masking(&f.engine, &unknown, Default::default());
    assert_eq!(masking.await.unwrap_err(), code);
}
