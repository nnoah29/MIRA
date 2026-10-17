//! Réponses du moteur à la vue « Brouillons » et états où elle est ouverte.
#![allow(dead_code)]

use mira_assembly::api::{Response, WriteResponse};
use mira_assembly::mira_vocabulary::error::codes::incompatible_state::DRAFT_NOT_EDITABLE;
use mira_assembly::mira_vocabulary::{Draft, ErrorCategory, MiraError};
use mira_tui::commands::Tag;
use mira_tui::state::AppState;
use mira_tui::update::{DraftsIntent, Intent, Msg, Reply, update};

use crate::action_fixtures::state_with_mailboxes;
use crate::navigation_fixtures::press;

/// Réponse à la lecture des brouillons suivie par ce jeton.
pub fn drafts_reply(token: u64, drafts: Vec<Draft>) -> Msg {
    Msg::Reply(Reply {
        tag: Tag::Drafts(token),
        result: Ok(Response::Write(WriteResponse::Drafts(drafts))),
    })
}

/// Réponse à la relecture d'un brouillon suivie par ce jeton.
pub fn loaded_reply(token: u64, result: Result<Draft, MiraError>) -> Msg {
    Msg::Reply(Reply {
        tag: Tag::DraftLoaded(token),
        result: result.map(|draft| Response::Write(WriteResponse::Draft(Box::new(draft)))),
    })
}

/// Refus du moteur : le brouillon est planifié ou en cours d'envoi.
pub fn not_editable_error() -> MiraError {
    MiraError::new(
        ErrorCategory::IncompatibleState,
        DRAFT_NOT_EDITABLE,
        "le brouillon est planifié ou en cours d'envoi : annulez l'envoi avant de le modifier",
    )
}

/// Ouvre la vue (`B`) sur le compte a1 et lui donne ces brouillons (jeton 1).
pub fn open_drafts_with(state: &mut AppState, drafts: Vec<Draft>) {
    press(state, Intent::Drafts(DraftsIntent::Open));
    update(state, drafts_reply(1, drafts));
}

/// État où ce brouillon enregistré a été rouvert depuis la vue (lecture jeton 1, relecture jeton 2).
pub fn reopen(draft: Draft) -> AppState {
    let mut state = state_with_mailboxes();
    open_drafts_with(&mut state, vec![draft.clone()]);
    press(&mut state, Intent::Open);
    update(&mut state, loaded_reply(2, Ok(draft)));
    state
}

/// Brouillon enregistré d1 à bob, objet « Re: Bonjour », corps « Corps ».
pub fn saved_draft() -> Draft {
    crate::compose_fixtures::draft("d1", &["bob@example.org"], "Re: Bonjour", "Corps")
}
