//! Vue « Brouillons » d'un compte : brouillons lus, sélection ancrée, lecture, réouverture et suppression en cours.

pub mod anchor;
pub mod listing;
pub mod pending;

use mira_assembly::mira_vocabulary::{AccountId, Draft, DraftId};

use super::selection::Selection;

/// Vue des brouillons ; fermée tant qu'aucun compte n'y est ouvert. Le compteur de jetons ne revient jamais en arrière.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DraftsState {
    account: Option<AccountId>,
    items: Vec<Draft>,
    pub selection: Selection,
    anchor: Option<DraftId>,
    tokens: u64,
    listing: Option<u64>,
    again: bool,
    loaded: bool,
    opening: Option<u64>,
    deleting: Option<(DraftId, String)>,
}

impl DraftsState {
    /// Ouvre la vue, vide, sur ce compte.
    pub fn open(&mut self, account: AccountId) {
        self.reset();
        self.account = Some(account);
    }

    /// Ferme la vue ; les réponses encore attendues seront ignorées.
    pub fn close(&mut self) {
        self.reset();
    }

    /// Vrai si la vue est ouverte.
    pub fn is_open(&self) -> bool {
        self.account.is_some()
    }

    /// Compte de la vue ouverte.
    pub fn account(&self) -> Option<&AccountId> {
        self.account.as_ref()
    }

    /// Jeton neuf, jamais rendu auparavant dans la session.
    fn next_token(&mut self) -> u64 {
        self.tokens += 1;
        self.tokens
    }

    /// Remet tout à zéro sauf le compteur de jetons.
    fn reset(&mut self) {
        let tokens = self.tokens;
        *self = DraftsState::default();
        self.tokens = tokens;
    }
}
