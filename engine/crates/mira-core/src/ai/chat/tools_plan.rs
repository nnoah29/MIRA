//! Outils d'une génération : ceux du mode quand ils sont permis, jamais ceux que le client propose.

use mira_vocabulary::{ChatMode, ChatRequest};

use super::framing::{COMPOSE_NOTICE, TOOLS_NOTICE};
use crate::ai::tools::specs_for;

/// Annonce les outils du mode et les consignes qui les encadrent, ou aucun si `enabled` est faux.
pub(super) fn with_tools(mut request: ChatRequest, enabled: bool, mode: ChatMode) -> ChatRequest {
    request.tools = if enabled { specs_for(mode) } else { Vec::new() };
    if enabled {
        let notice = format!("{TOOLS_NOTICE}\n\n{COMPOSE_NOTICE}");
        request.system = Some(match request.system.take() {
            Some(system) => format!("{system}\n\n{notice}"),
            None => notice,
        });
    }
    request
}
