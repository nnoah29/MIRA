//! Écritures sur les identités : insertion, mise à jour, identité par défaut.

use mira_vocabulary::{AccountId, Identity, IdentityId, MiraError};
use rusqlite::params;

use super::rows::identity_not_found;
use crate::storage::Tx;
use crate::storage::errors::storage_error;

impl Tx<'_> {
    /// Insère une identité ; si elle est par défaut, elle remplace l'ancienne identité par défaut
    pub fn insert_identity(&self, identity: &Identity) -> Result<(), MiraError> {
        if identity.is_default {
            self.clear_default_identity(&identity.account)?;
        }
        self.sql()
            .execute(
                "INSERT INTO identities (id, account_id, address, display_name, signature, is_default) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    identity.id.as_str(),
                    identity.account.as_str(),
                    identity.address.as_str(),
                    identity.display_name,
                    identity.signature,
                    identity.is_default,
                ],
            )
            .map_err(storage_error)?;
        Ok(())
    }

    /// Met à jour l'adresse, le nom et la signature (le statut par défaut ne change pas ici)
    pub fn update_identity(&self, identity: &Identity) -> Result<(), MiraError> {
        let changed = self
            .sql()
            .execute(
                "UPDATE identities SET address = ?2, display_name = ?3, signature = ?4 WHERE id = ?1",
                params![
                    identity.id.as_str(),
                    identity.address.as_str(),
                    identity.display_name,
                    identity.signature,
                ],
            )
            .map_err(storage_error)?;
        if changed == 0 {
            return Err(identity_not_found());
        }
        Ok(())
    }

    /// Désigne l'identité par défaut d'un compte ; il n'en reste jamais plus d'une
    pub fn set_default_identity(
        &self,
        account: &AccountId,
        identity: &IdentityId,
    ) -> Result<(), MiraError> {
        let owned: i64 = self
            .sql()
            .query_row(
                "SELECT COUNT(*) FROM identities WHERE id = ?1 AND account_id = ?2",
                [identity.as_str(), account.as_str()],
                |row| row.get(0),
            )
            .map_err(storage_error)?;
        if owned == 0 {
            return Err(identity_not_found());
        }
        self.clear_default_identity(account)?;
        self.sql()
            .execute(
                "UPDATE identities SET is_default = 1 WHERE id = ?1",
                [identity.as_str()],
            )
            .map_err(storage_error)?;
        Ok(())
    }

    /// Retire le statut par défaut de toutes les identités d'un compte
    fn clear_default_identity(&self, account: &AccountId) -> Result<(), MiraError> {
        self.sql()
            .execute(
                "UPDATE identities SET is_default = 0 WHERE account_id = ?1",
                [account.as_str()],
            )
            .map_err(storage_error)?;
        Ok(())
    }
}
