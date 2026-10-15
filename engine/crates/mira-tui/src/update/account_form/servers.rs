//! Serveurs saisis à la main, seulement pour le fournisseur de repli : lecture des champs et choix du chiffrement.

use mira_assembly::mira_vocabulary::{Security, ServerEndpoint, ServerSettings};

use crate::commands::Command;
use crate::state::AppState;
use crate::state::account_form::{AccountForm, FormField, Phase};
use crate::state::compose::TextBuffer;

/// Serveurs à envoyer : ceux des champs pour le repli, rien pour les autres (le moteur utilise ceux du fournisseur).
pub fn manual_servers(form: &AccountForm) -> Result<Option<ServerSettings>, String> {
    let manual = form
        .prepared
        .as_ref()
        .is_some_and(|prepared| prepared.manual_servers);
    if !manual {
        return Ok(None);
    }
    let imap = endpoint(&form.imap_host, &form.imap_port, form.imap_security, "IMAP")?;
    let smtp = endpoint(&form.smtp_host, &form.smtp_port, form.smtp_security, "SMTP")?;
    Ok(Some(ServerSettings { imap, smtp }))
}

/// Adresse d'un serveur : hôte sans espace et port de 1 à 65535.
fn endpoint(
    host: &TextBuffer,
    port: &TextBuffer,
    security: Security,
    name: &str,
) -> Result<ServerEndpoint, String> {
    let host = host.text().trim().to_string();
    if host.is_empty() || host.contains(char::is_whitespace) {
        return Err(format!("Serveur {name} : hôte invalide"));
    }
    match port.text().trim().parse::<u16>() {
        Ok(port) if port > 0 => Ok(ServerEndpoint {
            host,
            port,
            security,
        }),
        _ => Err(format!("Serveur {name} : port invalide (1 à 65535)")),
    }
}

/// Chiffrement suivant : TLS, STARTTLS, puis clair, puis de nouveau TLS.
fn next_security(security: Security) -> Security {
    match security {
        Security::Tls => Security::StartTls,
        Security::StartTls => Security::Plaintext,
        Security::Plaintext => Security::Tls,
    }
}

/// Change le chiffrement du serveur (IMAP ou SMTP) dont un champ a le focus ; sans effet ailleurs.
pub fn cycle_security(state: &mut AppState) -> Vec<Command> {
    let Some(form) = state.account_form.form_mut() else {
        return Vec::new();
    };
    if !matches!(form.phase, Phase::Typing | Phase::Ready) {
        return Vec::new();
    }
    match form.focus {
        FormField::ImapHost | FormField::ImapPort => {
            form.imap_security = next_security(form.imap_security);
        }
        FormField::SmtpHost | FormField::SmtpPort => {
            form.smtp_security = next_security(form.smtp_security);
        }
        FormField::Address | FormField::Name | FormField::Method | FormField::Password => {}
    }
    Vec::new()
}
