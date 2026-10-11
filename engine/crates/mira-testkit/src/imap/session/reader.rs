//! Lecture d'une commande complète, littéraux compris.

use std::io;

use super::literal_marker::literal_marker;
use super::transport::Transport;
use crate::imap::capabilities::{Capability, CapabilitySet};

const MAX_LITERAL: usize = 32 * 1024 * 1024;

/// Commande lue sur la socket, littéraux inclus à leur place.
#[derive(Debug)]
pub(crate) struct RawCommand {
    pub bytes: Vec<u8>,
    pub first_line: String,
    pub error: Option<String>,
}

/// Lit la prochaine commande ; `None` si le client a fermé la connexion
pub(crate) async fn read_command(
    io: &mut Transport,
    caps: &CapabilitySet,
) -> io::Result<Option<RawCommand>> {
    let mut line = Vec::new();
    if io.read_line_into(&mut line).await? == 0 {
        return Ok(None);
    }
    let first_line = String::from_utf8_lossy(&line).trim_end().to_string();
    let mut command = RawCommand {
        bytes: line.clone(),
        first_line,
        error: None,
    };
    while let Some((size, plus)) = literal_marker(&line) {
        line = read_literal_part(io, caps, &mut command, size, plus).await?;
    }
    Ok(Some(command))
}

/// Lit un littéral et la suite de la ligne ; renvoie cette suite
async fn read_literal_part(
    io: &mut Transport,
    caps: &CapabilitySet,
    command: &mut RawCommand,
    size: usize,
    plus: bool,
) -> io::Result<Vec<u8>> {
    if size > MAX_LITERAL {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "literal too large",
        ));
    }
    if plus && !caps.contains(Capability::Literalplus) {
        command.error = Some("LITERAL+ not supported".to_string());
    } else if !plus {
        io.send(b"+ Ready\r\n").await?;
    }
    let data = io.read_exact_bytes(size).await?;
    if command.error.is_none() {
        command.bytes.extend_from_slice(&data);
    }
    let mut next = Vec::new();
    if io.read_line_into(&mut next).await? == 0 {
        return Err(io::ErrorKind::UnexpectedEof.into());
    }
    command.bytes.extend_from_slice(&next);
    Ok(next)
}
