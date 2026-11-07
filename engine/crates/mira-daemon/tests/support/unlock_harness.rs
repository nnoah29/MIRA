//! Outils des tests d'attente du trousseau : contexte de serveur sans écoute et attente d'un état annoncé.

use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use mira_daemon::server::ServerContext;
use mira_daemon::token::Token;
use mira_host::Host;
use mira_protocol::HostState;

/// Contexte de serveur autonome (hôte vide, jeton jetable dans `dir`).
pub fn context(dir: &Path) -> Arc<ServerContext> {
    let token = Token::load_or_create(&dir.join("daemon.token")).unwrap();
    Arc::new(ServerContext::new(Arc::new(Host::new()), token))
}

/// Attend, au plus cinq secondes, que le contexte annonce cet état.
pub async fn wait_state(ctx: &ServerContext, wanted: HostState) {
    for _ in 0..250 {
        if ctx.host_state() == wanted {
            return;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    panic!("état {wanted:?} jamais annoncé");
}
