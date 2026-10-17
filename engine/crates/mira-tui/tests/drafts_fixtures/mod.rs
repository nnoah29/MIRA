//! Commandes, réponses et états de la vue « Brouillons ».
#![allow(dead_code, unused_imports)]

mod replies;

use mira_assembly::api::{ComposeRequest, Request};
use mira_assembly::mira_vocabulary::{AccountId, Draft, DraftId, DraftState, Timestamp};
use mira_tui::commands::{Command, Tag};

pub use replies::{
    drafts_reply, loaded_reply, not_editable_error, open_drafts_with, reopen, saved_draft,
};

/// Lecture attendue des brouillons de ce compte, suivie par ce jeton.
pub fn list_command(account: &str, token: u64) -> Command {
    Command::Execute {
        tag: Tag::Drafts(token),
        request: Request::Compose(ComposeRequest::ListDrafts {
            account: AccountId::from_raw(account),
        }),
    }
}

/// Relecture attendue de ce brouillon avant sa réouverture, suivie par ce jeton.
pub fn get_command(token: u64, draft: &str) -> Command {
    Command::Execute {
        tag: Tag::DraftLoaded(token),
        request: Request::Compose(ComposeRequest::GetDraft {
            draft: DraftId::from_raw(draft),
        }),
    }
}

/// Le même brouillon, planifié pour l'envoi.
pub fn scheduled(mut draft: Draft) -> Draft {
    draft.state = DraftState::Scheduled {
        at: Timestamp::from_millis(60_000),
    };
    draft
}

/// Le même brouillon, en cours d'envoi.
pub fn sending(mut draft: Draft) -> Draft {
    draft.state = DraftState::Sending;
    draft
}

/// Brouillons d'objets « Objet <id> », dans cet ordre.
pub fn named(ids: &[&str]) -> Vec<Draft> {
    ids.iter()
        .map(|id| {
            crate::compose_fixtures::draft(id, &["bob@example.org"], &format!("Objet {id}"), "")
        })
        .collect()
}

/// Seules les lectures de la vue « Brouillons » parmi ces commandes.
pub fn only_drafts(commands: Vec<Command>) -> Vec<Command> {
    commands
        .into_iter()
        .filter(|command| {
            matches!(
                command,
                Command::Execute {
                    tag: Tag::Drafts(_),
                    ..
                }
            )
        })
        .collect()
}
