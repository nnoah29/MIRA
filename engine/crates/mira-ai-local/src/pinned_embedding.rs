//! Modèle d'embeddings de la mémoire de l'IA, épinglé par taille et somme SHA-256, et ses arguments de serveur.

use mira_vocabulary::{AiLocalModel, LocalTier};

const FILE: &str = "granite-embedding-278m-multilingual-Q6_K.gguf";
const REPO: &str = "bartowski/granite-embedding-278m-multilingual-GGUF";

/// Granite Embedding 278M multilingue en Q6_K : vecteurs de 768 valeurs, contexte de 512 jetons, licence Apache-2.0.
pub fn embedding_model() -> AiLocalModel {
    AiLocalModel {
        id: "granite-embedding-278m-multilingual".to_string(),
        display_name: "Granite Embedding 278M multilingue".to_string(),
        tier: LocalTier::Mini,
        url: format!("https://huggingface.co/{REPO}/resolve/main/{FILE}"),
        file_name: FILE.to_string(),
        sha256: "a1ef1eec2c50415808f3c3f3613d392a2bf084735731442c30f471ed6c66a38b".to_string(),
        size: 236_067_488,
        license: "Apache-2.0".to_string(),
        ram_recommended_mib: 700,
        ctx_size: 512,
        supports_tools: false,
        reasoning: false,
    }
}

/// Arguments propres au mode embeddings : pooling CLS explicite et lot physique de la taille du contexte.
pub fn embedding_server_args() -> Vec<String> {
    [
        "--embedding",
        "--pooling",
        "cls",
        "-c",
        "512",
        "-b",
        "512",
        "-ub",
        "512",
    ]
    .iter()
    .map(|a| (*a).to_string())
    .collect()
}
