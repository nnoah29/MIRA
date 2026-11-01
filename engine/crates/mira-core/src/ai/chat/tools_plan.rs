//! Outils d'une génération : ceux du mode quand ils sont permis, jamais ceux que le client propose.

use mira_vocabulary::{ChatMode, ChatRequest};

use super::framing::{ACTION_NOTICE, COMPOSE_NOTICE, TOOLS_NOTICE};
use super::prepare::ChatPlan;
use crate::ai::tools::specs_for;

/// Annonce les outils du mode et les consignes qui les encadrent, ou aucun si `enabled` est faux.
pub(super) fn with_tools(mut request: ChatRequest, enabled: bool, mode: ChatMode) -> ChatRequest {
    request.tools = if enabled { specs_for(mode) } else { Vec::new() };
    if enabled {
        let actions = matches!(mode, ChatMode::Assistant)
            .then(|| format!("\n\n{ACTION_NOTICE}"))
            .unwrap_or_default();
        let notice = format!("{TOOLS_NOTICE}\n\n{COMPOSE_NOTICE}{actions}");
        request.system = Some(match request.system.take() {
            Some(system) => format!("{system}\n\n{notice}"),
            None => notice,
        });
    }
    request
}

/// Dit si l'outil fait partie de ceux annoncés au modèle pour la suite de la génération.
pub(super) fn is_announced(plan: &ChatPlan, name: &str) -> bool {
    plan.request.tools.iter().any(|tool| tool.name == name)
}
