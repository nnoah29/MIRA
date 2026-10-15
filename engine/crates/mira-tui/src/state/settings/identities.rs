//! Identités du compte visé : liste lue, sélection ancrée sur l'identifiant.

use mira_assembly::mira_vocabulary::{AccountId, Identity, IdentityId};

use crate::state::Selection;

/// Identités d'un compte, telles que lues par le moteur.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct IdentitiesState {
    account: Option<AccountId>,
    items: Vec<Identity>,
    pub selection: Selection,
    anchor: Option<IdentityId>,
    loaded: bool,
}

impl IdentitiesState {
    /// Prépare une lecture pour ce compte ; la liste d'un autre compte est vidée.
    pub fn prepare_for(&mut self, account: &AccountId) {
        if self.account.as_ref() != Some(account) {
            *self = IdentitiesState::default();
            self.account = Some(account.clone());
        }
    }

    /// Remplace la liste lue ; la sélection reste sur la même identité si elle existe encore.
    pub fn replace(&mut self, items: Vec<Identity>) {
        self.items = items;
        self.loaded = true;
        let kept = self
            .anchor
            .as_ref()
            .and_then(|anchor| self.items.iter().position(|i| &i.id == anchor));
        match kept {
            Some(index) => self.selection.selected = Some(index),
            None => self.selection.clamp_to(self.items.len()),
        }
        self.anchor = self.selected().map(|identity| identity.id.clone());
    }

    /// Déplace la sélection et ancre l'identité choisie.
    pub fn move_selection(&mut self, step: fn(&mut Selection, usize)) {
        step(&mut self.selection, self.items.len());
        self.anchor = self.selected().map(|identity| identity.id.clone());
    }

    /// Identités affichées.
    pub fn items(&self) -> &[Identity] {
        &self.items
    }

    /// Identité sélectionnée.
    pub fn selected(&self) -> Option<&Identity> {
        self.selection.selected.and_then(|i| self.items.get(i))
    }

    /// Vrai une fois une lecture reçue pour le compte visé.
    pub fn is_loaded(&self) -> bool {
        self.loaded
    }
}
