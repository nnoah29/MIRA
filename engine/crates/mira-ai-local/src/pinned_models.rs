//! Les trois modèles proposés, épinglés par taille et somme SHA-256.

use mira_vocabulary::{AiLocalModel, LocalTier};

const HOSTING: &str = "https://huggingface.co";

/// Les trois paliers, du plus léger au plus capable.
pub(crate) fn models() -> Vec<AiLocalModel> {
    vec![mini(), light(), advanced()]
}

/// Palier Mini : Granite 4.0 H 1B.
fn mini() -> AiLocalModel {
    model(Spec {
        id: "granite-4.0-h-1b",
        name: "Granite 4.0 H 1B",
        tier: LocalTier::Mini,
        repo: "ibm-granite/granite-4.0-h-1b-GGUF",
        file: "granite-4.0-h-1b-Q4_K_M.gguf",
        size: 901_162_208,
        sha256: "da3d737121a96f3c9a316685212376257a7f167b74380855666dd488d6af3bcb",
        ram: 1300,
        ctx: 4096,
        tools: false,
    })
}

/// Palier Léger : Ministral 3 3B.
fn light() -> AiLocalModel {
    model(Spec {
        id: "ministral-3-3b",
        name: "Ministral 3 3B",
        tier: LocalTier::Light,
        repo: "mistralai/Ministral-3-3B-Instruct-2512-GGUF",
        file: "Ministral-3-3B-Instruct-2512-Q4_K_M.gguf",
        size: 2_147_023_008,
        sha256: "9ed150d4367e68df0ac8e1540f6ddc65b42d0ee26378329d1ecbca60f93fc5f8",
        ram: 3500,
        ctx: 8192,
        tools: false,
    })
}

/// Palier Avancé : Granite 4.2 8B, annoncé avec outils.
fn advanced() -> AiLocalModel {
    model(Spec {
        id: "granite-4.2-8b",
        name: "Granite 4.2 8B",
        tier: LocalTier::Advanced,
        repo: "ibm-granite/granite-4.2-8b-GGUF",
        file: "granite-4.2-8b-Q4_K_M.gguf",
        size: 5_347_917_952,
        sha256: "16a9369d0805f80b7377d25d87f937a90c05dc04ad79173a52001e42c9aab311",
        ram: 7000,
        ctx: 8192,
        tools: true,
    })
}

/// Valeurs propres à un modèle ; le reste est commun aux trois.
struct Spec {
    id: &'static str,
    name: &'static str,
    tier: LocalTier,
    repo: &'static str,
    file: &'static str,
    size: u64,
    sha256: &'static str,
    ram: u32,
    ctx: u32,
    tools: bool,
}

/// Complète une fiche de modèle : adresse d'hébergement, licence, sans raisonnement.
fn model(spec: Spec) -> AiLocalModel {
    AiLocalModel {
        id: spec.id.to_string(),
        display_name: spec.name.to_string(),
        tier: spec.tier,
        url: format!("{HOSTING}/{}/resolve/main/{}", spec.repo, spec.file),
        file_name: spec.file.to_string(),
        sha256: spec.sha256.to_string(),
        size: spec.size,
        license: "Apache-2.0".to_string(),
        ram_recommended_mib: spec.ram,
        ctx_size: spec.ctx,
        supports_tools: spec.tools,
        reasoning: false,
    }
}
