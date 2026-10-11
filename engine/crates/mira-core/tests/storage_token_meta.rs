#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Dépôt token_meta : expiration du jeton d'accès (migration account_token_meta).

mod support;

use mira_vocabulary::{AccountId, Timestamp};
use support::accounts::database_with_account;

/// L'expiration s'écrit, se remplace, se lit et s'efface avec None
#[test]
fn expiry_can_be_set_replaced_and_cleared() {
    let db = database_with_account("acc-1");
    let id = AccountId::from_raw("acc-1");
    assert_eq!(db.read(|tx| tx.token_expiry(&id)).unwrap(), None);
    db.write(|tx| tx.set_token_expiry(&id, Some(Timestamp::from_millis(10))))
        .unwrap();
    db.write(|tx| tx.set_token_expiry(&id, Some(Timestamp::from_millis(20))))
        .unwrap();
    assert_eq!(
        db.read(|tx| tx.token_expiry(&id)).unwrap(),
        Some(Timestamp::from_millis(20))
    );
    db.write(|tx| tx.set_token_expiry(&id, None)).unwrap();
    assert_eq!(db.read(|tx| tx.token_expiry(&id)).unwrap(), None);
}

/// La suppression du compte efface son expiration et laisse les autres comptes intacts
#[test]
fn deleting_the_account_removes_its_expiry_only() {
    let db = database_with_account("acc-1");
    let (first, second) = (AccountId::from_raw("acc-1"), AccountId::from_raw("acc-2"));
    let at = Some(Timestamp::from_millis(7));
    db.write(|tx| {
        tx.set_token_expiry(&first, at)?;
        tx.set_token_expiry(&second, at)
    })
    .unwrap();
    db.write(|tx| tx.delete_account(&first)).unwrap();
    assert_eq!(db.read(|tx| tx.token_expiry(&first)).unwrap(), None);
    assert_eq!(db.read(|tx| tx.token_expiry(&second)).unwrap(), at);
    assert!(
        db.read(|tx| tx.table_names())
            .unwrap()
            .contains(&"account_token_meta".to_string())
    );
}
