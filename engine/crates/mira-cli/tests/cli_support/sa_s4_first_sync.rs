//! Décor du scénario S4 : serveur garni (réception, envoyés, vieux courrier) et compte ajouté sans synchro.

use mira_testkit::imap::CapabilitySet;

use super::sa_world::SaWorld;

pub const PASSWORD: &str = "pa55-SECRET-xyz";

/// Dépose les courriers : trois récents et un vieux en réception, un récent en Envoyés.
fn deliver_first_sync_mail(world: &SaWorld) {
    let store = &world.bench.store;
    store.create_mailbox(&world.user, "Sent", Some("\\Sent"));
    store.create_mailbox(&world.user, "Archive", Some("\\Archive"));
    store.create_mailbox(&world.user, "Trash", Some("\\Trash"));
    for subject in ["Alpha", "Beta", "Gamma"] {
        world.deliver("INBOX", subject, "");
    }
    world.deliver_aged("INBOX", "Ancien", 100);
    world.deliver("Sent", "Envoye", "");
}

/// Serveur garni, compte ajouté (sans synchro de fond) ; rend le monde et l'identifiant du compte.
pub async fn start_first_sync_world() -> (SaWorld, String) {
    let user = "alice@folders.test";
    let world = SaWorld::start(user, PASSWORD, CapabilitySet::full()).await;
    deliver_first_sync_mail(&world);
    let account = world.add_account(&[]).await;
    (world, account)
}
