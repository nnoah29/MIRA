//! API publique d'injection de pannes et de lecture du journal.

use super::handle::FakeMailStore;

impl FakeMailStore {
    /// Fait répondre `response` (après l'étiquette) à la prochaine commande `command`
    pub fn fail_next(&self, command: &str, response: &str) {
        self.shared.faults.push_failure(command, response);
    }

    /// Commandes reçues, sans mot de passe
    pub fn command_log(&self) -> Vec<String> {
        self.shared.faults.snapshot()
    }

    /// Retire la panne programmée pour l'une des clés de commande
    pub(crate) fn take_failure(&self, keys: &[String]) -> Option<String> {
        self.shared.faults.take_failure(keys)
    }

    /// Ajoute une ligne au journal des commandes
    pub(crate) fn record_command(&self, line: String) {
        self.shared.faults.record(line);
    }
}
