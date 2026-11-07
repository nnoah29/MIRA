//! Vie du démon : préparer, écouter, ouvrir le moteur, annoncer l'état, attendre l'ordre d'arrêt, fermer.

use std::process::ExitCode;
use std::sync::Arc;

use mira_assembly::mira_vocabulary::error::codes::incompatible_state;
use mira_host::Host;
use mira_protocol::HostState;
use serde_json::Value;

use crate::config::DaemonConfig;
use crate::report::{say, say_error};
use crate::server::ServerContext;
use crate::unlock::UnlockBackoff;
use crate::{idle, listen, setup, shutdown, signals, unlock};

/// Fait vivre le démon jusqu'à son arrêt et rend le code de sortie.
pub async fn run(config: DaemonConfig) -> ExitCode {
    let config = match setup::load_config(config) {
        Ok(config) => config,
        Err(code) => return code,
    };
    let setup = match setup::prepare(&config) {
        Ok(setup) => setup,
        Err(code) => return code,
    };
    let ctx = Arc::new(ServerContext::new(Arc::new(Host::new()), setup.token));
    let tasks = match listen::start(&config, &setup.paths, &ctx) {
        Ok(tasks) => tasks,
        Err(code) => return code,
    };
    let outcome = serve_until_stopped(&ctx, setup.engine, config.idle_grace()).await;
    ctx.begin_shutdown();
    for task in tasks {
        let _ = task.await;
    }
    outcome
}

/// Ouvre le moteur en attendant le trousseau ; faux si l'arrêt est demandé pendant l'attente.
async fn open_engine(
    ctx: &Arc<ServerContext>,
    engine: mira_assembly::api::EngineConfig,
) -> Result<bool, ExitCode> {
    let waiting = unlock::open_when_unlocked(ctx, engine, UnlockBackoff::default());
    let envelope = tokio::select! {
        opened = waiting => opened,
        _ = signals::wait_for_stop(ctx) => None,
    };
    let Some(envelope) = envelope else {
        return Ok(false);
    };
    let opened: Value = serde_json::from_str(&envelope).unwrap_or(Value::Null);
    if opened["ok"] == true {
        return Ok(true);
    }
    Err(
        if opened["error"]["code"] == incompatible_state::PROFILE_LOCKED.0 {
            say_error("ce profil est déjà utilisé par un autre démon", 3)
        } else {
            say_error("ouverture du moteur impossible", 1)
        },
    )
}

/// Ouvre le moteur, annonce `ready`, attend la fin du démon puis ferme proprement.
async fn serve_until_stopped(
    ctx: &Arc<ServerContext>,
    engine: mira_assembly::api::EngineConfig,
    idle_grace: Option<std::time::Duration>,
) -> ExitCode {
    match open_engine(ctx, engine).await {
        Ok(true) => {}
        Ok(false) => return ExitCode::SUCCESS,
        Err(code) => return code,
    }
    ctx.set_state(HostState::Ready);
    say("mira-daemon prêt");
    idle::wait_for_end(ctx, idle_grace).await;
    let _ = shutdown::run(ctx.clone()).await;
    ExitCode::SUCCESS
}
