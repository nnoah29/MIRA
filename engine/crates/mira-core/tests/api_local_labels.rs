#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Domaine « local_label » par la façade : création, noms, plafond, renommage, couleur, idempotence.

mod support;
mod support_api;
mod support_imap;
mod support_label_api;
mod support_mirror;
mod support_write;

use mira_core::api::LocalLabelRequest;
use support_imap::within;
use support_label_api::calls::{call, changed_count, create, key, labels};
use support_write::fixture::Fixture;

/// Créer puis lister : identifiants du moteur, couleur de la palette par défaut, un événement par création
#[tokio::test]
async fn create_then_list_returns_labels_with_default_colors() {
    within(async {
        let f = Fixture::start().await;
        let first = create(f.engine(), "  Urgent ", "k-1").await.unwrap();
        let second = create(f.engine(), "Banque", "k-2").await.unwrap();
        assert_eq!(first.name, "Urgent");
        assert_ne!(first.id, second.id);
        assert_ne!(first.color, second.color);
        assert_eq!(labels(f.engine()).await, vec![second, first]);
        assert_eq!(changed_count(f.engine()).await, 2);
    })
    .await;
}

/// Renomme puis recolore un libellé par la façade
async fn rename_and_recolor(f: &Fixture, id: &mira_vocabulary::LocalLabelId) {
    let rename = LocalLabelRequest::RenameLocalLabel {
        label: id.clone(),
        name: "À payer".to_string(),
    };
    call(f.engine(), rename).await.unwrap();
    let color = LocalLabelRequest::RecolorLocalLabel {
        label: id.clone(),
        color: 0xFF00_FF00,
    };
    call(f.engine(), color).await.unwrap();
}

/// Renommer, recolorer, supprimer ; un libellé inconnu donne `not_found.local_label`
#[tokio::test]
async fn rename_recolor_delete_roundtrip() {
    within(async {
        let f = Fixture::start().await;
        let label = create(f.engine(), "Urgent", "k-1").await.unwrap();
        rename_and_recolor(&f, &label.id).await;
        let shown = labels(f.engine()).await;
        assert_eq!(
            (shown[0].name.as_str(), shown[0].color),
            ("À payer", 0xFF00_FF00)
        );
        let delete = LocalLabelRequest::DeleteLocalLabel {
            label: label.id.clone(),
        };
        call(f.engine(), delete.clone()).await.unwrap();
        assert_eq!(
            call(f.engine(), delete).await.unwrap_err(),
            "not_found.local_label"
        );
        assert!(labels(f.engine()).await.is_empty());
        assert_eq!(changed_count(f.engine()).await, 4);
    })
    .await;
}

/// Nom vide, trop long ou en doublon : refusé sans rien écrire, sans reprendre le texte saisi
#[tokio::test]
async fn invalid_names_are_refused_without_writing() {
    within(async {
        let f = Fixture::start().await;
        create(f.engine(), "Urgent", "k-1").await.unwrap();
        let too_long = "x".repeat(65);
        for (name, sent) in [
            ("   ", "k-2"),
            (too_long.as_str(), "k-3"),
            ("URGENT", "k-4"),
        ] {
            let refused = create(f.engine(), name, sent).await;
            assert_eq!(refused.unwrap_err(), "invalid_input.parameter");
        }
        let request = LocalLabelRequest::CreateLocalLabel {
            name: "Secret-à-ne-pas-citer ".repeat(10),
            color: None,
            idempotency_key: key("k-5"),
        };
        let error = f
            .engine()
            .execute(mira_core::api::Request::LocalLabel(request))
            .await;
        assert!(!error.unwrap_err().message.contains("Secret-à-ne-pas-citer"));
        assert_eq!(labels(f.engine()).await.len(), 1);
        assert_eq!(changed_count(f.engine()).await, 1);
    })
    .await;
}

/// Au-delà de 200 libellés, la création est refusée
#[tokio::test]
async fn label_limit_is_enforced() {
    within(async {
        let f = Fixture::start().await;
        for n in 0..200 {
            create(f.engine(), &format!("L{n}"), &format!("k-{n}"))
                .await
                .unwrap();
        }
        let over = create(f.engine(), "L200", "k-200").await;
        assert_eq!(over.unwrap_err(), "invalid_input.parameter");
        assert_eq!(labels(f.engine()).await.len(), 200);
    })
    .await;
}

/// Même clé et mêmes données : même libellé, sans doublon ni événement ; autres données sous la clé : conflit
#[tokio::test]
async fn same_key_returns_same_label_and_other_command_conflicts() {
    within(async {
        let f = Fixture::start().await;
        let first = create(f.engine(), "Urgent", "k-1").await.unwrap();
        let again = create(f.engine(), "Urgent", "k-1").await.unwrap();
        assert_eq!(first, again);
        assert_eq!(labels(f.engine()).await.len(), 1);
        assert_eq!(changed_count(f.engine()).await, 1);
        let other = create(f.engine(), "Autre", "k-1").await;
        assert_eq!(other.unwrap_err(), "invalid_input.idempotency_conflict");
    })
    .await;
}
