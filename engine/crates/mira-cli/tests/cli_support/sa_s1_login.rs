//! Autorisation OAuth du scénario S1 : navigateur simulé, jetons rangés, recherche de fuite.

use std::time::Duration;

use mira_assembly::mira_vocabulary::{AccountId, ProfileId, SecretKind, SecretRef};
use mira_assembly::ports::SecretStore;

use super::CliResult;
use super::oauth_bench::run_watching;
use super::sa_leak::tree_contains;
use super::sa_s1_gmail::{GMAIL_USER, GmailWorld};
use super::shared_buffer::SharedBuffer;

/// Identifiant du compte créé, lu dans la réponse `--json` de `account add-oauth`.
pub fn created_id(result: &CliResult) -> String {
    let document = super::parse_json(result);
    document["data"]["data"]["id"].as_str().unwrap().to_string()
}

/// Attend l'adresse d'autorisation sur stderr, puis approuve ou refuse comme le ferait l'utilisateur.
async fn answer_when_announced(err: &SharedBuffer, world: &GmailWorld, approve: bool) -> String {
    let url = loop {
        let text = err.text();
        if let Some(line) = text.lines().find(|line| line.starts_with("http")) {
            break line.to_string();
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    };
    if approve {
        world.oauth.approve(&url, GMAIL_USER).await.unwrap();
    } else {
        world.oauth.deny(&url).await.unwrap();
    }
    url
}

impl GmailWorld {
    /// Lance `account add-oauth` pendant que le navigateur simulé répond ; rend le résultat et l'adresse annoncée.
    pub async fn login(&self, extra: &[&str], approve: bool) -> (CliResult, String) {
        let err = SharedBuffer::default();
        let mut deps = self.deps();
        let base = ["account", "add-oauth", "--address", GMAIL_USER, "--json"];
        let line = [&base[..], extra, &["--profile", self.profile]].concat();
        let browser = answer_when_announced(&err, self, approve);
        let session = async { tokio::join!(run_watching(&mut deps, &line, err.clone()), browser) };
        let (result, url) = tokio::time::timeout(Duration::from_secs(60), session)
            .await
            .unwrap();
        let mut log = self.transcript.lock().unwrap();
        log.extend([result.stdout.clone(), result.stderr.clone()]);
        (result, url)
    }

    /// Jeton rangé au trousseau sous la clé (profil donné, compte, type), s'il y en a un.
    pub fn token(&self, profile: &str, account: &str, kind: SecretKind) -> Option<String> {
        let key = SecretRef {
            profile: ProfileId::from_raw(profile),
            account: AccountId::from_raw(account),
            kind,
        };
        let found = self.secrets.get(&key).unwrap();
        found.map(|secret| secret.expose().to_string())
    }

    /// Dit si ce texte apparaît dans une sortie du CLI ou dans un fichier du profil.
    pub fn leaked(&self, secret: &str) -> bool {
        let log = self.transcript.lock().unwrap();
        log.iter().any(|text| text.contains(secret)) || tree_contains(self.home.path(), secret)
    }
}
