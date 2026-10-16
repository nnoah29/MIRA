#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! La fréquence de relevé demandée est bornée ; absente, le délai du fournisseur s'applique.

use std::time::Duration;

use mira_bridge::polling::supervisor_for;

/// Sans valeur, le moteur garde le délai propre à chaque fournisseur.
#[test]
fn no_value_keeps_the_provider_delay() {
    assert_eq!(supervisor_for(None).poll_interval, None);
}

/// Une valeur raisonnable est reprise telle quelle.
#[test]
fn a_reasonable_value_is_kept() {
    assert_eq!(
        supervisor_for(Some(60)).poll_interval,
        Some(Duration::from_secs(60))
    );
}

/// Trop court ou trop long : ramené au plancher ou au plafond.
#[test]
fn extreme_values_are_clamped() {
    assert_eq!(
        supervisor_for(Some(1)).poll_interval,
        Some(Duration::from_secs(30))
    );
    assert_eq!(
        supervisor_for(Some(999_999)).poll_interval,
        Some(Duration::from_secs(3600))
    );
}
