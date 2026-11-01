//! Outil `edit_draft` : propose une modification du brouillon ouvert, ou d'un passage sélectionné.

use mira_vocabulary::{DraftProposal, DraftProposalKind, ToolSpec};
use serde::Deserialize;

use super::args::parse;
use super::proposal::recipients;
use super::proposal_text::{accepted, body, subject};
use super::run_result::ToolRun;

pub(super) const NAME: &str = "edit_draft";

const SCHEMA: &str = r#"{"type":"object","properties":{
"subject":{"type":"string","description":"Nouvel objet, sur une ligne."},
"to":{"type":"array","items":{"type":"string"},"description":"Nouveaux destinataires."},
"cc":{"type":"array","items":{"type":"string"},"description":"Nouveaux destinataires en copie."},
"body":{"type":"string","description":"Nouveau corps complet, sans formule finale ni signature."}
}}"#;

const SELECTION_SCHEMA: &str = r#"{"type":"object","properties":{
"selection":{"type":"string","description":"Texte qui remplace le passage sélectionné, rien d'autre."}
},"required":["selection"]}"#;

/// Arguments de la modification du brouillon.
#[derive(Debug, Deserialize)]
struct EditArgs {
    subject: Option<String>,
    to: Option<Vec<String>>,
    cc: Option<Vec<String>>,
    body: Option<String>,
    selection: Option<String>,
}

/// Fiche de l'outil annoncée au modèle ; avec une sélection, seul le passage est modifiable.
pub(super) fn spec(selection: bool) -> ToolSpec {
    let (description, schema) = match selection {
        true => (
            "Remplace le passage sélectionné du brouillon par le texte donné.",
            SELECTION_SCHEMA,
        ),
        false => (
            "Modifie le brouillon ouvert (objet, destinataires, corps) ; seuls les champs donnés changent, le corps est remplacé en entier.",
            SCHEMA,
        ),
    };
    ToolSpec {
        name: NAME.to_string(),
        description: description.to_string(),
        parameters_schema: schema.to_string(),
    }
}

/// Valide la modification et la rend comme proposition, ou explique l'échec au modèle.
pub(super) fn run(selection: bool, arguments: &str) -> ToolRun {
    match build(selection, arguments) {
        Ok(proposal) => accepted(proposal),
        Err(text) => ToolRun::error(text),
    }
}

/// Proposition « édition » tirée des arguments, selon que le brouillon a une sélection ou non.
fn build(selection: bool, arguments: &str) -> Result<DraftProposal, String> {
    let args: EditArgs = parse(arguments)?;
    match selection {
        true => selected(args),
        false => whole(args),
    }
}

/// Remplacement du passage sélectionné : aucun autre champ n'est accepté.
fn selected(args: EditArgs) -> Result<DraftProposal, String> {
    if args.subject.is_some() || args.to.is_some() || args.cc.is_some() || args.body.is_some() {
        return Err("avec une sélection, seul le champ « selection » est accepté".to_string());
    }
    let text = body(args.selection)?.ok_or("le champ « selection » est obligatoire")?;
    let mut proposal = DraftProposal::empty(DraftProposalKind::Edit);
    proposal.selection = Some(text);
    Ok(proposal)
}

/// Modification de champs du brouillon : au moins un champ, jamais de sélection.
fn whole(args: EditArgs) -> Result<DraftProposal, String> {
    if args.selection.is_some() {
        return Err("aucun passage n'est sélectionné : « selection » est refusé".to_string());
    }
    let mut proposal = DraftProposal::empty(DraftProposalKind::Edit);
    proposal.to = recipients(args.to, "to")?;
    proposal.cc = recipients(args.cc, "cc")?;
    proposal.subject = subject(args.subject)?;
    proposal.body = body(args.body)?;
    let empty = proposal.to.is_none()
        && proposal.cc.is_none()
        && proposal.subject.is_none()
        && proposal.body.is_none();
    match empty {
        true => Err("aucun champ à modifier".to_string()),
        false => Ok(proposal),
    }
}
