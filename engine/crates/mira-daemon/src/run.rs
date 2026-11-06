//! Vie du démon : préparer, écouter, ouvrir le moteur, annoncer l'état, attendre l'ordre d'arrêt, fermer.

use std::process::ExitCode;
use std::sync::Arc;

use mira_host::Host;
use mira_protocol::HostState;
use serde_json::Value;

use crate::config::DaemonConfig;
use crate::report::{say, say_error};
use crate::server::ServerContext;
use crate::{listen, setup, shutdown, signals};

/// Code d'erreur du moteur quand un autre moteur tient déjà le profil (D-27).
const PROFILE_LOCKED: &str = "incompatible_state.profile_locked";

/// Fait vivre le démon jusqu'à son arrêt et rend le code de sortie.
pub async fn run(config: DaemonConfig) -> ExitCode {
    let setup = match setup::prepare(&config) {
        Ok(setup) => setup,
        Err(code) => return code,
    };
    let ctx = Arc::new(ServerContext::new(Arc::new(Host::new()), setup.token));
    let tasks = match listen::start(&config, &setup.paths, &ctx) {
        Ok(tasks) => tasks,
        Err(code) => return code,
    };
    let outcome = serve_until_stopped(&ctx, setup.engine).await;
    ctx.begin_shutdown();
    for task in tasks {
        let _ = task.await;
    }
    outcome
}

/// Ouvre le moteur, annonce `ready`, attend l'ordre d'arrêt puis ferme proprement.
async fn serve_until_stopped(
    ctx: &Arc<ServerContext>,
    engine: mira_assembly::api::EngineConfig,
) -> ExitCode {
    let opened: Value = serde_json::from_str(&ctx.host.open(engine)).unwrap_or(Value::Null);
    if opened["ok"] != true {
        return if opened["error"]["code"] == PROFILE_LOCKED {
            say_error("ce profil est déjà utilisé par un autre démon", 3)
        } else {
            say_error("ouverture du moteur impossible", 1)
        };
    }
    ctx.set_state(HostState::Ready);
    say("mira-daemon prêt");
    signals::wait_for_stop(ctx).await;
    shutdown::run(ctx.clone()).await;
    ExitCode::SUCCESS
}
