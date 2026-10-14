//! Faux serveurs de l'essai : courrier, identifiants, IMAP et SMTP.

use mira_testkit::Credentials;
use mira_testkit::imap::{CapabilitySet, FakeImapServer, FakeMailStore};
use mira_testkit::smtp::FakeSmtpServer;

use super::seed::seed_server;
use super::{PASSWORD, USER};

/// Les quatre éléments que le banc d'écriture garde.
pub struct Servers {
    pub store: FakeMailStore,
    pub credentials: Credentials,
    pub imap: FakeImapServer,
    pub smtp: FakeSmtpServer,
}

/// Démarre les serveurs pour l'utilisateur de test, son courrier déjà déposé
pub async fn start_servers() -> Servers {
    let store = FakeMailStore::new();
    let credentials = Credentials::default();
    credentials.add_password(USER, PASSWORD);
    seed_server(&store, USER);
    let caps = CapabilitySet::full();
    let imap = FakeImapServer::start(store.clone(), credentials.clone(), caps)
        .await
        .unwrap();
    let smtp = FakeSmtpServer::start(credentials.clone()).await.unwrap();
    Servers {
        store,
        credentials,
        imap,
        smtp,
    }
}
