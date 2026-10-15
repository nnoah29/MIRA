//! Contenu d'un formulaire de Paramètres : titre, avertissement, champs et touches.

use ratatui::text::{Line, Span};

use crate::state::settings::{Form, FormKind};
use crate::theme;
use crate::view::safe_text::display_safe;

/// Titre de la fenêtre selon ce que le formulaire modifie.
pub fn form_title(kind: &FormKind) -> &'static str {
    match kind {
        FormKind::EditAccount(_) => "Modifier le compte",
        FormKind::DeleteAccount { .. } => "Supprimer le compte",
        FormKind::AddIdentity(_) => "Ajouter une identité",
        FormKind::EditIdentity(_) => "Modifier l'identité",
        FormKind::Export(_) => "Exporter",
    }
}

/// Phrase qui dit l'enjeu avant les champs.
fn intro(kind: &FormKind) -> Option<String> {
    match kind {
        FormKind::DeleteAccount { address, .. } => Some(format!(
            "Supprime {} et ses données locales et secrets (le serveur n'est pas touché). Tapez l'adresse du compte pour confirmer.",
            display_safe(address)
        )),
        FormKind::Export(_) => Some("Le fichier ne contient aucun secret.".to_string()),
        FormKind::EditAccount(_) | FormKind::AddIdentity(_) | FormKind::EditIdentity(_) => None,
    }
}

/// Une ligne de champ ; le curseur marque le champ qui a le focus.
fn field_line(label: &str, value: &str, focused: bool) -> Line<'static> {
    let cursor = if focused { "▏" } else { "" };
    let text = format!("{label} : {}{cursor}", display_safe(value));
    if focused {
        Line::styled(text, theme::selected())
    } else {
        Line::from(text)
    }
}

/// Toutes les lignes du formulaire, touches comprises.
pub fn form_lines(form: &Form) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    if let Some(text) = intro(&form.kind) {
        lines.push(Line::from(text));
        lines.push(Line::from(""));
    }
    for (index, field) in form.fields.iter().enumerate() {
        lines.push(field_line(field.label, &field.value, index == form.focus));
    }
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Entrée : valider · Tab : champ suivant · Échap : annuler",
        theme::dim(),
    )));
    lines
}
