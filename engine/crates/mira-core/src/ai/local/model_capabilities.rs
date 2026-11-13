//! Ce que sait faire un modèle local : les outils dépendent du modèle, pas du fournisseur.

use mira_vocabulary::AiProviderDescriptor;

/// Dit si les outils peuvent être annoncés : réglage du modèle pour un fournisseur local, de la fiche sinon.
pub fn tools_allowed(descriptor: &AiProviderDescriptor, model: &str) -> bool {
    match &descriptor.local_runtime {
        Some(runtime) => runtime
            .models
            .iter()
            .any(|m| m.id == model && m.supports_tools),
        None => descriptor.supports_tools,
    }
}
