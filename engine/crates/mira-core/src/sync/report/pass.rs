//! Passes de synchronisation et ruptures de session dans le rapport.

use mira_vocabulary::{MiraError, Timestamp};

use super::handle::ReportHandle;

impl ReportHandle {
    /// Note qu'une passe de synchronisation commence.
    pub fn pass_started(&self) {
        self.update(|state| state.running = true);
    }

    /// Note la fin d'une passe : dernier succès, ou code de l'erreur.
    pub fn pass_ended(&self, now: Timestamp, outcome: &Result<(), MiraError>) {
        self.update(|state| {
            state.running = false;
            state.mailboxes.clear();
            match outcome {
                Ok(()) => {
                    state.last_success = Some(now);
                    state.last_error = None;
                }
                Err(error) => state.last_error = Some(error.code.clone()),
            }
        });
    }

    /// Note qu'une session s'est rompue (connexion refusée ou perdue).
    pub fn broke(&self, error: &MiraError) {
        self.update(|state| {
            state.running = false;
            state.mailboxes.clear();
            state.last_error = Some(error.code.clone());
        });
    }
}
