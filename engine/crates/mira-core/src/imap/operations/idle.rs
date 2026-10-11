//! Attente d'un changement de la boîte sélectionnée (IDLE).

use std::time::Duration;

use mira_vocabulary::MiraError;
use tokio::time::Instant;

use crate::imap::client::ImapClient;
use crate::imap::errors::status::{Operation, status_error};
use crate::imap::parse::idle::is_mailbox_change;
use crate::imap::parse::status::tagged_status;
use crate::imap::types::IdleOutcome;

impl ImapClient {
    /// Envoie IDLE et attend l'invite `+` du serveur.
    async fn start_idle(&mut self, tag: &str) -> Result<(), MiraError> {
        self.wire
            .write_bytes(format!("{tag} IDLE\r\n").as_bytes())
            .await?;
        loop {
            let reply = self.wire.read_line().await?;
            if let Some(status) = tagged_status(&reply, tag) {
                return Err(status_error(&status, Operation::Extension));
            }
            if reply.starts_with(b"+") {
                return Ok(());
            }
        }
    }

    /// Attend une annonce de changement ou la fin du délai.
    async fn watch_changes(&mut self, limit: Duration) -> Result<IdleOutcome, MiraError> {
        let deadline = Instant::now() + limit;
        loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if !self.wire.wait_readable(remaining).await? {
                return Ok(IdleOutcome::TimedOut);
            }
            let line = self.wire.read_line().await?;
            if is_mailbox_change(line.strip_prefix(b"* ").unwrap_or(&line)) {
                return Ok(IdleOutcome::Changed);
            }
        }
    }

    /// Reste en IDLE jusqu'à un changement de la boîte sélectionnée ou jusqu'au délai donné.
    pub async fn idle_until_change(&mut self, limit: Duration) -> Result<IdleOutcome, MiraError> {
        self.require_selected()?;
        self.require_capability("IDLE")?;
        let tag = self.wire.fresh_tag();
        self.start_idle(&tag).await?;
        let outcome = self.watch_changes(limit).await?;
        self.wire.write_bytes(b"DONE\r\n").await?;
        let closing = self.wire.read_until_tag(&tag).await?;
        let late = closing.untagged.iter().any(|l| is_mailbox_change(l));
        Ok(if late { IdleOutcome::Changed } else { outcome })
    }
}
