//! Commande DATA : lecture du message jusqu'à `.` seul et dé-bourrage (RFC 5321 4.1.1.4, 4.5.2).

use std::io;

use crate::smtp::session::{Flow, Session};

/// Accepte le message, l'enregistre puis répond selon les pannes armées
pub(crate) async fn handle(session: &mut Session) -> io::Result<Flow> {
    if !session.transaction.is_ready_for_data() {
        return session.answer("503 5.5.1 Bad sequence of commands").await;
    }
    session.reply("354 End data with <CR><LF>.<CR><LF>").await?;
    let data = read_body(session).await?;
    let mail = session.transaction.take_mail(data);
    if let Some(failure) = session.take_failure() {
        return session.answer(&failure).await;
    }
    let (id, drop_now) = session.shared.with(|state| {
        (
            state.record(mail),
            std::mem::take(&mut state.drop_after_data),
        )
    });
    if drop_now {
        return Ok(Flow::Close);
    }
    session
        .answer(&format!("250 2.0.0 OK queued as {id}"))
        .await
}

/// Lit les lignes du message jusqu'au point final en retirant le bourrage
async fn read_body(session: &mut Session) -> io::Result<Vec<u8>> {
    let mut data = Vec::new();
    loop {
        let line = session.read_raw_line().await?;
        if line == b".\r\n" || line == b".\n" {
            return Ok(data);
        }
        data.extend_from_slice(unstuff(&line));
    }
}

/// Retire un point de tête quand la ligne en commence par deux
fn unstuff(line: &[u8]) -> &[u8] {
    if line.starts_with(b"..") {
        line.get(1..).unwrap_or(line)
    } else {
        line
    }
}
