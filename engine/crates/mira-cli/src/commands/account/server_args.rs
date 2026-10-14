//! Serveurs saisis à la main (compte « Autre » ou serveurs imposés).

use clap::{Args, ValueEnum};
use mira_assembly::mira_vocabulary::{MiraError, Security, ServerEndpoint, ServerSettings};

use crate::argument_error::invalid_argument;

/// Chiffrement choisi pour un serveur.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum SecurityChoice {
    Tls,
    Starttls,
    Plaintext,
}

/// Options des serveurs IMAP et SMTP ; absentes, les serveurs du provider s'appliquent.
#[derive(Debug, Args)]
pub struct ServerArgs {
    /// Hôte IMAP (avec `--smtp-host`, désactive la détection).
    #[arg(long)]
    pub imap_host: Option<String>,
    #[arg(long, default_value_t = 993)]
    pub imap_port: u16,
    #[arg(long, value_enum, default_value_t = SecurityChoice::Tls)]
    pub imap_security: SecurityChoice,
    /// Hôte SMTP.
    #[arg(long)]
    pub smtp_host: Option<String>,
    #[arg(long, default_value_t = 465)]
    pub smtp_port: u16,
    #[arg(long, value_enum, default_value_t = SecurityChoice::Tls)]
    pub smtp_security: SecurityChoice,
}

/// Convertit le choix de chiffrement en type du vocabulaire.
fn security_of(choice: SecurityChoice) -> Security {
    match choice {
        SecurityChoice::Tls => Security::Tls,
        SecurityChoice::Starttls => Security::StartTls,
        SecurityChoice::Plaintext => Security::Plaintext,
    }
}

/// Fabrique une adresse de serveur.
fn endpoint(host: &str, port: u16, choice: SecurityChoice) -> ServerEndpoint {
    ServerEndpoint {
        host: host.to_string(),
        port,
        security: security_of(choice),
    }
}

impl ServerArgs {
    /// Rend les serveurs saisis, ou `None` si aucun hôte n'est donné ; un seul hôte est refusé.
    pub fn settings(&self) -> Result<Option<ServerSettings>, MiraError> {
        match (&self.imap_host, &self.smtp_host) {
            (None, None) => Ok(None),
            (Some(imap), Some(smtp)) => Ok(Some(ServerSettings {
                imap: endpoint(imap, self.imap_port, self.imap_security),
                smtp: endpoint(smtp, self.smtp_port, self.smtp_security),
            })),
            _ => Err(invalid_argument("--imap-host et --smtp-host vont ensemble")),
        }
    }
}
