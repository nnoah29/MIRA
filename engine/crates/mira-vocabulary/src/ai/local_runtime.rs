//! Fiche d'un fournisseur dont le modèle tourne sur la machine : serveur et modèles épinglés.

use serde::{Deserialize, Serialize};

/// Palier d'un modèle local : plus il monte, plus il est capable et lourd.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LocalTier {
    Mini,
    Light,
    Advanced,
}

/// Archive du serveur pour une plateforme, épinglée par taille et somme SHA-256.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AiLocalServerBuild {
    pub platform: String,
    pub url: String,
    pub sha256: String,
    pub size: u64,
    pub executable: String,
}

/// Modèle téléchargeable, épinglé par taille et somme SHA-256.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AiLocalModel {
    pub id: String,
    pub display_name: String,
    pub tier: LocalTier,
    pub url: String,
    pub file_name: String,
    pub sha256: String,
    pub size: u64,
    pub license: String,
    pub ram_recommended_mib: u32,
    pub ctx_size: u32,
    pub supports_tools: bool,
    pub reasoning: bool,
}

/// Tout ce qu'il faut pour installer et lancer le serveur local, sans que le cœur le nomme.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AiLocalRuntime {
    pub server: Vec<AiLocalServerBuild>,
    pub server_args: Vec<String>,
    pub reasoning_off_args: Vec<String>,
    pub health_path: String,
    pub idle_stop_seconds: u64,
    pub start_timeout_seconds: u64,
    pub models: Vec<AiLocalModel>,
}
