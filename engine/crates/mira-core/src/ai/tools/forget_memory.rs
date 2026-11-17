//! Outil `forget_memory` : propose d'oublier un souvenir connu par son identifiant ; l'utilisateur confirme.

use mira_vocabulary::{MemoryKind, MemoryProposal, MemoryProposalAction, ToolSpec};
use serde::Deserialize;

use super::args::parse;
use super::run_result::ToolRun;

pub(super) const NAME: &str = "forget_memory";

const MAX_CHARS: usize = 200;

const SCHEMA: &str = r#"{"type":"object","properties":{
"id":{"type":"string","description":"Identifiant du souvenir, tel que rendu par recall_memory."},
"reason":{"type":"string","description":"Pourquoi l'oublier, en une phrase."}
},"required":["id"]}"#;

/// Arguments de la proposition d'oubli.
#[derive(Debug, Deserialize)]
struct ForgetArgs {
    id: String,
    reason: Option<String>,
}

/// Fiche de l'outil annoncée au modèle.
pub(super) fn spec() -> ToolSpec {
    ToolSpec {
        name: NAME.to_string(),
        description: "Propose à l'utilisateur d'oublier un souvenir que tu as retrouvé avec recall_memory. Il confirme ou ignore : rien n'est oublié avant.".to_string(),
        parameters_schema: SCHEMA.to_string(),
    }
}

/// Valide l'identifiant et la raison ; le modèle apprend que l'utilisateur décide.
pub(super) fn run(arguments: &str) -> ToolRun {
    match build(arguments) {
        Ok(proposal) => ToolRun::memory_proposed(
            "La proposition est affichée à l'utilisateur, qui décide d'oublier ou d'ignorer : ne dis pas que c'est oublié.".to_string(),
            proposal,
        ),
        Err(text) => ToolRun::error(text),
    }
}

/// Proposition d'oubli : l'application retrouve le texte du souvenir par son identifiant.
fn build(arguments: &str) -> Result<MemoryProposal, String> {
    let args: ForgetArgs = parse(arguments)?;
    let id = args.id.trim().to_string();
    let reason = args.reason.unwrap_or_default().trim().to_string();
    let bad = id.is_empty() || id.chars().count() > MAX_CHARS;
    if bad || reason.contains(['\n', '\r']) || reason.chars().count() > MAX_CHARS {
        return Err("identifiant vide ou raison trop longue ou sur plusieurs lignes".to_string());
    }
    Ok(MemoryProposal {
        action: MemoryProposalAction::Forget,
        text: reason,
        kind: MemoryKind::Fact,
        id: Some(id),
    })
}
