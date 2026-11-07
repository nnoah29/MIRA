//! Configuration du démon : options de ligne de commande, chemins par défaut, fichier de clé du trousseau.

mod file;
mod key;
mod paths;

use std::path::PathBuf;
use std::time::Duration;

use clap::Parser;

pub use file::{FileConfig, config_file_path, read_file_config};
pub use key::read_secrets_key;
pub use paths::{DaemonPaths, environment_of, resolve_paths};

/// Options du démon ; elles l'emportent sur le fichier `daemon.toml`.
#[derive(Debug, Parser)]
#[command(
    name = "mira-daemon",
    about = "Démon MIRA : héberge le moteur et expose le contrat JSON"
)]
pub struct DaemonConfig {
    /// Profil à ouvrir (défaut : `default`)
    #[arg(long)]
    pub profile: Option<String>,
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
    /// Fichier de configuration (défaut : `daemon.toml` du dossier MIRA)
    #[arg(long)]
    pub config_file: Option<PathBuf>,
    /// S'arrêter quand le dernier client est parti depuis `--idle-grace-secs` (démon lancé à la demande)
    #[arg(long)]
    pub exit_when_idle: bool,
    /// Délai de grâce avant l'arrêt à l'inactivité, en secondes
    #[arg(long, default_value_t = 30)]
    pub idle_grace_secs: u64,
}

impl DaemonConfig {
    /// Nom du profil à ouvrir.
    pub fn profile(&self) -> &str {
        self.profile.as_deref().unwrap_or("default")
    }

    /// Délai de grâce de l'arrêt à l'inactivité, si le démon doit s'arrêter ainsi.
    pub fn idle_grace(&self) -> Option<Duration> {
        self.exit_when_idle
            .then(|| Duration::from_secs(self.idle_grace_secs))
    }

    /// Complète les options absentes avec les valeurs du fichier.
    pub fn merged_with(self, file: FileConfig) -> Self {
        Self {
            profile: self.profile.or(file.profile),
            home: self.home.or(file.home),
            socket: self.socket.or(file.socket),
            token_file: self.token_file.or(file.token_file),
            ..self
        }
    }
}
