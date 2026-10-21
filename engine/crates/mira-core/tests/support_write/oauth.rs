//! Banc OAuth (S10) : compte OAuth2 de `oauth.test`, faux serveur d'autorisation, horloge manuelle, miroir garni.

use std::path::PathBuf;
use std::sync::Arc;

use mira_core::api::{Engine, EngineConfig};
use mira_core::ports::{Clock, ManualClock, MemorySecretStore};
use mira_core::profile::ProfilePaths;
use mira_core::storage::Database;
use mira_testkit::Credentials;
use mira_testkit::imap::{CapabilitySet, FakeImapServer, FakeMailStore};
use mira_testkit::oauth::FakeOAuthServer;
use mira_testkit::smtp::FakeSmtpServer;
use mira_vocabulary::{AccountId, CONTRACT_VERSION, Timestamp};

use super::START_MILLIS;
use super::seed::{seed_mirror, seed_server};
use crate::support_api::config_with_secrets;
use crate::support_api::endpoints::local_servers;
use crate::support_api::fixture::Fixture as ApiFixture;
use crate::support_api::oauth::{OAUTH_USER, add_oauth_account, point_oauth_at};

/// Banc de la façade avec un compte OAuth2 et l'horloge que le test pilote.
pub struct OauthBench {
    pub api: ApiFixture,
    pub clock: Arc<ManualClock>,
    pub account: AccountId,
    pub db: Database,
}

/// Configuration dont le domaine `oauth.test` mène au faux serveur IMAP et au faux serveur OAuth
fn oauth_config(
    home: &std::path::Path,
    secrets: &Arc<MemorySecretStore>,
    clock: &Arc<ManualClock>,
    servers: (&FakeImapServer, &FakeSmtpServer, &FakeOAuthServer),
) -> EngineConfig {
    let mut config = config_with_secrets(home, Arc::clone(secrets));
    config.clock = Arc::clone(clock) as Arc<dyn Clock>;
    let wiring = local_servers(servers.0.port(), servers.1.port());
    config
        .network
        .server_overrides
        .insert("oauth.test".to_string(), wiring);
    point_oauth_at(&mut config, servers.2);
    config
}

/// Faux serveurs de l'essai : courrier, identifiants, IMAP et autorisation OAuth
type Servers = (
    FakeMailStore,
    Credentials,
    FakeImapServer,
    FakeSmtpServer,
    FakeOAuthServer,
);

/// Démarre les faux serveurs et garnit le courrier de l'utilisateur OAuth
async fn start_servers() -> Servers {
    let store = FakeMailStore::new();
    let credentials = Credentials::default();
    seed_server(&store, OAUTH_USER);
    let caps = CapabilitySet::full();
    let imap = FakeImapServer::start(store.clone(), credentials.clone(), caps)
        .await
        .unwrap();
    let smtp = FakeSmtpServer::start(credentials.clone()).await.unwrap();
    let oauth = FakeOAuthServer::start(credentials.clone()).await.unwrap();
    (store, credentials, imap, smtp, oauth)
}

/// Ouvre le moteur sur ces serveurs et rend le banc de la façade avec le chemin de la base du profil
fn open_api(servers: Servers, clock: &Arc<ManualClock>) -> (ApiFixture, PathBuf) {
    let (store, credentials, imap, smtp, oauth) = servers;
    let home = tempfile::tempdir().unwrap();
    let secrets = Arc::new(MemorySecretStore::new());
    let config = oauth_config(home.path(), &secrets, clock, (&imap, &smtp, &oauth));
    let paths = ProfilePaths::resolve(&config.profile, &config.environment).unwrap();
    let engine = Engine::open(config.clone(), CONTRACT_VERSION).unwrap();
    let api = ApiFixture {
        home,
        engine,
        config,
        secrets,
        imap,
        smtp,
        store,
        oauth,
        credentials,
    };
    (api, paths.database)
}

/// Ouvre un moteur, ajoute le compte OAuth2 (flux complet) et garnit serveur et miroir
pub async fn start() -> OauthBench {
    let clock = Arc::new(ManualClock::new(Timestamp::from_millis(START_MILLIS)));
    let (api, database) = open_api(start_servers().await, &clock);
    let account = add_oauth_account(&api, "k0").await.id;
    let db = Database::open(&database).unwrap();
    seed_mirror(&db, &account, &api.store, OAUTH_USER);
    OauthBench {
        api,
        clock,
        account,
        db,
    }
}
