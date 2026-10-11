//! Lecture des lignes logiques : lignes physiques et littéraux réunis.

use mira_vocabulary::error::codes::server;
use mira_vocabulary::{ErrorCategory, MiraError};
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncReadExt};

use super::literal::literal_size;
use super::wire::Wire;
use crate::imap::errors::network::{connection_lost, from_io, timeout};

const MAX_LITERAL: usize = 128 * 1024 * 1024;

impl Wire {
    /// Lit une ligne physique sans sa fin de ligne ; les octets déjà lus survivent à une annulation.
    async fn read_physical(&mut self) -> Result<Vec<u8>, MiraError> {
        let read = self
            .reader
            .read_until(b'\n', &mut self.pending)
            .await
            .map_err(|error| from_io(&error))?;
        if read == 0 {
            return Err(connection_lost());
        }
        let mut line = std::mem::take(&mut self.pending);
        while matches!(line.last(), Some(b'\n' | b'\r')) {
            line.pop();
        }
        Ok(line)
    }

    /// Lit une ligne logique : chaque littéral est replacé à sa position, précédé de `{n}` et CRLF.
    pub(crate) async fn read_logical(&mut self) -> Result<Vec<u8>, MiraError> {
        let mut line = self.read_physical().await?;
        let mut announced = literal_size(&line);
        while let Some(size) = announced {
            if size > MAX_LITERAL {
                return Err(MiraError::new(
                    ErrorCategory::Server,
                    server::PROTOCOL,
                    "littéral annoncé par le serveur trop gros",
                ));
            }
            line.extend_from_slice(b"\r\n");
            let start = line.len();
            line.resize(start + size, 0);
            self.reader
                .read_exact(&mut line[start..])
                .await
                .map_err(|error| from_io(&error))?;
            let more = self.read_physical().await?;
            announced = literal_size(&more);
            line.extend_from_slice(&more);
        }
        Ok(line)
    }

    /// Lit une ligne logique dans le délai d'attente du flux.
    pub(crate) async fn read_line(&mut self) -> Result<Vec<u8>, MiraError> {
        match tokio::time::timeout(self.wait, self.read_logical()).await {
            Ok(result) => result,
            Err(_) => Err(timeout()),
        }
    }

    /// Attend que des octets soient disponibles ; `false` si le délai donné est écoulé.
    pub(crate) async fn wait_readable(&mut self, limit: Duration) -> Result<bool, MiraError> {
        match tokio::time::timeout(limit, self.reader.fill_buf()).await {
            Err(_) => Ok(false),
            Ok(Err(error)) => Err(from_io(&error)),
            Ok(Ok([])) => Err(connection_lost()),
            Ok(Ok(_)) => Ok(true),
        }
    }
}
