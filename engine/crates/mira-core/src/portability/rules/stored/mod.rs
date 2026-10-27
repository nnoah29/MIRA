//! Références lues dans la base : une erreur de stockage est retenue, puis rendue à l'appelant.

mod accounts;
mod labels;
mod mailboxes;
mod templates;

use std::cell::RefCell;

use mira_vocabulary::MiraError;

use crate::storage::Tx;

/// Résolution des références d'une règle dans la transaction en cours
#[derive(Debug)]
pub struct StoredRefs<'t, 'c> {
    tx: &'t Tx<'c>,
    failure: RefCell<Option<MiraError>>,
}

impl<'t, 'c> StoredRefs<'t, 'c> {
    /// Résolution sur cette transaction
    pub fn new(tx: &'t Tx<'c>) -> Self {
        Self {
            tx,
            failure: RefCell::new(None),
        }
    }

    /// Première erreur de stockage survenue depuis le dernier appel, s'il y en a une
    pub fn take_failure(&self) -> Option<MiraError> {
        self.failure.borrow_mut().take()
    }

    /// Valeur lue, ou `None` en retenant l'erreur de stockage
    fn absorb<T>(&self, read: Result<Option<T>, MiraError>) -> Option<T> {
        read.unwrap_or_else(|error| {
            self.failure.borrow_mut().get_or_insert(error);
            None
        })
    }
}
