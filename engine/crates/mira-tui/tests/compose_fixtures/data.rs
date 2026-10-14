//! Brouillons de test et états où l'éditeur est ouvert.
#![allow(dead_code)]

use super::commands::created_reply;
use crate::action_fixtures::state_with_mailboxes;
use crate::navigation_fixtures::press;
use mira_assembly::mira_vocabulary::{
    AccountId, AddressEntry, Draft, DraftId, DraftState, IdentityId, Timestamp,
};
use mira_tui::state::AppState;
use mira_tui::update::{ComposeIntent, Intent, update};

/// Brouillon du compte a1 avec ces destinataires (adresses seules), cet objet et ce corps.
pub fn draft(id: &str, to: &[&str], subject: &str, body: &str) -> Draft {
    let entry = |address: &&str| AddressEntry {
        name: None,
        address: address.to_string(),
    };
    Draft {
        id: DraftId::from_raw(id),
        account: AccountId::from_raw("a1"),
        identity: IdentityId::from_raw("i1"),
        to: to.iter().map(entry).collect(),
        cc: Vec::new(),
        bcc: Vec::new(),
        subject: subject.to_string(),
        body_text: body.to_string(),
        body_html: None,
        attachments: Vec::new(),
        in_reply_to: None,
        state: DraftState::Editing,
        updated_at: Timestamp::from_millis(0),
    }
}

/// État où « répondre » a été demandé puis où le moteur a rendu ce brouillon `d1`.
pub fn open_with(draft: Draft) -> AppState {
    let mut state = state_with_mailboxes();
    press(&mut state, Intent::Compose(ComposeIntent::Reply));
    update(&mut state, created_reply("tui-1-draft-1", draft));
    state
}

/// État où l'éditeur est ouvert sur la réponse `d1` à bob, objet « Re: Bonjour », corps « Corps ».
pub fn open_reply() -> AppState {
    open_with(draft("d1", &["bob@example.org"], "Re: Bonjour", "Corps"))
}

/// État où l'éditeur est ouvert et l'objet a reçu un « ! » pas encore enregistré.
pub fn open_modified() -> AppState {
    let mut state = open_reply();
    super::typing::focus_on(&mut state, mira_tui::state::compose::Field::Subject);
    super::typing::type_in_draft(&mut state, "!");
    state
}
