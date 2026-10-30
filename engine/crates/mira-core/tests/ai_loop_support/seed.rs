//! Boîte garnie pour les tests de la boucle d'outils : un compte, une boîte, trois messages dont deux en fil.
#![allow(dead_code)]

use mira_core::storage::Database;
use mira_vocabulary::{
    AccountId, AiConnection, AiConnectionId, AiProviderId, MailboxId, MaskingSettings, MessageId,
};

use crate::ai_models_support::ModelsFixture;
use crate::support::accounts::sample_account;
use crate::support_read::seed::{add_body, add_inbox, add_message, link_threads};

/// Compte `acc-1`, boîte `box-1`, messages m1 (bonjour), m2 (le devis de mars) et m3 (autre chose) ; m1 et m2 forment le fil `t-r1`.
pub fn seed(db: &Database) -> (AccountId, MailboxId) {
    let account = AccountId::from_raw("acc-1");
    db.write(|tx| tx.insert_account(&sample_account("acc-1")))
        .unwrap();
    let mailbox = add_inbox(db, &account);
    let bodies = ["bonjour", "le devis de mars", "autre chose"];
    for (index, text) in bodies.iter().enumerate() {
        let id = format!("m{}", index + 1);
        let uid = u32::try_from(index).unwrap() + 1;
        add_message(db, &account, &mailbox, &id, uid, i64::from(uid) * 1_000);
        add_body(db, &MessageId::from_raw(id), text, None);
    }
    link_threads(db, &account);
    (account, mailbox)
}

/// Connexion `conn-1` du fournisseur de test, visant le faux serveur (boucle locale), sans type autorisé.
pub fn local_connection(models: &ModelsFixture) -> AiConnection {
    AiConnection {
        id: AiConnectionId::from_raw("conn-1"),
        provider: AiProviderId("test-provider".to_string()),
        label: "Essai".to_string(),
        base_url: Some(models.server.base_url()),
        default_model: Some("model-a".to_string()),
        is_default: false,
        has_key: false,
        consent_at: None,
        models: None,
        allowed_kinds: Vec::new(),
        masking: MaskingSettings::remote_default(),
    }
}
