//! Outil `compose_draft` : propose un nouveau message ; le moteur valide, l'application ouvre le brouillon.

use mira_vocabulary::{DraftProposal, DraftProposalKind, ToolSpec};
use serde::Deserialize;

use super::args::parse;
use super::proposal::recipients;
use super::proposal_text::{accepted, body, subject};
use super::run_result::ToolRun;

pub(super) const NAME: &str = "compose_draft";

const SCHEMA: &str = r#"{"type":"object","properties":{
"to":{"type":"array","items":{"type":"string"},"description":"Destinataires : adresse ou « Nom <adresse> »."},
"cc":{"type":"array","items":{"type":"string"},"description":"Destinataires en copie."},
"subject":{"type":"string","description":"Objet, sur une ligne."},
"body":{"type":"string","description":"Corps du message, sans formule finale ni signature."},
"reply_to":{"type":"string","description":"Identifiant du message auquel on répond, s'il y en a un."},
"identity":{"type":"string","description":"Adresse d'envoi à utiliser, parmi les adresses de list_identities."}
},"required":["body"]}"#;

/// Arguments de la composition d'un message.
#[derive(Debug, Deserialize)]
struct ComposeArgs {
    to: Option<Vec<String>>,
    cc: Option<Vec<String>>,
    subject: Option<String>,
    body: Option<String>,
    reply_to: Option<String>,
    identity: Option<String>,
}

/// Fiche de l'outil annoncée au modèle.
pub(super) fn spec() -> ToolSpec {
    ToolSpec {
        name: NAME.to_string(),
        description: "Prépare un nouveau message ou une réponse sous forme de brouillon que l'utilisateur relira et enverra lui-même. À utiliser dès qu'on te demande d'écrire un mail, au lieu de le recopier dans ta réponse.".to_string(),
        parameters_schema: SCHEMA.to_string(),
    }
}

/// Valide la composition et la rend comme proposition, ou explique l'échec au modèle.
pub(super) fn run(arguments: &str) -> ToolRun {
    match build(arguments) {
        Ok(proposal) => accepted(proposal),
        Err(text) => ToolRun::error(text),
    }
}

/// Proposition « nouveau message » tirée des arguments.
fn build(arguments: &str) -> Result<DraftProposal, String> {
    let args: ComposeArgs = parse(arguments)?;
    let text = body(args.body)?.ok_or("le corps du message est obligatoire")?;
    let mut proposal = DraftProposal::empty(DraftProposalKind::New);
    proposal.to = recipients(args.to, "to")?;
    proposal.cc = recipients(args.cc, "cc")?;
    proposal.subject = subject(args.subject)?;
    proposal.body = Some(text);
    proposal.reply_to = args.reply_to.filter(|id| !id.trim().is_empty());
    proposal.identity = args.identity.filter(|id| !id.trim().is_empty());
    Ok(proposal)
}
