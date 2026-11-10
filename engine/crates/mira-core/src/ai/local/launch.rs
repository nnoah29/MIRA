//! Lancement d'un serveur : modèle présent, port libre, clé neuve, processus, attente de l'état prêt.

use std::time::Duration;

use mira_vocabulary::error::codes::incompatible_state;
use mira_vocabulary::{AiLocalModel, ErrorCategory, MiraError};
use tokio::process::Child;

use super::download::io_error;
use super::key_file::{new_key, write_key_file};
use super::launch_args::{ArgValues, build_args, thread_count};
use super::process::{spawn, terminate, wait_ready};
use super::running::Running;
use super::runtime::RuntimeContext;

/// Réserve un port libre de la boucle locale (il peut être repris entre-temps : risque accepté).
async fn free_port() -> Result<u16, MiraError> {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(|_| io_error())?;
    listener
        .local_addr()
        .map(|a| a.port())
        .map_err(|_| io_error())
}

/// Chemin du fichier du modèle, ou `incompatible_state.ai_model_missing` s'il n'est pas installé.
fn model_file(
    ctx: &RuntimeContext<'_>,
    model: &AiLocalModel,
) -> Result<std::path::PathBuf, MiraError> {
    let path = ctx.models_dir.join(&model.file_name);
    if path.is_file() {
        return Ok(path);
    }
    Err(MiraError::new(
        ErrorCategory::IncompatibleState,
        incompatible_state::AI_MODEL_MISSING,
        "le modèle local n'est pas installé",
    ))
}

/// Dit si la mémoire disponible est sous la valeur conseillée pour ce modèle.
fn is_low_memory(ctx: &RuntimeContext<'_>, model: &AiLocalModel) -> bool {
    ctx.resources
        .available_memory_mib()
        .is_some_and(|available| available < u64::from(model.ram_recommended_mib))
}

/// Lance le serveur du modèle et attend qu'il réponde ; en cas d'échec le processus est arrêté.
pub(super) async fn launch(
    ctx: &RuntimeContext<'_>,
    model: &AiLocalModel,
) -> Result<Running, MiraError> {
    let model_path = model_file(ctx, model)?;
    let (port, key) = (free_port().await?, new_key());
    let cores = std::thread::available_parallelism().map_or(2, usize::from);
    let values = ArgValues {
        model: model_path,
        host: "127.0.0.1".to_string(),
        port,
        api_key_file: write_key_file(&ctx.run_dir, &key).await?,
        threads: thread_count(cores),
    };
    let args = build_args(ctx.runtime, model, &values)?;
    let child = spawn(&ctx.executable, &args)?;
    let root_url = format!("http://127.0.0.1:{port}");
    wait_and_wrap(ctx, model, child, root_url, key).await
}

/// Attend l'état prêt ; en cas d'échec le processus est arrêté, sinon le serveur en marche est rendu.
async fn wait_and_wrap(
    ctx: &RuntimeContext<'_>,
    model: &AiLocalModel,
    mut child: Child,
    root_url: String,
    key: String,
) -> Result<Running, MiraError> {
    let timeout = Duration::from_secs(ctx.runtime.start_timeout_seconds);
    let health = &ctx.runtime.health_path;
    let low = is_low_memory(ctx, model);
    if let Err(error) = wait_ready(&mut child, &root_url, health, timeout, low).await {
        terminate(&mut child).await;
        return Err(error);
    }
    Ok(Running::new(model.id.clone(), child, root_url, key))
}
