//! Configuration du démon : options de ligne de commande, chemins par défaut, fichier de clé du trousseau.

mod key;
mod paths;

use std::path::PathBuf;

use clap::Parser;

pub use key::read_secrets_key;
pub use paths::{DaemonPaths, environment_of, resolve_paths};

/// Options du démon ; il n'y a pas de fichier de configuration avant la phase 4.
#[derive(Debug, Parser)]
#[command(
    name = "mira-daemon",
    about = "Démon MIRA : héberge le moteur et expose le contrat JSON"
)]
pub struct DaemonConfig {
    /// Profil à ouvrir
    #[arg(long, default_value = "default")]
    pub profile: String,
    /// Dossier de données de MIRA (comme MIRA_HOME)
    #[arg(long)]
    pub home: Option<PathBuf>,
    /// Relevé périodique en secondes (30 à 3600), sinon le délai du fournisseur
    #[arg(long)]
    pub poll_secs: Option<u32>,
    /// Fichier de 32 octets (0600) qui remplace le trousseau du système par un fichier chiffré
    #[arg(long)]
    pub secrets_key_file: Option<PathBuf>,
    /// Socket Unix (défaut : dossier d'exécution de l'utilisateur)
    #[arg(long)]
    pub socket: Option<PathBuf>,
    /// Port TCP loopback (0 : port libre) ; absent, pas d'écoute TCP
    #[arg(long)]
    pub tcp_port: Option<u16>,
    /// Fichier du jeton local (défaut : dossier du profil)
    #[arg(long)]
    pub token_file: Option<PathBuf>,
}
