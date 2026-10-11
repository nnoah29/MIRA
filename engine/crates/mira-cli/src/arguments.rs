//! Définition de la ligne de commande `mira` (clap).

use clap::{Parser, Subcommand};

/// Client en ligne de commande de MIRA.
#[derive(Debug, Parser)]
#[command(
    name = "mira",
    version,
    about = "Client mail MIRA en ligne de commande"
)]
pub struct Cli {
    /// Écrit le résultat sous forme d'enveloppe JSON versionnée.
    #[arg(long, global = true)]
    pub json: bool,
    #[command(subcommand)]
    pub command: Commands,
}

/// Domaines de commandes.
#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Catalogue des providers et détection.
    #[command(subcommand)]
    Provider(ProviderCommand),
}

/// Commandes du domaine `provider`.
#[derive(Debug, Subcommand)]
pub enum ProviderCommand {
    /// Liste les providers connus, le repli en dernier.
    List,
    /// Détecte le provider d'une adresse et affiche ses serveurs.
    Detect {
        /// Adresse e-mail à analyser.
        address: String,
    },
}
