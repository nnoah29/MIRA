//! Lecture des lignes logiques : lignes physiques et littéraux réunis, mémoire bornée, délai d'inactivité par bloc.

use std::time::Duration;

use mira_vocabulary::MiraError;
use tokio::io::AsyncBufReadExt;

use super::limits::{MAX_LINE, available, limit_exceeded};
use super::literal::literal_size;
use super::wire::Wire;
use crate::imap::errors::network::{connection_lost, from_io};

impl Wire {
    /// Lit une ligne physique sans sa fin de ligne ; les octets déjà lus survivent à une annulation.
    async fn read_physical(&mut self) -> Result<Vec<u8>, MiraError> {
        loop {
            let block = available(&mut self.reader, self.wait).await?;
            let end = block.iter().position(|byte| *byte == b'\n');
            let used = end.map_or(block.len(), |index| index + 1);
            self.pending.extend_from_slice(&block[..used]);
            self.reader.consume(used);
            if self.pending.len() > MAX_LINE {
                self.pending.clear();
                self.broken = true;
                return Err(limit_exceeded("ligne"));
            }
            if end.is_some() {
                break;
            }
        }
        let mut line = std::mem::take(&mut self.pending);
        while matches!(line.last(), Some(b'\n' | b'\r')) {
            line.pop();
        }
        Ok(line)
    }

    /// Lit les `size` octets d'un littéral par blocs, sans réserver à l'avance la taille annoncée.
    async fn read_literal(&mut self, size: usize, line: &mut Vec<u8>) -> Result<(), MiraError> {
        let mut remaining = size;
        while remaining > 0 {
            let block = available(&mut self.reader, self.wait).await?;
            let used = block.len().min(remaining);
            line.extend_from_slice(&block[..used]);
            self.reader.consume(used);
            remaining -= used;
            self.charge(used)?;
        }
        Ok(())
    }

    /// Lit une ligne logique : chaque littéral est replacé à sa position, précédé de `{n}` et CRLF.
    pub(crate) async fn read_logical(&mut self) -> Result<Vec<u8>, MiraError> {
        if self.broken {
            return Err(connection_lost());
        }
        let mut line = self.read_physical().await?;
        self.charge(line.len())?;
        if literal_size(&line).is_none() {
            return Ok(line);
        }
        let outcome = self.read_literals(&mut line).await;
        if outcome.is_err() {
            self.broken = true;
        }
        outcome.map(|()| line)
    }

    /// Lit les littéraux annoncés en fin de ligne, chacun suivi de la suite de la ligne.
    async fn read_literals(&mut self, line: &mut Vec<u8>) -> Result<(), MiraError> {
        let mut announced = literal_size(line);
        while let Some(size) = announced {
            self.admit_literal(size)?;
            line.extend_from_slice(b"\r\n");
            self.read_literal(size, line).await?;
            let more = self.read_physical().await?;
            self.charge(more.len())?;
            announced = literal_size(&more);
            line.extend_from_slice(&more);
        }
        Ok(())
    }

    /// Lit une ligne logique ; le délai d'attente du flux s'applique à chaque bloc reçu, pas à la ligne entière.
    pub(crate) async fn read_line(&mut self) -> Result<Vec<u8>, MiraError> {
        self.read_logical().await
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
