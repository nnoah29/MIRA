//! Commandes `mira maintenance …` : volumes, caches, index, intégrité, exports sans secret.

use clap::{Args, Subcommand};
use mira_assembly::api::{MaintenanceRequest, Request};

/// Fichier de destination d'un export.
#[derive(Debug, Args)]
pub struct DestinationArg {
    /// Fichier à écrire (jamais de secret dedans).
    pub destination: String,
}

/// Commandes du domaine `maintenance`.
#[derive(Debug, Subcommand)]
pub enum MaintenanceCommand {
    /// Affiche les volumes du miroir.
    Stats,
    /// Supprime les pièces jointes téléchargées (le serveur les garde).
    PurgeAttachments,
    /// Reconstruit l'index de recherche locale.
    RebuildIndex,
    /// Contrôle l'intégrité de la base.
    Check,
    /// Écrit un diagnostic sans adresse ni secret.
    ExportDiagnostic(DestinationArg),
    /// Écrit la configuration et les intentions du profil, sans secret ni miroir.
    ExportProfile(DestinationArg),
}

/// Traduit une commande `maintenance` en requête du contrat.
pub fn build_request(command: &MaintenanceCommand) -> Request {
    let request = match command {
        MaintenanceCommand::Stats => MaintenanceRequest::GetStorageStats,
        MaintenanceCommand::PurgeAttachments => MaintenanceRequest::PurgeAttachmentCache,
        MaintenanceCommand::RebuildIndex => MaintenanceRequest::RebuildSearchIndex,
        MaintenanceCommand::Check => MaintenanceRequest::CheckIntegrity,
        MaintenanceCommand::ExportDiagnostic(target) => MaintenanceRequest::ExportDiagnostic {
            destination: target.destination.clone(),
        },
        MaintenanceCommand::ExportProfile(target) => MaintenanceRequest::ExportProfile {
            destination: target.destination.clone(),
        },
    };
    Request::Maintenance(request)
}
