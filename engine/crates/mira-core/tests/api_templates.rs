#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Domaine « template » par la façade : création, idempotence, mise à jour, événements.

mod support;
mod support_api;
mod support_imap;
mod support_mirror;
mod support_template_api;
mod support_write;

use mira_core::api::TemplateRequest;
use support_imap::within;
use support_template_api::calls::{call, changed_count, create, templates};
use support_write::fixture::Fixture;

/// Créer puis lister : identifiants du moteur, tri par nom sans casse, un événement par création
#[tokio::test]
async fn create_then_list_returns_templates_sorted() {
    within(async {
        let f = Fixture::start().await;
        let first = create(f.engine(), "  zèbre ", "Corps", "k-1")
            .await
            .unwrap();
        let second = create(f.engine(), "Alpha", "Corps", "k-2").await.unwrap();
        assert_eq!(first.name, "zèbre");
        assert_ne!(first.id, second.id);
        assert_eq!(templates(f.engine()).await, vec![second, first]);
        assert_eq!(changed_count(f.engine()).await, 2);
    })
    .await;
}

/// La même clé rejouée rend le même modèle sans doublon ni nouvel événement (invariant 18)
#[tokio::test]
async fn create_with_same_key_replays_without_duplicate_or_event() {
    within(async {
        let f = Fixture::start().await;
        let first = create(f.engine(), "Merci", "Corps", "k-1").await.unwrap();
        let again = create(f.engine(), "Merci", "Corps", "k-1").await.unwrap();
        assert_eq!(first, again);
        assert_eq!(templates(f.engine()).await.len(), 1);
        assert_eq!(changed_count(f.engine()).await, 1);
    })
    .await;
}

/// La même clé avec une autre commande est un conflit d'idempotence
#[tokio::test]
async fn same_key_other_payload_is_idempotency_conflict() {
    within(async {
        let f = Fixture::start().await;
        create(f.engine(), "Merci", "Corps", "k-1").await.unwrap();
        let conflict = create(f.engine(), "Merci", "Autre corps", "k-1").await;
        assert_eq!(conflict.unwrap_err(), "invalid_input.idempotency_conflict");
        assert_eq!(changed_count(f.engine()).await, 1);
    })
    .await;
}

/// Mettre à jour change les champs et émet un seul événement de plus
#[tokio::test]
async fn update_changes_fields_and_emits_one_event() {
    within(async {
        let f = Fixture::start().await;
        let made = create(f.engine(), "Merci", "Corps", "k-1").await.unwrap();
        let update = TemplateRequest::UpdateTemplate {
            template: made.id.clone(),
            name: "Remerciement".to_string(),
            subject: Some("Objet".to_string()),
            body: "Nouveau corps".to_string(),
        };
        call(f.engine(), update).await.unwrap();
        let shown = templates(f.engine()).await;
        assert_eq!(shown[0].name, "Remerciement");
        assert_eq!(shown[0].subject.as_deref(), Some("Objet"));
        assert_eq!(shown[0].body, "Nouveau corps");
        assert_eq!(changed_count(f.engine()).await, 2);
    })
    .await;
}

/// Renommer vers le nom d'un autre modèle est refusé et n'émet rien
#[tokio::test]
async fn update_to_taken_name_is_refused() {
    within(async {
        let f = Fixture::start().await;
        create(f.engine(), "Merci", "Corps", "k-1").await.unwrap();
        let other = create(f.engine(), "Relance", "Corps", "k-2").await.unwrap();
        let update = TemplateRequest::UpdateTemplate {
            template: other.id,
            name: "MERCI".to_string(),
            subject: None,
            body: "Corps".to_string(),
        };
        let refused = call(f.engine(), update).await.unwrap_err();
        assert_eq!(refused, "invalid_input.template_name_taken");
        assert_eq!(changed_count(f.engine()).await, 2);
    })
    .await;
}
