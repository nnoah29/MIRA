//! Décor du scénario S6 : un corps préchargé, un message avec pièce jointe, un message en-tête seul.

use mira_testkit::imap::CapabilitySet;

use super::raw_mail::with_attachment;
use super::sa_mail::mail_with_notes;
use super::sa_world::SaWorld;

/// Compte synchronisé : deux courriers en réception (corps préchargés), un en Archive (en-tête seul).
pub struct ReadWorld {
    pub world: SaWorld,
    pub account: String,
    pub inbox: String,
    pub present: String,
    pub with_file: String,
    pub header_only: String,
    pub notes: String,
}

/// Identifiant du message de ce sujet dans la boîte, lu par `message list`.
pub async fn id_of_subject(world: &SaWorld, mailbox: &str, subject: &str) -> String {
    let args = ["message", "list", "--mailbox", mailbox, "--limit", "100"];
    let page = world.run_ok(&args).await;
    let items = page["data"]["items"].as_array().unwrap();
    let found = items.iter().find(|item| item["subject"] == subject);
    found.unwrap()["id"].as_str().unwrap().to_string()
}

/// Dépose sur le serveur deux courriers en réception et deux en Archive (dont un à pièce jointe).
fn deliver_read_mail(world: &SaWorld) {
    let store = &world.bench.store;
    store.create_mailbox(&world.user, "Archive", Some("\\Archive"));
    world.deliver("INBOX", "Alpha", "");
    store.deliver(&world.user, "INBOX", &with_attachment(), &[]);
    world.deliver("Archive", "Range", "");
    let notes = mail_with_notes(&world.user, "Notes");
    store.deliver(&world.user, "Archive", &notes, &[]);
}

/// Prépare le serveur, ajoute le compte et le synchronise ; le réseau reste établi.
pub async fn start_read_world() -> ReadWorld {
    let user = "alice@folders.test";
    let world = SaWorld::start(user, "pa55-SECRET-xyz", CapabilitySet::full()).await;
    deliver_read_mail(&world);
    let account = world.add_account(&[]).await;
    world.run_sync_ok(&["sync", "start"]).await;
    let inbox = world.mailbox_id(&account, "INBOX").await;
    let archive = world.mailbox_id(&account, "Archive").await;
    let present = id_of_subject(&world, &inbox, "Alpha").await;
    let with_file = id_of_subject(&world, &inbox, "Sujet m4").await;
    let header_only = id_of_subject(&world, &archive, "Range").await;
    let notes = id_of_subject(&world, &archive, "Notes").await;
    ReadWorld {
        world,
        account,
        inbox,
        present,
        with_file,
        header_only,
        notes,
    }
}

impl ReadWorld {
    /// Pièces jointes d'un message telles que `message attachments` les rend (tableau JSON).
    pub async fn attachments_of(&self, message: &str) -> serde_json::Value {
        let listed = self
            .world
            .run_ok(&["message", "attachments", message])
            .await;
        listed["data"].clone()
    }

    /// Identifiant de l'unique pièce jointe du message avec fichier, lu par `message attachments`.
    pub async fn attachment_id(&self) -> String {
        let listed = self.attachments_of(&self.with_file).await;
        listed[0]["id"].as_str().unwrap().to_string()
    }
}
