//! Ligne de commande du serveur : le gabarit du descripteur dont les jetons `{...}` sont remplacés.

use std::path::PathBuf;

use mira_vocabulary::error::codes::internal;
use mira_vocabulary::{AiLocalModel, AiLocalRuntime, ErrorCategory, MiraError};

/// Valeurs des jetons du gabarit pour un lancement.
#[derive(Debug, Clone)]
pub struct ArgValues {
    pub model: PathBuf,
    pub host: String,
    pub port: u16,
    pub api_key_file: PathBuf,
    pub threads: usize,
}

/// Nombre de threads : les cœurs moins deux, au moins deux, jamais plus que les cœurs.
pub fn thread_count(cores: usize) -> usize {
    cores.saturating_sub(2).max(2).min(cores.max(1))
}

/// Remplace les jetons du gabarit ; un jeton inconnu est une erreur interne (le descripteur est épinglé).
pub fn build_args(
    runtime: &AiLocalRuntime,
    model: &AiLocalModel,
    values: &ArgValues,
) -> Result<Vec<String>, MiraError> {
    let mut args = Vec::new();
    for item in &runtime.server_args {
        args.push(substitute(item, model, values)?);
    }
    if !model.reasoning {
        args.extend(runtime.reasoning_off_args.iter().cloned());
    }
    Ok(args)
}

/// Valeur d'un argument : littéral tel quel, jeton remplacé.
fn substitute(item: &str, model: &AiLocalModel, values: &ArgValues) -> Result<String, MiraError> {
    let Some(token) = item
        .strip_prefix('{')
        .and_then(|rest| rest.strip_suffix('}'))
    else {
        return Ok(item.to_string());
    };
    let text = |path: &PathBuf| path.to_string_lossy().into_owned();
    match token {
        "model" => Ok(text(&values.model)),
        "host" => Ok(values.host.clone()),
        "port" => Ok(values.port.to_string()),
        "api_key_file" => Ok(text(&values.api_key_file)),
        "ctx" => Ok(model.ctx_size.to_string()),
        "threads" => Ok(values.threads.to_string()),
        "alias" => Ok(model.id.clone()),
        _ => Err(MiraError::new(
            ErrorCategory::Internal,
            internal::BUG,
            "jeton inconnu dans la ligne de commande du serveur local",
        )),
    }
}
