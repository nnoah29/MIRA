//! Récupération des drapeaux, éventuellement seulement ceux modifiés depuis un numéro (CONDSTORE).

use mira_vocabulary::MiraError;

use crate::imap::client::ImapClient;
use crate::imap::errors::status::Operation;
use crate::imap::parse::fetch::parse_fetch;
use crate::imap::parse::fetched::fetched_flags;
use crate::imap::protocol::quoting::uid_sets;
use crate::imap::types::FetchedFlags;

impl ImapClient {
    /// Récupère les drapeaux des messages ; avec `changed_since`, seulement ceux modifiés depuis ce numéro.
    pub async fn fetch_flags(
        &mut self,
        uids: &[u32],
        changed_since: Option<u64>,
    ) -> Result<Vec<FetchedFlags>, MiraError> {
        self.require_selected()?;
        let (modifier, operation) = match changed_since {
            Some(modseq) => {
                self.require_capability("CONDSTORE")?;
                (format!(" (CHANGEDSINCE {modseq})"), Operation::Extension)
            }
            None => (String::new(), Operation::Plain),
        };
        let mut found = Vec::new();
        for set in uid_sets(uids) {
            let line = format!("UID FETCH {set} (FLAGS){modifier}");
            let response = self.run(&line, operation).await?;
            let rows = response.untagged.iter().filter_map(|l| parse_fetch(l));
            found.extend(rows.filter_map(|data| fetched_flags(&data)));
        }
        Ok(found)
    }

    /// Garde, parmi les UID demandés, ceux que la boîte sélectionnée contient encore.
    pub async fn existing_uids(&mut self, uids: &[u32]) -> Result<Vec<u32>, MiraError> {
        let found = self.fetch_flags(uids, None).await?;
        Ok(found.into_iter().map(|flags| flags.uid).collect())
    }
}
