#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Interrupteur de l'historique des conversations d'IA, rangé dans les réglages du moteur.

use mira_core::storage::Database;
use mira_vocabulary::EngineSettings;

/// L'historique est activé par défaut, et le réglage est indépendant des réglages typés
#[test]
fn history_setting_defaults_to_true_and_is_independent() {
    let db = Database::open_in_memory().unwrap();
    assert!(db.read(|tx| tx.ai_history_enabled()).unwrap());
    db.write(|tx| tx.set_ai_history_enabled(false)).unwrap();
    assert!(!db.read(|tx| tx.ai_history_enabled()).unwrap());
    let settings = db.read(|tx| tx.get_engine_settings()).unwrap();
    assert_eq!(settings, EngineSettings::default());
    db.write(|tx| tx.put_engine_settings(&settings)).unwrap();
    assert!(!db.read(|tx| tx.ai_history_enabled()).unwrap());
    db.write(|tx| tx.set_ai_history_enabled(true)).unwrap();
    assert!(db.read(|tx| tx.ai_history_enabled()).unwrap());
}
