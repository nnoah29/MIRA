//! Préparation du démarrage : chemins, jeton et configuration du moteur, avant d'écouter ou d'ouvrir quoi que ce soit.

use std::process::ExitCode;

use mira_assembly::api::EngineConfig;
use mira_assembly::mira_vocabulary::ProfileId;
use mira_assembly::production_config;
use mira_assembly::sync::BackgroundSync;
use mira_host::platform::adapt;
use mira_host::polling::supervisor_for;

use crate::config::{DaemonConfig, DaemonPaths, environment_of, read_secrets_key, resolve_paths};
use crate::report::say_error;
use crate::token::Token;

/// Ce qu'il faut pour démarrer : chemins, jeton, configuration du moteur.
pub struct Setup {
    pub paths: DaemonPaths,
    pub token: Token,
    pub engine: EngineConfig,
}

/// Prépare le démarrage ; en cas d'échec, écrit un message fixe et rend le code de sortie 2.
pub fn prepare(config: &DaemonConfig) -> Result<Setup, ExitCode> {
    let runtime = std::env::var_os("XDG_RUNTIME_DIR")
        .filter(|dir| !dir.is_empty())
        .map(std::path::PathBuf::from);
    let paths = resolve_paths(config, &environment_of(config), runtime.as_deref())
        .map_err(|_| say_error("configuration invalide", 2))?;
    let token = Token::load_or_create(&paths.token)
        .map_err(|_| say_error("jeton local inutilisable", 2))?;
    let engine =
        engine_config(config).map_err(|_| say_error("configuration du moteur invalide", 2))?;
    Ok(Setup {
        paths,
        token,
        engine,
    })
}

/// Configuration de production, relevé périodique et adaptation (dossier de données, trousseau chiffré).
fn engine_config(
    config: &DaemonConfig,
) -> Result<EngineConfig, mira_assembly::mira_vocabulary::MiraError> {
    let mut engine = production_config(ProfileId::from_raw(&config.profile));
    engine.network.background_sync = BackgroundSync::On(supervisor_for(config.poll_secs));
    let key = config
        .secrets_key_file
        .as_deref()
        .map(read_secrets_key)
        .transpose()?;
    let home = config
        .home
        .as_ref()
        .map(|dir| dir.to_string_lossy().into_owned());
    adapt(&mut engine, home, key)?;
    Ok(engine)
}
