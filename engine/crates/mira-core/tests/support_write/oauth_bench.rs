//! Lectures et commandes du banc OAuth : états relus en base, serveur, marquage lu par la façade.

use mira_core::api::{Response, WriteResponse};
use mira_vocabulary::{AccountState, ActionId, ActionKind, ActionState};

use super::builders::apply_request;
use super::oauth::OauthBench;
use crate::support_api::oauth::OAUTH_USER;

impl OauthBench {
    /// État d'une action relu en base
    pub fn state_of(&self, action: &ActionId) -> ActionState {
        let found = self.db.read(|tx| tx.get_action(action)).unwrap();
        found.unwrap().state
    }

    /// État de connexion du compte relu en base
    pub fn account_state(&self) -> AccountState {
        let found = self.db.read(|tx| tx.get_account(&self.account));
        found.unwrap().unwrap().state
    }

    /// Dit si le message de la réception est marqué lu sur le serveur
    pub fn server_seen(&self, uid: u32) -> bool {
        let views = self.api.store.messages(OAUTH_USER, "INBOX");
        let found = views.into_iter().find(|view| view.uid == uid);
        found.is_some_and(|view| view.flags.iter().any(|flag| flag.contains("Seen")))
    }

    /// Marque le message lu par la façade et rend l'action mise en file
    pub async fn mark_read(&self, id: &str, key: &str) -> ActionId {
        let request = apply_request(ActionKind::MarkRead { read: true }, &[id], key);
        match self.api.engine.execute(request).await.unwrap() {
            Response::Write(WriteResponse::Actions(actions)) => actions[0].clone(),
            other => panic!("réponse inattendue : {other:?}"),
        }
    }
}
