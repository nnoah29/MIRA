#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Registre des échanges d'une connexion : un identifiant en cours ne peut pas être réutilisé.

use mira_daemon::session::Exchanges;

/// Un identifiant en cours est refusé, puis redevient libre une fois l'échange fini.
#[test]
fn an_id_in_flight_is_refused_until_the_exchange_finishes() {
    let exchanges = Exchanges::default();
    assert!(exchanges.begin_call(5));
    assert!(!exchanges.begin_call(5));
    assert!(exchanges.begin_call(6));
    exchanges.finish(5);
    assert!(exchanges.begin_call(5));
}
