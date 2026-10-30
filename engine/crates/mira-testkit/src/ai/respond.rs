//! Aiguillage d'un comportement vers l'écriture de la réponse correspondante.

use std::io;

use tokio::io::{AsyncWrite, AsyncWriteExt};

use super::behavior::Behavior;
use super::chunked::{write_partial, write_stream};
use super::oversize::write_oversize;
use super::plain::write_plain;

/// Joue le comportement ; `Ok` signifie que le serveur a terminé ou coupé volontairement
pub(super) async fn respond<W: AsyncWrite + Unpin>(
    writer: &mut W,
    behavior: Behavior,
) -> io::Result<()> {
    match behavior {
        Behavior::Stream {
            chunks,
            chunk_delay_ms,
        } => write_stream(writer, &chunks, chunk_delay_ms).await,
        Behavior::Status {
            code,
            headers,
            body,
        } => write_plain(writer, code, &headers, &body).await,
        Behavior::Hang => std::future::pending().await,
        Behavior::DropAfter { chunks } => write_partial(writer, &chunks, 0).await,
        Behavior::Redirect { to } => {
            let headers = [("Location".to_string(), to)];
            write_plain(writer, 302, &headers, "").await
        }
        Behavior::Models { json } => write_plain(writer, 200, &[], &json).await,
        Behavior::Oversize { bytes } => write_oversize(writer, bytes).await,
    }
}

/// Ferme le côté écriture pour signaler la fin de la réponse
pub(super) async fn finish<W: AsyncWrite + Unpin>(writer: &mut W) {
    let _ = writer.shutdown().await;
}
