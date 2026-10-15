//! Textes du formulaire d'ajout de compte autour des champs : ce que propose le moteur, état de la requête, erreur, confirmation.

use mira_assembly::mira_vocabulary::{AuthMethodKind, ServerEndpoint};
use ratatui::text::Line;

use super::account_form_rows::{method_label, security_label};
use crate::state::account_form::{AccountForm, Phase, Prepared};
use crate::theme;
use crate::view::safe_text::display_safe;

/// Un serveur : hôte, port et chiffrement.
fn endpoint_label(endpoint: &ServerEndpoint) -> String {
    let host = display_safe(&endpoint.host);
    format!(
        "{host}:{} ({})",
        endpoint.port,
        security_label(endpoint.security)
    )
}

/// Lignes d'en-tête : invite avant la préparation, puis fournisseur, serveurs et méthodes proposés.
pub fn header_lines(form: &AccountForm) -> Vec<Line<'static>> {
    let Some(prepared) = &form.prepared else {
        return vec![Line::styled(
            "Saisissez l'adresse puis Entrée : MIRA propose les serveurs.",
            theme::dim(),
        )];
    };
    let name = display_safe(&prepared.provider_name);
    let mut lines = vec![Line::from(format!("Fournisseur : {name}"))];
    lines.push(Line::from(servers_text(prepared)));
    lines.extend(methods_lines(prepared));
    lines
}

/// Serveurs proposés, ou invite à les saisir pour le fournisseur de repli.
fn servers_text(prepared: &Prepared) -> String {
    if prepared.manual_servers {
        return "Serveurs à confirmer ou à saisir (Ctrl+t : chiffrement)".to_string();
    }
    let (imap, smtp) = (&prepared.servers.imap, &prepared.servers.smtp);
    format!(
        "Serveurs : IMAP {} · SMTP {}",
        endpoint_label(imap),
        endpoint_label(smtp)
    )
}

/// Méthodes saisissables et, s'il y a lieu, la mention d'OAuth indisponible.
fn methods_lines(prepared: &Prepared) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    if !prepared.methods.is_empty() {
        let names: Vec<&str> = prepared
            .methods
            .iter()
            .map(|kind| method_label(*kind))
            .collect();
        lines.push(Line::from(format!(
            "Authentification : {}",
            names.join(", ")
        )));
    }
    if prepared.unavailable.contains(&AuthMethodKind::OAuth2) {
        lines.push(Line::styled(
            "OAuth : non disponible dans cette version",
            theme::status(),
        ));
    } else if prepared.methods.is_empty() {
        lines.push(Line::styled(
            "Aucune méthode d'authentification disponible",
            theme::status(),
        ));
    }
    lines
}

/// Lignes de pied : confirmation de fermeture, requête en cours ou erreur.
pub fn footer_lines(form: &AccountForm) -> Vec<Line<'static>> {
    if form.confirm_close {
        return vec![Line::styled(
            "Abandonner la saisie ? Entrée/y : oui · Échap/n : continuer",
            theme::status(),
        )];
    }
    let busy = match form.phase {
        Phase::Preparing(_) => Some("Recherche des serveurs…"),
        Phase::Adding(_) => Some("Test de la connexion et ajout du compte…"),
        Phase::Typing | Phase::Ready => None,
    };
    let error = form.error.as_deref().map(display_safe);
    busy.map(str::to_string)
        .into_iter()
        .chain(error)
        .map(|text| Line::styled(text, theme::status()))
        .collect()
}
