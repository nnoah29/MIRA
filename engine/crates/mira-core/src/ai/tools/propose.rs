//! Exécution d'un outil de proposition : validation seule, aucune requête au moteur ni écriture.

use mira_vocabulary::{ChatMode, ToolCall};

use super::run_result::ToolRun;
use super::{compose_draft, edit_draft, specs_for};

/// Valide l'appel d'un outil de brouillon ; un outil absent du jeu du mode est indisponible.
pub fn propose(mode: ChatMode, call: &ToolCall) -> ToolRun {
    let offered = specs_for(mode).iter().any(|spec| spec.name == call.name);
    if !offered {
        return ToolRun::error("outil indisponible dans ce contexte".to_string());
    }
    match call.name.as_str() {
        compose_draft::NAME => compose_draft::run(&call.arguments),
        edit_draft::NAME => edit_draft::run(
            matches!(mode, ChatMode::Composer { selection: true }),
            &call.arguments,
        ),
        _ => ToolRun::error("outil inconnu".to_string()),
    }
}
