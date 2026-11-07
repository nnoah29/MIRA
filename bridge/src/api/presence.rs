//! Présence du client embarqué : l'app déclare si sa fenêtre est au premier plan.

use flutter_rust_bridge::frb;

/// Déclare si la fenêtre de l'app est au premier plan (sert à décider d'une notification).
#[frb(sync)]
pub fn set_presence(focused: bool) {
    crate::session::host().presence().set("embedded", focused);
}
