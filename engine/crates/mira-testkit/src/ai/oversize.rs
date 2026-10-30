//! Flux de commentaires SSE d'une taille exacte, pour éprouver les plafonds de lecture.

use std::io;

use tokio::io::{AsyncWrite, AsyncWriteExt};

use super::chunked::{write_chunk, write_partial};

const PIECE: usize = 4096;

/// Diffuse exactement `bytes` octets de commentaires SSE, morceau final compris
pub(super) async fn write_oversize<W: AsyncWrite + Unpin>(
    writer: &mut W,
    bytes: usize,
) -> io::Result<()> {
    write_partial(writer, &[], 0).await?;
    let mut remaining = bytes;
    while remaining > 0 {
        let size = remaining.min(PIECE);
        write_chunk(writer, &filler(size)).await?;
        remaining -= size;
    }
    writer.write_all(b"0\r\n\r\n").await?;
    writer.flush().await
}

/// Commentaire SSE de la taille donnée, ou lignes vides si elle est trop petite
fn filler(size: usize) -> Vec<u8> {
    if size < 4 {
        return vec![b'\n'; size];
    }
    let mut piece = vec![b'x'; size];
    piece[0] = b':';
    piece[size - 2] = b'\n';
    piece[size - 1] = b'\n';
    piece
}
