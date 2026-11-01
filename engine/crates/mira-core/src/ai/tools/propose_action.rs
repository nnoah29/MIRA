//! Outil `propose_mailbox_action` : propose d'archiver, jeter, marquer ou suivre des messages ; l'utilisateur confirme, l'application agit.

use mira_vocabulary::{ActionProposal, MailActionKind, ToolSpec};
use serde::Deserialize;

use super::args::parse;
use super::run_result::ToolRun;

pub(super) const NAME: &str = "propose_mailbox_action";

const MAX_MESSAGES: usize = 100;
const MAX_ID_CHARS: usize = 200;
const MAX_REASON_CHARS: usize = 200;

const SCHEMA: &str = r#"{"type":"object","properties":{
"action":{"type":"string","enum":["archive","trash","mark_read","mark_unread","star","unstar"],"description":"Ce qu'on propose de faire aux messages."},
"messages":{"type":"array","items":{"type":"string"},"description":"Identifiants des messages (champ id donné par les outils de lecture), 100 au plus."},
"reason":{"type":"string","description":"Pourquoi, en une phrase."}
},"required":["action","messages"]}"#;

/// Arguments de la proposition d'action.
#[derive(Debug, Deserialize)]
struct ActionArgs {
    action: MailActionKind,
    messages: Vec<String>,
    reason: Option<String>,
}

/// Fiche de l'outil annoncée au modèle.
pub(super) fn spec() -> ToolSpec {
    ToolSpec {
        name: NAME.to_string(),
        description: "Propose d'archiver, de mettre à la corbeille, de marquer comme lu ou non lu, ou de mettre en favori des messages que tu as lus avec les outils. Rien ne change avant la confirmation de l'utilisateur.".to_string(),
        parameters_schema: SCHEMA.to_string(),
    }
}

/// Valide la proposition ; le modèle apprend que l'utilisateur confirmera ou ignorera.
pub(super) fn run(arguments: &str) -> ToolRun {
    match build(arguments) {
        Ok(action) => ToolRun::action_proposed(
            "Action proposée à l'utilisateur : il la confirmera ou l'ignorera, et rien n'est modifié avant sa confirmation ; tu ne peux rien faire directement.".to_string(),
            action,
        ),
        Err(text) => ToolRun::error(text),
    }
}

/// Proposition tirée des arguments : identifiants distincts et bornés, raison sur une ligne.
fn build(arguments: &str) -> Result<ActionProposal, String> {
    let args: ActionArgs = parse(arguments)?;
    let messages = distinct_ids(args.messages)?;
    let reason = args
        .reason
        .map(|text| text.trim().to_string())
        .filter(|text| !text.is_empty());
    if let Some(text) = reason.as_deref() {
        if text.contains(['\n', '\r']) || text.chars().count() > MAX_REASON_CHARS {
            return Err(format!(
                "la raison doit tenir sur une ligne de {MAX_REASON_CHARS} caractères au plus"
            ));
        }
    }
    Ok(ActionProposal {
        action: args.action,
        messages,
        reason,
    })
}

/// Identifiants sans doublon ni blanc, de 1 à 100, chacun de 200 caractères au plus.
fn distinct_ids(list: Vec<String>) -> Result<Vec<String>, String> {
    let mut kept: Vec<String> = Vec::new();
    for id in list {
        let id = id.trim().to_string();
        if id.is_empty() || id.chars().count() > MAX_ID_CHARS {
            return Err("identifiant de message vide ou trop long".to_string());
        }
        if !kept.contains(&id) {
            kept.push(id);
        }
    }
    match kept.len() {
        0 => Err("aucun message indiqué".to_string()),
        n if n > MAX_MESSAGES => Err(format!("{MAX_MESSAGES} messages au plus par proposition")),
        _ => Ok(kept),
    }
}
