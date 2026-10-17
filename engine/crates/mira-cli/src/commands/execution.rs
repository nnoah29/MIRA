//! Exécution d'une commande : requête, ouverture du moteur, appel, rendu.

use std::time::Duration;

use mira_assembly::api::{Engine, Request, Response};
use mira_assembly::mira_vocabulary::{CONTRACT_VERSION, MiraError, ProfileId};
use mira_assembly::sync::BackgroundSync;

use super::background_policy::background_sync_for;
use super::request_builder::build_request;
use super::steps::run_steps;
use crate::argument_context::ArgumentContext;
use crate::arguments::Commands;
use crate::cli_io::CliIo;
use crate::deps::CliDeps;
use crate::json_envelope::to_json_value;
use crate::output::{OutputMode, report_error, report_success};
use crate::render::human_text;

/// Ouvre le moteur du profil (synchro de fond selon la commande), déroule la commande, puis le ferme.
async fn execute_request(
    request: Request,
    command: &Commands,
    profile: &str,
    io: &mut CliIo<'_>,
    deps: &CliDeps,
) -> Result<Response, MiraError> {
    let mut config = deps.engine_config(ProfileId::from_raw(profile));
    let configured = config.network.background_sync;
    let config_mode = background_sync_for(command, configured);
    config.network.background_sync = config_mode;
    let engine = Engine::open(config, CONTRACT_VERSION)?;
    let outcome = run_steps(&engine, command, request, io).await;
    close_engine(engine, config_mode).await;
    outcome
}

/// Ferme le moteur : arrêt propre des synchronisations si elles tournaient, sinon fermeture immédiate.
async fn close_engine(engine: Engine, mode: BackgroundSync) {
    match mode {
        BackgroundSync::On(_) => {
            let _ = engine.shutdown_within(Duration::from_secs(5)).await;
        }
        BackgroundSync::Off => {
            let _ = engine.shutdown();
        }
    }
}

/// Écrit la réponse : le contrat lui-même en JSON, un texte lisible sinon.
fn report_response(response: &Response, mode: OutputMode, io: &mut CliIo) -> i32 {
    match to_json_value(response) {
        Ok(data) => report_success(io, mode, data, &human_text(response)),
        Err(error) => report_error(io, mode, &error),
    }
}

/// Exécute une commande du contrat et rend son code de sortie.
pub async fn run_command(
    command: &Commands,
    profile: &str,
    mode: OutputMode,
    io: &mut CliIo<'_>,
    deps: &mut CliDeps,
) -> i32 {
    let built = {
        let mut context = ArgumentContext {
            stdin: &mut *deps.stdin,
            ids: deps.ids.as_ref(),
        };
        build_request(command, &mut context)
    };
    let outcome = match built {
        Ok(request) => execute_request(request, command, profile, io, deps).await,
        Err(error) => Err(error),
    };
    match outcome {
        Ok(response) => report_response(&response, mode, io),
        Err(error) => report_error(io, mode, &error),
    }
}
