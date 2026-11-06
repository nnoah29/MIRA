//! Codec des trames : longueur sur quatre octets grand-boutistes, puis le corps JSON.

use thiserror::Error;

use crate::Frame;

/// Taille maximale d'un corps de trame par défaut (16 Mio).
pub const MAX_FRAME_BYTES: u32 = 16 * 1024 * 1024;

/// Échec d'encodage ou de décodage d'une trame.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum CodecError {
    #[error("trame trop grande ({announced} octets, maximum {max})")]
    TooLarge { announced: u32, max: u32 },
    #[error("trame illisible")]
    Malformed,
    #[error("trame vide")]
    Empty,
}

/// Écrit la trame : quatre octets de longueur puis le JSON ; refuse un corps au-delà de `max`.
pub fn encode(frame: &Frame, max: u32) -> Result<Vec<u8>, CodecError> {
    let body = serde_json::to_vec(frame).map_err(|_| CodecError::Malformed)?;
    let announced = u32::try_from(body.len()).unwrap_or(u32::MAX);
    if announced > max {
        return Err(CodecError::TooLarge { announced, max });
    }
    let mut bytes = announced.to_be_bytes().to_vec();
    bytes.extend_from_slice(&body);
    Ok(bytes)
}

/// Longueur du corps annoncée par l'en-tête, vérifiée avant toute lecture du corps.
pub fn body_len(header: [u8; 4], max: u32) -> Result<usize, CodecError> {
    let announced = u32::from_be_bytes(header);
    match announced {
        0 => Err(CodecError::Empty),
        n if n > max => Err(CodecError::TooLarge { announced, max }),
        n => Ok(n as usize),
    }
}

/// Lit une trame depuis son corps JSON.
pub fn decode(body: &[u8]) -> Result<Frame, CodecError> {
    if body.is_empty() {
        return Err(CodecError::Empty);
    }
    serde_json::from_slice(body).map_err(|_| CodecError::Malformed)
}
