//! Plantage simulé et redémarrage du moteur : action laissée « en cours », envoi coupé après DATA.

use mira_vocabulary::{ActionId, ActionState, Draft, EventKind};

use super::fixture::Fixture;
use crate::support_api::queries::event_kinds;

impl Fixture {
    /// Envoi dont la connexion SMTP est coupée après DATA : le message part, le moteur ne le sait pas
    pub async fn interrupted_send(&self) -> (Draft, ActionId) {
        let draft = self.ready_draft().await;
        let action = self.send(&draft.id, "k-interrupted").await;
        self.smtp.drop_after_data_once();
        self.advance(10_001);
        self.wait_for_state(&action, ActionState::Uncertain).await;
        (draft, action)
    }

    /// Arrête le moteur, laisse l'action dans cet état comme après un plantage, puis rouvre le moteur
    pub fn crash_leaving(&mut self, action: &ActionId, state: ActionState) {
        if let Some(old) = self.engine_slot.take() {
            old.shutdown().unwrap();
        }
        self.db
            .write(|tx| tx.set_action_state(action, state))
            .unwrap();
        self.restart();
    }

    /// Valeurs de l'en-tête `Message-ID` des messages reçus par le faux serveur SMTP
    pub fn smtp_message_ids(&self) -> Vec<String> {
        let header = |data: &[u8]| {
            let text = String::from_utf8_lossy(data).to_string();
            let line = text
                .lines()
                .find(|line| line.to_ascii_lowercase().starts_with("message-id:"));
            line.unwrap_or_default().to_string()
        };
        let received = self.smtp.received();
        received.iter().map(|mail| header(&mail.data)).collect()
    }

    /// Nombre d'événements `SendUncertain` émis depuis l'ouverture du profil
    pub async fn uncertain_events(&self) -> usize {
        let kinds = event_kinds(self.engine()).await;
        let uncertain = |kind: &&EventKind| matches!(kind, EventKind::SendUncertain { .. });
        kinds.iter().filter(uncertain).count()
    }

    /// Attend que `SendUncertain` ait été émis au moins `count` fois
    pub async fn wait_uncertain_events(&self, count: usize) {
        for _ in 0..600 {
            if self.uncertain_events().await >= count {
                return;
            }
            tokio::time::sleep(std::time::Duration::from_millis(25)).await;
        }
        panic!("délai dépassé : {count} événements d'incertitude");
    }

    /// Redémarre le moteur avec cette action « en cours » et laisse le temps à la reprise
    pub async fn crash_in_progress(&mut self, action: &ActionId) {
        self.crash_leaving(action, ActionState::InProgress);
        tokio::time::sleep(std::time::Duration::from_millis(900)).await;
    }
}
