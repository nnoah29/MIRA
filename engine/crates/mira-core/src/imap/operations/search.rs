//! Recherches de messages : par date, tous, par texte.

use mira_vocabulary::{MiraError, Timestamp};

use crate::imap::client::ImapClient;
use crate::imap::errors::status::Operation;
use crate::imap::parse::dates::format_search_date;
use crate::imap::parse::search::parse_search;
use crate::imap::protocol::quoting::quote;

impl ImapClient {
    /// Exécute une recherche par UID et rend les UID trouvés, triés.
    async fn search_uids(&mut self, criteria: &str) -> Result<Vec<u32>, MiraError> {
        self.require_selected()?;
        let response = self
            .run(&format!("UID SEARCH {criteria}"), Operation::Plain)
            .await?;
        let mut found: Vec<u32> = response
            .untagged
            .iter()
            .filter_map(|line| parse_search(line))
            .flatten()
            .collect();
        found.sort_unstable();
        found.dedup();
        Ok(found)
    }

    /// UID des messages reçus depuis le jour de l'instant donné (granularité d'un jour, UTC).
    pub async fn uids_since(&mut self, since: Timestamp) -> Result<Vec<u32>, MiraError> {
        self.search_uids(&format!("SINCE {}", format_search_date(since)))
            .await
    }

    /// UID des messages dont l'UID est supérieur ou égal à `first`.
    pub async fn uids_from(&mut self, first: u32) -> Result<Vec<u32>, MiraError> {
        let found = self.search_uids(&format!("UID {first}:*")).await?;
        Ok(found.into_iter().filter(|uid| *uid >= first).collect())
    }

    /// UID de tous les messages de la boîte sélectionnée.
    pub async fn all_uids(&mut self) -> Result<Vec<u32>, MiraError> {
        self.search_uids("ALL").await
    }

    /// UID des messages dont le texte contient la requête (recherche du serveur).
    pub async fn search_text(&mut self, query: &str) -> Result<Vec<u32>, MiraError> {
        self.search_uids(&format!("TEXT {}", quote(query)?)).await
    }

    /// UID des messages dont l'en-tête Message-ID contient l'identifiant donné (recherche du serveur).
    pub async fn uids_with_message_id(&mut self, message_id: &str) -> Result<Vec<u32>, MiraError> {
        self.search_uids(&format!("HEADER Message-ID {}", quote(message_id)?))
            .await
    }
}
