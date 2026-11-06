//! Démon jetable : un hôte, un jeton, une socket Unix dans un dossier temporaire et la tâche `serve`.

use std::path::PathBuf;
use std::sync::Arc;

use mira_assembly::api::EngineConfig;
use mira_daemon::server::{ServerContext, serve};
use mira_daemon::token::Token;
use mira_daemon::transport::unix::UnixSocketListener;
use mira_host::Host;
use mira_protocol::HostState;
use tokio::task::JoinHandle;

/// Démon de test ; l'arrêt du test abandonne la tâche et supprime le dossier.
pub struct TestDaemon {
    pub(super) dir: tempfile::TempDir,
    pub(super) socket: PathBuf,
    pub(super) token: String,
    pub(super) context: Arc<ServerContext>,
    pub(super) task: JoinHandle<()>,
}

impl TestDaemon {
    /// Démon dont le moteur est ouvert sur cette configuration, état `ready`.
    pub async fn start(config: EngineConfig) -> Self {
        Self::start_with(Some(config), |_| {}).await
    }

    /// Démon dont l'hôte n'a jamais ouvert de moteur, état `starting`.
    pub async fn start_closed() -> Self {
        Self::start_with(None, |_| {}).await
    }

    /// Comme `start`, en réglant le contexte (délais, tailles) avant d'écouter.
    pub async fn start_with(
        config: Option<EngineConfig>,
        tune: impl FnOnce(&mut ServerContext),
    ) -> Self {
        let dir = tempfile::tempdir().unwrap();
        let host = Arc::new(Host::new());
        let token = Token::load_or_create(&dir.path().join("daemon.token")).unwrap();
        let token_text = token.as_str().to_string();
        let mut context = ServerContext::new(host.clone(), token);
        tune(&mut context);
        if let Some(config) = config {
            let opened = host.open(config);
            assert!(opened.contains("\"ok\":true"), "{opened}");
            context.set_state(HostState::Ready);
        }
        let context = Arc::new(context);
        let socket = dir.path().join("run/mira.sock");
        let listener = UnixSocketListener::bind(&socket).unwrap();
        let task = tokio::spawn(serve(listener, context.clone()));
        Self {
            dir,
            socket,
            token: token_text,
            context,
            task,
        }
    }
}
