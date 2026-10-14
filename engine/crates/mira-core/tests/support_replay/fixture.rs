//! Banc complet : monde local d'acc-1, faux serveur IMAP garni comme le miroir, faux serveur SMTP.

use mira_core::actions::{ReplayContext, ReplayReport, replay_actions};
use mira_core::imap::ImapClient;
use mira_testkit::imap::CapabilitySet;
use mira_testkit::smtp::FakeSmtpServer;
use mira_vocabulary::{AccountId, MailboxId, MailboxRole, ProviderQuirks, ServerEndpoint};

use crate::support_actions::world::World;
use crate::support_imap::env::{Env, start};
use crate::support_imap::logins::connected;
use crate::support_imap::mail::mail;
use crate::support_mirror::fixtures::sample_mailbox;
use crate::support_smtp::server::{endpoint, started};

use super::credentials::FakeCredentials;

const SERVER_BOXES: [&str; 7] = [
    "Archive", "Trash", "Junk", "Sent", "Projects", "Work", "Drafts",
];

/// Tout ce qu'un test de rejeu manipule.
#[derive(Debug)]
pub struct Fixture {
    pub world: World,
    pub env: Env,
    pub smtp: FakeSmtpServer,
    pub smtp_endpoint: ServerEndpoint,
    pub credentials: FakeCredentials,
    pub quirks: ProviderQuirks,
    pub account: AccountId,
}

impl Fixture {
    /// Banc dont le serveur IMAP contient les mêmes messages que le miroir : msg-1 à msg-3 dans la réception (UID 1 à 3), msg-3 aussi dans « Work »
    pub async fn start(caps: CapabilitySet, quirks: ProviderQuirks) -> Self {
        let world = World::seeded();
        let env = start(caps).await;
        for name in SERVER_BOXES {
            env.store.create_mailbox("alice", name, None);
        }
        for id in ["msg-1", "msg-2", "msg-3"] {
            let raw = mail(&format!("Sujet {id}"), &format!("{id}@example.org"));
            env.store.deliver("alice", "INBOX", &raw, &[]);
        }
        let third = mail("Sujet msg-3", "msg-3@example.org");
        env.store.deliver("alice", "Work", &third, &[]);
        let (smtp, _) = started().await;
        let smtp_endpoint = endpoint(&smtp);
        let fixture = Self {
            world,
            env,
            smtp,
            smtp_endpoint,
            credentials: FakeCredentials::valid(),
            quirks,
            account: AccountId::from_raw("acc-1"),
        };
        fixture.align_mirror_with_server();
        fixture
    }

    /// Cale la validité d'UID de la réception du miroir sur celle du serveur et ajoute la boîte Brouillons
    fn align_mirror_with_server(&self) {
        let validity = self.env.store.uidvalidity("alice", "INBOX").unwrap();
        self.world
            .db
            .write(|tx| {
                let marker = mira_vocabulary::SyncMarker {
                    uid_validity: validity,
                    uid_next: 10,
                    highest_modseq: None,
                };
                tx.set_mailbox_marker(&MailboxId::from_raw("box-in"), Some(marker))?;
                let drafts = sample_mailbox("box-drafts", "acc-1", "Drafts", MailboxRole::Drafts);
                tx.upsert_mailbox(&drafts).map(|_| ())
            })
            .unwrap();
    }

    /// Contexte de rejeu du compte acc-1
    pub fn ctx(&self) -> ReplayContext<'_> {
        ReplayContext {
            queue: self.world.ctx(),
            account: &self.account,
            quirks: &self.quirks,
            credentials: &self.credentials,
            smtp: Some(&self.smtp_endpoint),
        }
    }

    /// Nouvelle session IMAP authentifiée (« redémarrage » de la connexion)
    pub async fn client(&self) -> ImapClient {
        connected(&self.env).await
    }

    /// Fait avancer l'horloge du monde local
    pub fn advance(&self, millis: i64) {
        self.world.clock.advance_millis(millis);
    }

    /// Une passe de rejeu sur la session donnée
    pub async fn replay(&self, client: &mut ImapClient) -> ReplayReport {
        replay_actions(&self.ctx(), client).await.unwrap()
    }
}
