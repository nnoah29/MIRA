//! Courriers déposés sur le faux serveur pour les recherches côté serveur.

use super::mail;
use super::seed::add_message;
use super::world::World;

/// Dépose trois courriers (le premier et le dernier parlent d'une licorne) ; le premier est connu du miroir sous `m1`
pub fn deliver_unicorn_mails(world: &World) {
    let store = &world.fixture.store;
    let user = world.user();
    let first = store.deliver(user, "INBOX", &mail::plain("a", "une licorne"), &[]);
    store.deliver(user, "INBOX", &mail::plain("b", "rien"), &[]);
    store.deliver(user, "INBOX", &mail::plain("c", "encore une licorne"), &[]);
    let (db, account, mailbox) = (&world.mirror, &world.account, &world.mailbox);
    add_message(db, account, mailbox, "m1", first, 1_000);
}
