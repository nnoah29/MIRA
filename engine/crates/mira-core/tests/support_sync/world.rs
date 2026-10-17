//! Monde de test : faux serveur IMAP, base en mémoire, compte, contexte de synchronisation.

use std::sync::Arc;

use mira_core::accounts::NetworkPolicy;
use mira_core::events::{EventSink, RecordingSink};
use mira_core::ports::{SequentialIds, SystemClock};
use mira_core::storage::Database;
use mira_core::sync::{
    ConnectionBudget, CredentialProvider, SyncContext, SyncSession, open_connection,
};
use mira_testkit::Credentials;
use mira_testkit::imap::{CapabilitySet, FakeImapServer, FakeMailStore};
use mira_vocabulary::{
    Account, AccountId, AccountState, AuthMethodKind, EmailAddress, ProfileId, ProviderId,
    Security, ServerEndpoint, ServerSettings, SyncSettings, Timestamp,
};

use super::credentials::TestCredentials;
use super::mail::mail;
use super::provider::SyncTestProvider;

/// Tout ce qu'un test de synchronisation manipule.
#[derive(Debug)]
pub struct SyncWorld {
    pub server: Arc<FakeImapServer>,
    pub store: FakeMailStore,
    pub db: Arc<Database>,
    pub recording: Arc<RecordingSink>,
    pub ctx: SyncContext,
    pub credentials: Arc<dyn CredentialProvider>,
}

/// Démarre le faux serveur pour `alice` (mot de passe `secret`)
pub async fn start_server(caps: CapabilitySet) -> (Arc<FakeImapServer>, FakeMailStore) {
    let store = FakeMailStore::new();
    let credentials = Credentials::default();
    credentials.add_password("alice", "secret");
    let server = FakeImapServer::start(store.clone(), credentials, caps)
        .await
        .unwrap();
    (Arc::new(server), store)
}

/// Compte de test pointant vers le faux serveur, avec les réglages par défaut (90 jours, 50 corps)
pub fn account_for(server: &FakeImapServer) -> Account {
    let endpoint = ServerEndpoint {
        host: "127.0.0.1".to_string(),
        port: server.port(),
        security: Security::Plaintext,
    };
    Account {
        id: AccountId::from_raw("acc-sync"),
        profile: ProfileId::from_raw("default"),
        address: EmailAddress::parse("alice@example.org").unwrap(),
        display_name: "Alice".to_string(),
        provider: ProviderId::new("fake_sync"),
        auth: AuthMethodKind::AppPassword,
        servers: Some(ServerSettings {
            imap: endpoint.clone(),
            smtp: endpoint,
        }),
        sync: SyncSettings::default(),
        state: AccountState::Connected,
        created_at: Timestamp::from_millis(1_000),
    }
}

/// Assemble le monde autour d'un serveur déjà démarré et d'un destinataire d'événements
pub fn build_world(
    server: Arc<FakeImapServer>,
    store: FakeMailStore,
    provider: SyncTestProvider,
    recording: Arc<RecordingSink>,
    events: Arc<dyn EventSink>,
) -> SyncWorld {
    let db = Arc::new(Database::open_in_memory().unwrap());
    let account = account_for(&server);
    db.write(|tx| tx.insert_account(&account)).unwrap();
    let ctx = SyncContext {
        db: Arc::clone(&db),
        provider: Arc::new(provider),
        account,
        clock: Arc::new(SystemClock),
        ids: Arc::new(SequentialIds::new()),
        events,
        network: NetworkPolicy::default(),
    };
    SyncWorld {
        server,
        store,
        db,
        recording,
        ctx,
        credentials: Arc::new(TestCredentials::valid()),
    }
}

/// Monde complet avec un enregistreur d'événements simple
pub async fn start_world(caps: CapabilitySet, provider: SyncTestProvider) -> SyncWorld {
    let (server, store) = start_server(caps).await;
    let recording = Arc::new(RecordingSink::default());
    let events: Arc<dyn EventSink> = recording.clone();
    build_world(server, store, provider, recording, events)
}

/// Dépose `count` messages dont l'objet est « <préfixe> <n> » et dont l'identifiant de message est unique
pub fn deliver_many(world: &SyncWorld, mailbox: &str, prefix: &str, count: u32) {
    for n in 1..=count {
        let (subject, id) = (format!("{prefix} {n}"), format!("{prefix}{n}@x"));
        let raw = mail(&subject, &id);
        world.store.deliver("alice", mailbox, &raw, &[]);
    }
}

/// Ouvre une connexion de travail dans un budget de dix connexions et prépare la session
pub async fn open_session(world: &SyncWorld) -> SyncSession {
    let permit = ConnectionBudget::new(10).acquire().await.unwrap();
    let connection = open_connection(&world.ctx, permit, world.credentials.as_ref())
        .await
        .unwrap();
    SyncSession::new(world.ctx.clone(), connection)
}
