#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Présence côté pont : l'app embarquée déclare son focus à l'hôte du processus.

use mira_bridge::api::presence::set_presence;
use mira_bridge::session::host;

/// Le focus déclaré rend l'hôte actif ; le perdre le désactive.
#[test]
fn set_presence_marks_the_process_host_active_then_inactive() {
    set_presence(true);
    assert!(host().presence().is_active());
    set_presence(false);
    assert!(!host().presence().is_active());
}
