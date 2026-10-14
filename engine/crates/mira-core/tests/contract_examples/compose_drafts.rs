//! Exemples de rédaction : création dans chaque mode, modification, pièces jointes, lecture.

use mira_core::api::{ComposeRequest, DraftChanges, DraftMode};
use mira_vocabulary::{AccountId, DraftId, IdentityId, MessageId};

/// Création d'un brouillon dans chacun des quatre modes
pub fn creation_requests() -> Vec<ComposeRequest> {
    let create = |mode| ComposeRequest::CreateDraft {
        account: AccountId::from_raw("account-1"),
        identity: Some(IdentityId::from_raw("identity-1")),
        mode,
    };
    let message = || MessageId::from_raw("message-1");
    vec![
        create(DraftMode::New),
        create(DraftMode::Reply { message: message() }),
        create(DraftMode::ReplyAll { message: message() }),
        create(DraftMode::Forward { message: message() }),
    ]
}

/// Modification, pièces jointes, suppression, lecture et liste
pub fn editing_requests() -> Vec<ComposeRequest> {
    let draft = DraftId::from_raw("draft-1");
    let changes = DraftChanges {
        subject: Some("Nouveau sujet".to_string()),
        ..DraftChanges::default()
    };
    vec![
        ComposeRequest::UpdateDraft {
            draft: draft.clone(),
            changes,
        },
        ComposeRequest::AttachFile {
            draft: draft.clone(),
            path: "/tmp/note.txt".to_string(),
        },
        ComposeRequest::RemoveAttachment {
            draft: draft.clone(),
            position: 0,
        },
        ComposeRequest::DeleteDraft {
            draft: draft.clone(),
        },
        ComposeRequest::GetDraft { draft },
        ComposeRequest::ListDrafts {
            account: AccountId::from_raw("account-1"),
        },
    ]
}
