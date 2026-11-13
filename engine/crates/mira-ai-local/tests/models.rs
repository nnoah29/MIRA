#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Les trois modèles proposés sont épinglés exactement comme dans la spec.

use mira_ai_local::LocalAiProvider;
use mira_vocabulary::{AiLocalModel, AiProvider, LocalTier};

/// Cherche un modèle de la fiche par son identifiant
fn model(id: &str) -> AiLocalModel {
    let runtime = LocalAiProvider.descriptor().local_runtime.unwrap();
    runtime.models.into_iter().find(|m| m.id == id).unwrap()
}

/// Le palier Mini est Granite 4.0 H 1B, sans outils, contexte 4096
#[test]
fn mini_model_matches_spec() {
    let m = model("granite-4.0-h-1b");
    assert_eq!(m.tier, LocalTier::Mini);
    assert_eq!(
        m.url,
        "https://huggingface.co/ibm-granite/granite-4.0-h-1b-GGUF/resolve/main/granite-4.0-h-1b-Q4_K_M.gguf"
    );
    assert_eq!(m.file_name, "granite-4.0-h-1b-Q4_K_M.gguf");
    assert_eq!(m.size, 901_162_208);
    assert_eq!(
        m.sha256,
        "da3d737121a96f3c9a316685212376257a7f167b74380855666dd488d6af3bcb"
    );
    assert_eq!((m.ctx_size, m.ram_recommended_mib), (4096, 1300));
    assert!(!m.supports_tools && !m.reasoning);
    assert_eq!(m.license, "Apache-2.0");
}

/// Le palier Léger est Ministral 3 3B, sans outils, contexte 8192
#[test]
fn light_model_matches_spec() {
    let m = model("ministral-3-3b");
    assert_eq!(m.tier, LocalTier::Light);
    assert_eq!(
        m.url,
        "https://huggingface.co/mistralai/Ministral-3-3B-Instruct-2512-GGUF/resolve/main/Ministral-3-3B-Instruct-2512-Q4_K_M.gguf"
    );
    assert_eq!(m.size, 2_147_023_008);
    assert_eq!(
        m.sha256,
        "9ed150d4367e68df0ac8e1540f6ddc65b42d0ee26378329d1ecbca60f93fc5f8"
    );
    assert_eq!((m.ctx_size, m.ram_recommended_mib), (8192, 3500));
    assert!(!m.supports_tools && !m.reasoning);
}

/// Le palier Avancé est Granite 4.2 8B, avec outils, contexte 8192
#[test]
fn advanced_model_matches_spec() {
    let m = model("granite-4.2-8b");
    assert_eq!(m.tier, LocalTier::Advanced);
    assert_eq!(
        m.url,
        "https://huggingface.co/ibm-granite/granite-4.2-8b-GGUF/resolve/main/granite-4.2-8b-Q4_K_M.gguf"
    );
    assert_eq!(m.size, 5_347_917_952);
    assert_eq!(
        m.sha256,
        "16a9369d0805f80b7377d25d87f937a90c05dc04ad79173a52001e42c9aab311"
    );
    assert_eq!((m.ctx_size, m.ram_recommended_mib), (8192, 7000));
    assert!(m.supports_tools && !m.reasoning);
}
