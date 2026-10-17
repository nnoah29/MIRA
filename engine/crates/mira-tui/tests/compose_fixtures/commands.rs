//! Commandes attendues et réponses du moteur pour la rédaction.
#![allow(dead_code)]

use mira_assembly::api::{
    ComposeRequest, DraftChanges, DraftMode, Request, Response, WriteResponse,
};
use mira_assembly::mira_vocabulary::{AccountId, Draft, DraftId, IdempotencyKey, MiraError};
use mira_tui::commands::{Command, Tag};
use mira_tui::update::{Msg, Reply};

/// Création de brouillon attendue pour ce compte, ce mode et cette clé de suivi.
pub fn create_command(account: &str, mode: DraftMode, key: &str) -> Command {
    Command::Execute {
        tag: Tag::DraftCreated(IdempotencyKey(key.to_string())),
        request: Request::Compose(ComposeRequest::CreateDraft {
            account: AccountId::from_raw(account),
            identity: None,
            mode,
        }),
    }
}

/// Enregistrement attendu de ce brouillon avec ces changements.
pub fn update_command(draft: &str, changes: DraftChanges) -> Command {
    Command::Execute {
        tag: Tag::DraftSaved(DraftId::from_raw(draft)),
        request: Request::Compose(ComposeRequest::UpdateDraft {
            draft: DraftId::from_raw(draft),
            changes,
        }),
    }
}

/// Suppression attendue de ce brouillon.
pub fn delete_command(draft: &str) -> Command {
    Command::Execute {
        tag: Tag::DraftDeleted(DraftId::from_raw(draft)),
        request: Request::Compose(ComposeRequest::DeleteDraft {
            draft: DraftId::from_raw(draft),
        }),
    }
}

/// Réponse du moteur à la création suivie par cette clé.
pub fn created_reply(key: &str, draft: Draft) -> Msg {
    Msg::Reply(Reply {
        tag: Tag::DraftCreated(IdempotencyKey(key.to_string())),
        result: Ok(Response::Write(WriteResponse::Draft(Box::new(draft)))),
    })
}

/// Réponse du moteur à l'enregistrement de ce brouillon : son état à jour, ou une erreur.
pub fn saved_reply(id: &str, result: Result<Draft, MiraError>) -> Msg {
    Msg::Reply(Reply {
        tag: Tag::DraftSaved(DraftId::from_raw(id)),
        result: result.map(|draft| Response::Write(WriteResponse::Draft(Box::new(draft)))),
    })
}

/// Réponse du moteur à la suppression de ce brouillon.
pub fn deleted_reply(id: &str, result: Result<(), MiraError>) -> Msg {
    Msg::Reply(Reply {
        tag: Tag::DraftDeleted(DraftId::from_raw(id)),
        result: result.map(|()| Response::Done),
    })
}
