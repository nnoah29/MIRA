//! Serveur et modèles épinglés par taille et somme SHA-256 : les changer demande une nouvelle version de MIRA.

use mira_vocabulary::AiLocalRuntime;

use crate::pinned_models::models;
use crate::pinned_server::server_builds;

/// Données d'installation et de lancement du serveur local avec les trois modèles proposés.
pub fn runtime() -> AiLocalRuntime {
    AiLocalRuntime {
        server: server_builds(),
        server_args: server_args(),
        reasoning_off_args: strings(&["--reasoning", "off"]),
        health_path: "/health".to_string(),
        idle_stop_seconds: 300,
        start_timeout_seconds: 120,
        models: models(),
    }
}

/// Convertit une liste de littéraux en chaînes possédées.
fn strings(items: &[&str]) -> Vec<String> {
    items.iter().map(|item| (*item).to_string()).collect()
}

/// Gabarit de la ligne de commande : jetons `{...}` remplacés par le moteur au lancement.
fn server_args() -> Vec<String> {
    strings(&[
        "--model",
        "{model}",
        "--host",
        "{host}",
        "--port",
        "{port}",
        "--api-key-file",
        "{api_key_file}",
        "--ctx-size",
        "{ctx}",
        "--threads",
        "{threads}",
        "--alias",
        "{alias}",
        "--jinja",
        "--no-webui",
        "--parallel",
        "1",
    ])
}
