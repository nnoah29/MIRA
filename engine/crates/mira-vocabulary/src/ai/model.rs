//! Modèles d'IA proposés par un service, tels que relevés à une date.

use serde::{Deserialize, Serialize};

use crate::time::Timestamp;

/// Un modèle annoncé par le service ; la date de création est celle qu'il déclare.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AiModel {
    pub id: String,
    pub created_at: Option<i64>,
}

/// Liste des modèles relevée à un instant, gardée pour ne pas interroger le service à chaque fois.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AiModelList {
    pub models: Vec<AiModel>,
    pub fetched_at: Timestamp,
}
