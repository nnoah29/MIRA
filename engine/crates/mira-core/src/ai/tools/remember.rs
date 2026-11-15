//! Outil `remember` : propose de retenir un souvenir ; rien n'est écrit avant le clic de l'utilisateur.

use mira_vocabulary::{MemoryKind, MemoryProposal, MemoryProposalAction, ToolSpec};
use serde::Deserialize;

use super::args::parse;
use super::run_result::ToolRun;
use crate::ai::memory::checked_text;

pub(super) const NAME: &str = "remember";

const SCHEMA: &str = r#"{"type":"object","properties":{
"text":{"type":"string","description":"Ce qu'il faut retenir, en une ou deux phrases autonomes (1000 caractères au plus)."},
"kind":{"type":"string","enum":["fact","preference","action"],"description":"fact : une information sur l'utilisateur ; preference : une manière de faire qu'il aime ; action : quelque chose qui a été fait."}
},"required":["text"]}"#;

/// Arguments de la proposition de souvenir.
#[derive(Debug, Deserialize)]
struct RememberArgs {
    text: String,
    kind: Option<MemoryKind>,
}

/// Fiche de l'outil annoncée au modèle.
pub(super) fn spec() -> ToolSpec {
    ToolSpec {
        name: NAME.to_string(),
        description: "Propose à l'utilisateur de retenir une information durable (un fait, une préférence, une action menée). Il confirme ou ignore : rien n'est retenu avant. Ne propose jamais de secret, de mot de passe ni de donnée bancaire.".to_string(),
        parameters_schema: SCHEMA.to_string(),
    }
}

/// Valide le texte avec les mêmes garde-fous que l'enregistrement ; le modèle apprend que l'utilisateur décide.
pub(super) fn run(arguments: &str) -> ToolRun {
    match build(arguments) {
        Ok(proposal) => ToolRun::memory_proposed(
            "La proposition est affichée à l'utilisateur, qui décide de retenir ou d'ignorer : ne dis pas que c'est retenu.".to_string(),
            proposal,
        ),
        Err(text) => ToolRun::error(text),
    }
}

/// Proposition tirée des arguments ; le genre vaut « fait » par défaut.
fn build(arguments: &str) -> Result<MemoryProposal, String> {
    let args: RememberArgs = parse(arguments)?;
    let text = checked_text(&args.text).map_err(|error| error.message)?;
    Ok(MemoryProposal {
        action: MemoryProposalAction::Remember,
        text,
        kind: args.kind.unwrap_or(MemoryKind::Fact),
        id: None,
    })
}
