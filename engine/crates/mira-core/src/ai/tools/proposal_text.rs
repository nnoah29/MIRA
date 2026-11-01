//! Textes d'une proposition de brouillon : objet, corps, trous entre crochets, et résultat renvoyé au modèle.

use mira_vocabulary::DraftProposal;

use super::run_result::ToolRun;

pub(super) const MAX_BODY_CHARS: usize = 50_000;
pub(super) const MAX_SUBJECT_CHARS: usize = 300;
const MAX_HOLE_CHARS: usize = 40;
const MAX_HOLES: usize = 3;

/// Objet sur une ligne et de 300 caractères au plus ; absent, il reste absent.
pub(super) fn subject(text: Option<String>) -> Result<Option<String>, String> {
    let Some(text) = text else { return Ok(None) };
    let text = text.trim().to_string();
    if text.contains(['\n', '\r']) {
        return Err("l'objet doit tenir sur une seule ligne".to_string());
    }
    match text.chars().count() > MAX_SUBJECT_CHARS {
        true => Err(format!("l'objet dépasse {MAX_SUBJECT_CHARS} caractères")),
        false => Ok(Some(text)),
    }
}

/// Texte non blanc de 50 000 caractères au plus ; absent, il reste absent.
pub(super) fn body(text: Option<String>) -> Result<Option<String>, String> {
    let Some(text) = text else { return Ok(None) };
    if text.trim().is_empty() {
        return Err("le texte ne peut pas être vide".to_string());
    }
    match text.chars().count() > MAX_BODY_CHARS {
        true => Err(format!("le texte dépasse {MAX_BODY_CHARS} caractères")),
        false => Ok(Some(text)),
    }
}

/// Passages entre crochets d'une ligne (trois au plus), signes probables de trous à remplir.
pub(super) fn brackets(text: &str) -> Vec<String> {
    let mut found = Vec::new();
    let mut rest = text;
    while let Some(start) = rest.find('[') {
        let tail = &rest[start..];
        let Some(end) = tail.find(']') else { break };
        let hole = &tail[..=end];
        if hole.chars().count() <= MAX_HOLE_CHARS && !hole.contains('\n') {
            found.push(hole.to_string());
        }
        rest = &tail[end + 1..];
    }
    found.truncate(MAX_HOLES);
    found
}

/// Résultat accepté : phrase fixe, plus l'avertissement des trous entre crochets s'il y en a.
pub(super) fn accepted(proposal: DraftProposal) -> ToolRun {
    let mut holes = proposal.body.as_deref().map(brackets).unwrap_or_default();
    holes.extend(
        proposal
            .selection
            .as_deref()
            .map(brackets)
            .unwrap_or_default(),
    );
    holes.truncate(MAX_HOLES);
    let mut text = "Proposition de brouillon transmise à l'utilisateur, qui la relira et l'enverra lui-même ; tu ne peux pas l'envoyer.".to_string();
    if !holes.is_empty() {
        text.push_str(&format!("\nAttention : le texte contient des passages entre crochets ({}). Si ce sont des trous à remplir, complète-les avec les outils ou demande l'information à l'utilisateur, puis propose de nouveau.", holes.join(", ")));
    }
    ToolRun::proposed(text, proposal)
}
