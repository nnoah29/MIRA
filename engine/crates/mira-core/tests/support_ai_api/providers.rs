//! Fournisseurs de test et moteur prêt : un fournisseur à adresse libre, un autre à adresse fixe.

use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use super::flaky_store::FlakyStore;
use crate::ai_models_support::{ModelsFixture, TestProvider};

pub const EDITABLE: &str = "test-provider";
pub const FIXED: &str = "fixed-provider";

/// Fournisseurs du registre de test : adresse libre puis adresse fixe
fn both_providers() -> Vec<TestProvider> {
    let mut fixed = TestProvider::new(FIXED);
    fixed.editable = false;
    vec![TestProvider::new(EDITABLE), fixed]
}

/// Moteur dont le registre contient le fournisseur à adresse libre puis celui à adresse fixe
pub async fn started() -> ModelsFixture {
    ModelsFixture::start_many(both_providers()).await
}

/// Comme `started`, avec un trousseau dont les suppressions échouent tant que l'indicateur est levé
pub async fn started_with_flaky_delete() -> (ModelsFixture, Arc<AtomicBool>) {
    let fail = Arc::new(AtomicBool::new(false));
    let flag = Arc::clone(&fail);
    let wrap = move |inner| -> Arc<dyn mira_core::ports::SecretStore> {
        Arc::new(FlakyStore::new(inner, flag))
    };
    (
        ModelsFixture::start_with_store(both_providers(), wrap).await,
        fail,
    )
}
