//! Une requête illisible ou un moteur fermé donnent une enveloppe d'erreur, jamais un plantage.

use serde_json::Value;

use super::driver::Driver;

/// Une requête illisible ou un moteur fermé donnent une enveloppe d'erreur, jamais un plantage.
pub async fn errors_are_reported_as_envelopes<D: Driver>() {
    let driver = D::start(None).await;
    let closed: Value = serde_json::from_str(&driver.execute("{}").await).unwrap();
    assert_eq!(closed["ok"], false);
    assert_eq!(closed["contract_version"]["major"], 1);
}
