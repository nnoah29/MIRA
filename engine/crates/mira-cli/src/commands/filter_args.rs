//! Filtres de liste et de recherche communs, tous facultatifs et combinés par « et ».

use clap::Args;
use mira_assembly::mira_vocabulary::MiraError;
use mira_assembly::reading::MessageFilters;

use super::instant_arg::parse_instant;

/// Filtres sur les messages listés.
#[derive(Debug, Args)]
pub struct FilterArgs {
    /// Ne garde que les messages non lus.
    #[arg(long)]
    pub unread: bool,
    /// Ne garde que les messages suivis (drapeau).
    #[arg(long)]
    pub flagged: bool,
    /// Ne garde que les messages avec pièces jointes.
    #[arg(long)]
    pub with_attachments: bool,
    /// Ne garde que les messages dont l'expéditeur contient ce texte.
    #[arg(long)]
    pub from: Option<String>,
    /// Date minimale incluse (millisecondes Unix, AAAA-MM-JJ ou AAAA-MM-JJTHH:MM, UTC).
    #[arg(long)]
    pub since: Option<String>,
    /// Date maximale incluse (même format que --since).
    #[arg(long)]
    pub until: Option<String>,
}

impl FilterArgs {
    /// Traduit les options en filtres du contrat.
    pub fn message_filters(&self) -> Result<MessageFilters, MiraError> {
        Ok(MessageFilters {
            unread_only: self.unread,
            flagged_only: self.flagged,
            has_attachments: self.with_attachments,
            from_contains: self.from.clone(),
            since: self.since.as_deref().map(parse_instant).transpose()?,
            until: self.until.as_deref().map(parse_instant).transpose()?,
        })
    }
}
