//! Fournisseurs de test et moteur prêt : un fournisseur à adresse libre, un autre à adresse fixe.

use crate::ai_models_support::{ModelsFixture, TestProvider};

pub const EDITABLE: &str = "test-provider";
pub const FIXED: &str = "fixed-provider";

/// Moteur dont le registre contient le fournisseur à adresse libre puis celui à adresse fixe
pub async fn started() -> ModelsFixture {
    let mut fixed = TestProvider::new(FIXED);
    fixed.editable = false;
    ModelsFixture::start_many(vec![TestProvider::new(EDITABLE), fixed]).await
}
