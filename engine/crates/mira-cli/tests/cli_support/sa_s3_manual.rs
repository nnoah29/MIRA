//! Décor du scénario S3 : compte « Autre » dont les serveurs sont saisis à la main, sans redirection de domaine.

use mira_testkit::imap::CapabilitySet;

use super::CliResult;
use super::sa_world::SaWorld;

pub const OTHER_USER: &str = "carol@other.test";
pub const OTHER_PASSWORD: &str = "other-pass-9412";

/// Faux serveurs pour `carol@other.test`, sans aucune redirection du domaine : seuls les serveurs saisis comptent.
pub async fn start_other_world() -> SaWorld {
    let mut world = SaWorld::start(OTHER_USER, OTHER_PASSWORD, CapabilitySet::full()).await;
    world.domain = String::new();
    world
}

impl SaWorld {
    /// Lance `account add` avec un hôte IMAP saisi mais sans hôte SMTP : la saisie est incomplète.
    pub async fn account_with_lone_imap_host(&self, password: &str) -> CliResult {
        let line = [
            "account",
            "add",
            "--address",
            &self.user,
            "--password-stdin",
        ];
        let line = [&line[..], &["--imap-host", "127.0.0.1", "--json"]].concat();
        self.run_stdin(&format!("{password}\n"), &line).await
    }

    /// Nombre de comptes rendus par `account list`.
    pub async fn account_count(&self) -> usize {
        let listed = self.run_ok(&["account", "list"]).await;
        listed["data"].as_array().unwrap().len()
    }

    /// Lance `account <verbe>` avec des serveurs en clair saisis (IMAP sur `imap_port`, SMTP du faux serveur).
    pub async fn account_with_servers(
        &self,
        verb: &str,
        imap_port: u16,
        password: &str,
    ) -> CliResult {
        let (imap, smtp) = (imap_port.to_string(), self.bench.smtp.port().to_string());
        let line = ["account", verb, "--address", &self.user, "--password-stdin"];
        let hosts = ["--imap-host", "127.0.0.1", "--imap-port", &imap];
        let smtp_hosts = ["--smtp-host", "127.0.0.1", "--smtp-port", &smtp];
        let security = [
            "--imap-security",
            "plaintext",
            "--smtp-security",
            "plaintext",
        ];
        let all = [&line[..], &hosts, &smtp_hosts, &security, &["--json"]].concat();
        self.run_stdin(&format!("{password}\n"), &all).await
    }
}
