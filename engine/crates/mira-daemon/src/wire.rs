//! Lecture et écriture de trames sur un flux d'octets, avec la taille vérifiée avant le corps.

use mira_protocol::{CodecError, Frame, body_len, decode, encode};
use thiserror::Error;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

/// Échec de lecture ou d'écriture d'une trame.
#[derive(Debug, Error)]
pub enum WireError {
    #[error("trame trop grande ({announced} octets, maximum {max})")]
    TooLarge { announced: u32, max: u32 },
    #[error("trame illisible")]
    Malformed,
    #[error("trame coupée par la fin du flux")]
    Truncated,
    #[error("erreur d'entrée-sortie")]
    Io(#[from] std::io::Error),
}

impl From<CodecError> for WireError {
    /// Traduit une erreur de codec ; une trame vide est illisible
    fn from(error: CodecError) -> Self {
        match error {
            CodecError::TooLarge { announced, max } => Self::TooLarge { announced, max },
            CodecError::Malformed | CodecError::Empty => Self::Malformed,
        }
    }
}

/// Lit les quatre octets d'en-tête ; `None` si le flux se termine proprement avant le premier.
async fn read_header<R: AsyncRead + Unpin>(reader: &mut R) -> Result<Option<[u8; 4]>, WireError> {
    let mut header = [0u8; 4];
    if reader.read(&mut header[..1]).await? == 0 {
        return Ok(None);
    }
    match reader.read_exact(&mut header[1..]).await {
        Ok(_) => Ok(Some(header)),
        Err(error) if error.kind() == std::io::ErrorKind::UnexpectedEof => {
            Err(WireError::Truncated)
        }
        Err(error) => Err(error.into()),
    }
}

/// Lit une trame ; `None` sur fin propre. La taille annoncée est jugée avant de lire le corps.
pub async fn read_frame<R: AsyncRead + Unpin>(
    reader: &mut R,
    max: u32,
) -> Result<Option<Frame>, WireError> {
    let Some(header) = read_header(reader).await? else {
        return Ok(None);
    };
    let length = body_len(header, max)?;
    let mut body = Vec::new();
    reader.take(length as u64).read_to_end(&mut body).await?;
    if body.len() < length {
        return Err(WireError::Truncated);
    }
    Ok(Some(decode(&body)?))
}

/// Écrit une trame (longueur puis JSON) et vide le tampon.
pub async fn write_frame<W: AsyncWrite + Unpin>(
    writer: &mut W,
    frame: &Frame,
    max: u32,
) -> Result<(), WireError> {
    let bytes = encode(frame, max)?;
    writer.write_all(&bytes).await?;
    writer.flush().await?;
    Ok(())
}
