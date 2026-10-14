//! Ce qu'un compte de test possède localement : réception, message, brouillon et action en attente.

use super::sb_world::SbWorld;

/// Identifiants des objets locaux d'un compte préparé pour un scénario.
#[derive(Debug, Clone)]
pub struct AccountParts {
    pub account: String,
    pub inbox: String,
    pub message: String,
    pub draft: String,
    pub action: String,
}

impl SbWorld {
    /// Exige que le compte, son miroir, son brouillon et sa file soient introuvables (code 11).
    pub async fn assert_parts_gone(&self, parts: &AccountParts) {
        let lines = [
            vec!["mailbox", "list", &parts.account],
            vec!["message", "get", &parts.message],
            vec!["draft", "get", &parts.draft],
            vec!["queue", "list", &parts.account],
        ];
        for line in lines {
            let gone = self.run(&line).await;
            assert_eq!(gone.code, 11, "{line:?} : {}", gone.stderr);
        }
    }

    /// Relève la réception et le message de ce sujet, crée un brouillon et soumet une action « lu » restée en attente.
    pub async fn parts_of(&self, account: &str, subject: &str) -> AccountParts {
        let inbox = self.mailbox_named(account, "INBOX").await;
        let message = self.message_id_of(&inbox, subject).await;
        let created = self.run_ok(&["draft", "create", account]).await;
        let draft = created["data"]["data"]["id"].as_str().unwrap().to_string();
        let action = self.apply_pending("mark-read", &message).await;
        AccountParts {
            account: account.to_string(),
            inbox,
            message,
            draft,
            action,
        }
    }
}
