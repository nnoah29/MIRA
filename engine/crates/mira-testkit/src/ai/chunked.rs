//! Réponse `text/event-stream` découpée en morceaux HTTP, écrite et vidée morceau par morceau.

use std::io;
use std::time::Duration;

use tokio::io::{AsyncWrite, AsyncWriteExt};
use tokio::time::sleep;

const HEAD: &[u8] = b"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-cache\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n";

/// Écrit les en-têtes de flux puis chaque morceau, avec un délai entre deux morceaux
pub(super) async fn write_stream<W: AsyncWrite + Unpin>(
    writer: &mut W,
    chunks: &[Vec<u8>],
    chunk_delay_ms: u64,
) -> io::Result<()> {
    write_partial(writer, chunks, chunk_delay_ms).await?;
    writer.write_all(b"0\r\n\r\n").await?;
    writer.flush().await
}

/// Écrit les en-têtes de flux et les morceaux sans le morceau final
pub(super) async fn write_partial<W: AsyncWrite + Unpin>(
    writer: &mut W,
    chunks: &[Vec<u8>],
    chunk_delay_ms: u64,
) -> io::Result<()> {
    writer.write_all(HEAD).await?;
    writer.flush().await?;
    for (index, chunk) in chunks.iter().enumerate() {
        if index > 0 && chunk_delay_ms > 0 {
            sleep(Duration::from_millis(chunk_delay_ms)).await;
        }
        write_chunk(writer, chunk).await?;
    }
    Ok(())
}

/// Écrit un morceau HTTP (taille hexadécimale, données) et le vide vers le client
pub(super) async fn write_chunk<W: AsyncWrite + Unpin>(
    writer: &mut W,
    chunk: &[u8],
) -> io::Result<()> {
    let mut frame = format!("{:x}\r\n", chunk.len()).into_bytes();
    frame.extend_from_slice(chunk);
    frame.extend_from_slice(b"\r\n");
    writer.write_all(&frame).await?;
    writer.flush().await
}
