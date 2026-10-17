//! Bornes de lecture d'un serveur IMAP hostile ou défaillant : ligne, littéral, réponse entière, délai par bloc.

use std::time::Duration;

use mira_vocabulary::error::codes::server;
use mira_vocabulary::{ErrorCategory, MiraError};
use tokio::io::{AsyncBufReadExt, BufReader, ReadHalf};

use super::wire::{BoxedTransport, Wire};
use crate::imap::errors::network::{connection_lost, from_io, timeout};

pub(super) const MAX_LINE: usize = 1024 * 1024;
pub(super) const MAX_LITERAL: usize = 64 * 1024 * 1024;
pub(super) const MAX_RESPONSE: usize = 64 * 1024 * 1024;

/// Erreur de protocole : le serveur dépasse une borne de lecture
pub(super) fn limit_exceeded(what: &str) -> MiraError {
    MiraError::new(
        ErrorCategory::Server,
        server::PROTOCOL,
        format!("{what} trop gros dans la réponse du serveur"),
    )
}

/// Bloc d'octets disponible dans le flux ; le délai d'inactivité est réarmé à chaque bloc attendu
pub(super) async fn available(
    reader: &mut BufReader<ReadHalf<BoxedTransport>>,
    wait: Duration,
) -> Result<&[u8], MiraError> {
    match tokio::time::timeout(wait, reader.fill_buf()).await {
        Err(_) => Err(timeout()),
        Ok(Err(error)) => Err(from_io(&error)),
        Ok(Ok([])) => Err(connection_lost()),
        Ok(Ok(block)) => Ok(block),
    }
}

impl Wire {
    /// Compte des octets de plus dans la réponse en cours ; au-delà de la borne, le flux est abandonné
    pub(super) fn charge(&mut self, bytes: usize) -> Result<(), MiraError> {
        self.response_bytes = self.response_bytes.saturating_add(bytes);
        if self.response_bytes > MAX_RESPONSE {
            self.broken = true;
            return Err(limit_exceeded("réponse"));
        }
        Ok(())
    }

    /// Accepte l'annonce d'un littéral seulement s'il tient dans les bornes, avant de lire ou d'allouer quoi que ce soit
    pub(super) fn admit_literal(&mut self, size: usize) -> Result<(), MiraError> {
        let total = self.response_bytes.saturating_add(size);
        if size > MAX_LITERAL || total > MAX_RESPONSE {
            self.broken = true;
            return Err(limit_exceeded("littéral"));
        }
        Ok(())
    }
}
