#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Si l'écriture vers le client échoue, la session se termine au lieu de rester ouverte et muette.

mod support;

use std::io::Cursor;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Duration;

use mira_daemon::server::ServerContext;
use mira_daemon::session;
use mira_daemon::token::Token;
use mira_daemon::transport::{Accepted, Trust};
use mira_host::Host;
use mira_protocol::{Frame, MAX_FRAME_BYTES, encode};
use serde_json::json;
use support::valid_hello;
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

/// Flux qui sert des octets fixes puis se tait, et dont les écritures échouent après la première.
struct DeafAfterHello {
    input: Cursor<Vec<u8>>,
    writes: usize,
}

impl AsyncRead for DeafAfterHello {
    /// Sert les octets préparés ; une fois épuisés, reste en attente
    fn poll_read(
        mut self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        let position = self.input.position() as usize;
        let rest = &self.input.get_ref()[position..];
        if rest.is_empty() {
            return Poll::Pending;
        }
        let taken = rest.len().min(buf.remaining());
        buf.put_slice(&rest[..taken]);
        self.input.set_position((position + taken) as u64);
        Poll::Ready(Ok(()))
    }
}

impl AsyncWrite for DeafAfterHello {
    /// La première écriture (`hello_ok`) passe, les suivantes échouent
    fn poll_write(
        mut self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        self.writes += 1;
        if self.writes == 1 {
            return Poll::Ready(Ok(buf.len()));
        }
        Poll::Ready(Err(std::io::ErrorKind::BrokenPipe.into()))
    }

    /// Rien à vider
    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Poll::Ready(Ok(()))
    }

    /// Rien à fermer
    fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Poll::Ready(Ok(()))
    }
}

/// Après un `call` dont la réponse ne peut pas être écrite, `session::run` rend la main.
#[tokio::test(flavor = "multi_thread")]
async fn a_failed_write_ends_the_session() {
    let dir = tempfile::tempdir().unwrap();
    let token = Token::load_or_create(&dir.path().join("t")).unwrap();
    let hello = valid_hello(token.as_str());
    let call = Frame::Call {
        id: 1,
        request: json!({}),
    };
    let mut bytes = encode(&hello, MAX_FRAME_BYTES).unwrap();
    bytes.extend(encode(&call, MAX_FRAME_BYTES).unwrap());
    let ctx = Arc::new(ServerContext::new(Arc::new(Host::new()), token));
    let stream = DeafAfterHello {
        input: Cursor::new(bytes),
        writes: 0,
    };
    let accepted = Accepted {
        stream: Box::new(stream),
        trust: Trust::LocalUser(None),
    };
    let ended = tokio::time::timeout(Duration::from_secs(3), session::run(accepted, ctx)).await;
    assert!(
        ended.is_ok(),
        "la session doit finir quand l'écriture échoue"
    );
}
