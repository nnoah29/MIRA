//! Jetons des lectures de la vue : une réponse ne compte que si son jeton est celui attendu.

use super::SettingsState;
use super::operation::Operation;

/// Compteur jamais remis à zéro et jetons attendus ; l'opération en cours survit à la fermeture de la vue.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Tokens {
    counter: u64,
    identities: Option<u64>,
    stats: Option<u64>,
    pub(super) operation: Option<(u64, Operation)>,
}

impl Tokens {
    /// Jeton neuf, jamais rendu auparavant dans la session.
    pub(super) fn next(&mut self) -> u64 {
        self.counter += 1;
        self.counter
    }

    /// Oublie les lectures attendues ; leurs réponses seront ignorées.
    pub(super) fn clear_reads(&mut self) {
        self.identities = None;
        self.stats = None;
    }
}

impl SettingsState {
    /// Note une lecture des identités et rend son jeton.
    pub fn begin_identities(&mut self) -> u64 {
        let token = self.tokens.next();
        self.tokens.identities = Some(token);
        token
    }

    /// Vrai (et oublié) si c'est la lecture des identités attendue.
    pub fn take_identities(&mut self, token: u64) -> bool {
        let awaited = self.tokens.identities == Some(token);
        if awaited {
            self.tokens.identities = None;
        }
        awaited
    }

    /// Note une lecture des volumes et rend son jeton.
    pub fn begin_stats(&mut self) -> u64 {
        let token = self.tokens.next();
        self.tokens.stats = Some(token);
        token
    }

    /// Vrai (et oublié) si c'est la lecture des volumes attendue.
    pub fn take_stats(&mut self, token: u64) -> bool {
        let awaited = self.tokens.stats == Some(token);
        if awaited {
            self.tokens.stats = None;
        }
        awaited
    }
}
