//! Dialogue IDLE : annonces spontanées jusqu'à DONE.

use std::io;

use super::state::SessionState;
use super::sync::sync_selected;
use super::transport::Transport;

/// Événement qui réveille la boucle IDLE.
#[derive(Debug)]
enum IdleEvent {
    Input(usize),
    Changed(bool),
}

/// Envoie les changements de la boîte sélectionnée, s'il y en a
async fn send_sync(io: &mut Transport, state: &mut SessionState) -> io::Result<()> {
    let reply = sync_selected(state);
    if reply.out.is_empty() {
        return Ok(());
    }
    io.send(&reply.out).await
}

/// Reste en IDLE jusqu'à `DONE`, en annonçant les changements du magasin
pub(crate) async fn run_idle(
    io: &mut Transport,
    state: &mut SessionState,
    tag: &str,
) -> io::Result<()> {
    let mut changes = state.ctx.store.subscribe();
    changes.borrow_and_update();
    send_sync(io, state).await?;
    let mut line = Vec::new();
    loop {
        let event = tokio::select! {
            read = io.read_line_into(&mut line) => IdleEvent::Input(read?),
            changed = changes.changed() => IdleEvent::Changed(changed.is_ok()),
        };
        match event {
            IdleEvent::Input(0) | IdleEvent::Changed(false) => return Ok(()),
            IdleEvent::Input(_)
                if String::from_utf8_lossy(&line)
                    .trim()
                    .eq_ignore_ascii_case("DONE") =>
            {
                break;
            }
            IdleEvent::Input(_) => line.clear(),
            IdleEvent::Changed(true) => send_sync(io, state).await?,
        }
    }
    io.send(format!("{tag} OK IDLE terminated\r\n").as_bytes())
        .await
}
