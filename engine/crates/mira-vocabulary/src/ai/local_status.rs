//! État du modèle local tel que le client le lit : modèles, installation en cours, mémoire.

use serde::{Deserialize, Serialize};

use super::local_runtime::LocalTier;
use crate::ids::kinds::OperationId;

/// Un modèle proposé et son état sur cette machine.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalModelStatus {
    pub id: String,
    pub display_name: String,
    pub tier: LocalTier,
    pub size: u64,
    pub license: String,
    pub supports_tools: bool,
    pub ram_recommended_mib: u32,
    pub installed: bool,
    pub memory_warning: bool,
}

/// Installation en cours : le modèle visé et l'opération qui la porte.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalInstallInfo {
    pub model_id: String,
    pub operation: OperationId,
}

/// État complet du modèle local ; `supported` est faux hors des plateformes décrites.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalRuntimeStatus {
    pub supported: bool,
    pub models: Vec<LocalModelStatus>,
    pub server_installed: bool,
    pub running_model: Option<String>,
    pub memory_available_mib: Option<u64>,
    pub installing: Option<LocalInstallInfo>,
}
