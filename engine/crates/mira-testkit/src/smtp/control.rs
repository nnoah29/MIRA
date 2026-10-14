//! Pilotage du faux serveur SMTP par les tests : lecture des messages et pannes.

use super::mail::ReceivedMail;
use super::server::FakeSmtpServer;

impl FakeSmtpServer {
    /// Messages acceptés jusqu'ici, dans l'ordre d'arrivée
    pub fn received(&self) -> Vec<ReceivedMail> {
        self.shared.with(|state| state.received.clone())
    }

    /// Arme une réponse d'erreur pour la prochaine commande MAIL, RCPT ou fin de DATA
    pub fn fail_next(&self, code_and_text: &str) {
        let reply = code_and_text.to_string();
        self.shared.with(|state| state.failures.push_back(reply));
    }

    /// Enregistre le prochain message puis coupe la connexion sans répondre
    pub fn drop_after_data_once(&self) {
        self.shared.with(|state| state.drop_after_data = true);
    }

    /// Enregistre le prochain message puis se tait : aucune réponse de fin de DATA n'arrive
    pub fn stall_after_data_once(&self) {
        self.shared.with(|state| state.stall_after_data = true);
    }
}
