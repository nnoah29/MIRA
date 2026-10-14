#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Brouillons par la façade : création, réponse, transfert, modification, pièces jointes, suppression (12.4, 9.10).

mod support;
mod support_api;
mod support_imap;
mod support_mirror;
mod support_write;

use mira_core::api::{ComposeRequest, DraftChanges, DraftMode, Request, Response, WriteResponse};
use mira_vocabulary::{AccountId, DraftId, MessageId};
use support_imap::within;
use support_write::compose::recipient;
use support_write::fixture::Fixture;

/// Créer, relire, lister, modifier puis supprimer un brouillon
#[tokio::test]
async fn draft_lifecycle_create_update_list_delete() {
    within(async {
        let f = Fixture::start().await;
        let draft = f.create_draft(DraftMode::New).await;
        assert_eq!(f.get_draft(&draft.id).await, draft);
        assert_eq!(f.list_drafts().await.len(), 1);
        let html = |text: &str| DraftChanges {
            subject: Some("Nouveau sujet".into()),
            body_html: Some(text.into()),
            ..DraftChanges::default()
        };
        let updated = f.update_draft(&draft.id, html("<p>x</p>")).await;
        assert_eq!(updated.subject, "Nouveau sujet");
        assert_eq!(updated.body_html.as_deref(), Some("<p>x</p>"));
        assert!(
            f.update_draft(&draft.id, html(""))
                .await
                .body_html
                .is_none()
        );
        let delete = ComposeRequest::DeleteDraft { draft: draft.id };
        f.run(Request::Compose(delete)).await;
        assert!(f.list_drafts().await.is_empty());
    })
    .await;
}

/// Réponse, réponse à tous et transfert partent du message d'origine
#[tokio::test]
async fn reply_and_forward_drafts_start_from_the_original_message() {
    within(async {
        let f = Fixture::start().await;
        let message = MessageId::from_raw("msg-1");
        let reply = f
            .create_draft(DraftMode::Reply {
                message: message.clone(),
            })
            .await;
        assert_eq!(reply.subject, "Re: Sujet msg-1");
        assert_eq!(reply.to[0].address, "bob@example.org");
        assert_eq!(reply.in_reply_to, Some(message.clone()));
        let all = f
            .create_draft(DraftMode::ReplyAll {
                message: message.clone(),
            })
            .await;
        assert!(all.subject.starts_with("Re:"));
        let forward = f.create_draft(DraftMode::Forward { message }).await;
        assert_eq!(forward.subject, "Fwd: Sujet msg-1");
        assert!(forward.to.is_empty());
    })
    .await;
}

/// Un message ou un compte inconnu, une adresse mal formée : erreurs du contrat, aucun brouillon créé
#[tokio::test]
async fn draft_creation_and_update_validate_their_inputs() {
    within(async {
        let f = Fixture::start().await;
        let create = |account, mode| ComposeRequest::CreateDraft {
            account,
            identity: None,
            mode,
        };
        let missing = MessageId::from_raw("msg-inconnu");
        let request = create(f.account.clone(), DraftMode::Reply { message: missing });
        assert_eq!(f.refused_compose(request).await, "not_found.message");
        let nobody = AccountId::from_raw("compte-inconnu");
        let request = create(nobody, DraftMode::New);
        assert_eq!(f.refused_compose(request).await, "not_found.account");
        let draft = f.create_draft(DraftMode::New).await;
        let to = Some(vec![recipient("pas-une-adresse")]);
        let changes = DraftChanges {
            to,
            ..DraftChanges::default()
        };
        let update = ComposeRequest::UpdateDraft {
            draft: draft.id,
            changes,
        };
        assert_eq!(f.refused_compose(update).await, "invalid_input.address");
        assert_eq!(f.list_drafts().await.len(), 1);
    })
    .await;
}

/// Joindre un fichier local puis le retirer par son rang ; un rang inconnu est refusé
#[tokio::test]
async fn attach_file_then_remove_attachment_by_position() {
    within(async {
        let f = Fixture::start().await;
        let draft = f.create_draft(DraftMode::New).await;
        let path = f.home.path().join("note.txt");
        std::fs::write(&path, b"contenu de la piece jointe").unwrap();
        let attach = ComposeRequest::AttachFile {
            draft: draft.id.clone(),
            path: path.display().to_string(),
        };
        let attached = f.draft_of(attach).await;
        assert_eq!(attached.attachments.len(), 1);
        assert_eq!(attached.attachments[0].filename, "note.txt");
        let remove = |position: u32| {
            Request::Compose(ComposeRequest::RemoveAttachment {
                draft: draft.id.clone(),
                position,
            })
        };
        assert_eq!(f.refused(remove(5)).await.code, "invalid_input.parameter");
        match f.run(remove(0)).await {
            Response::Write(WriteResponse::Draft(left)) => assert!(left.attachments.is_empty()),
            other => panic!("réponse inattendue : {other:?}"),
        }
    })
    .await;
}

/// Un brouillon planifié ne se modifie ni ne se supprime ; l'annulation de l'envoi le libère
#[tokio::test]
async fn scheduled_draft_is_locked_until_the_send_is_cancelled() {
    within(async {
        let f = Fixture::start().await;
        let draft = f.ready_draft().await;
        let action = f.send(&draft.id, "k1").await;
        let changes = DraftChanges {
            subject: Some("Trop tard".into()),
            ..DraftChanges::default()
        };
        let update = Request::Compose(ComposeRequest::UpdateDraft {
            draft: draft.id.clone(),
            changes,
        });
        assert_eq!(
            f.refused(update).await.code,
            "incompatible_state.draft_not_editable"
        );
        let delete =
            |id: &DraftId| Request::Compose(ComposeRequest::DeleteDraft { draft: id.clone() });
        assert_eq!(
            f.refused(delete(&draft.id)).await.code,
            "incompatible_state.draft_not_editable"
        );
        f.run(Request::Compose(ComposeRequest::CancelSend { action }))
            .await;
        f.run(delete(&draft.id)).await;
        assert!(f.list_drafts().await.is_empty());
    })
    .await;
}
