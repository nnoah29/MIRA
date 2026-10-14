//! Exécution d'une commande : requête, ouverture du moteur, appel, rendu.

use mira_assembly::api::{Engine, Request, Response};
use mira_assembly::mira_vocabulary::{CONTRACT_VERSION, MiraError, ProfileId};

use super::request_builder::build_request;
use crate::argument_context::ArgumentContext;
use crate::arguments::Commands;
use crate::cli_io::CliIo;
use crate::deps::CliDeps;
use crate::json_envelope::to_json_value;
use crate::output::{OutputMode, report_error, report_success};
use crate::render::human_text;

/// Ouvre le moteur du profil, exécute la requête, puis ferme le moteur.
async fn execute_request(
    request: Request,
    profile: &str,
    deps: &CliDeps,
) -> Result<Response, MiraError> {
    let config = deps.engine_config(ProfileId::from_raw(profile));
    let engine = Engine::open(config, CONTRACT_VERSION)?;
    let outcome = engine.execute(request).await;
    let _ = engine.shutdown();
    outcome
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
        Ok(request) => execute_request(request, profile, deps).await,
        Err(error) => Err(error),
    };
    match outcome {
        Ok(response) => report_response(&response, mode, io),
        Err(error) => report_error(io, mode, &error),
    }
}
