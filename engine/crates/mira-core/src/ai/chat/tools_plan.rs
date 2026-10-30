//! Outils d'une génération : ceux du catalogue quand ils sont permis, jamais ceux que le client propose.

use mira_vocabulary::ChatRequest;

use super::framing::TOOLS_NOTICE;
use crate::ai::tools::specs;

/// Annonce les outils du catalogue et la consigne qui les encadre, ou aucun si `enabled` est faux.
pub(super) fn with_tools(mut request: ChatRequest, enabled: bool) -> ChatRequest {
    request.tools = if enabled { specs() } else { Vec::new() };
    if enabled {
        request.system = Some(match request.system.take() {
            Some(system) => format!("{system}\n\n{TOOLS_NOTICE}"),
            None => TOOLS_NOTICE.to_string(),
        });
    }
    request
}
