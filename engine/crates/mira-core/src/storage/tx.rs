//! Transaction de stockage : les dépôts sont des méthodes de `Tx` réparties par entité.

/// Transaction ouverte sur la base du profil ; seul point d'accès aux dépôts.
#[derive(Debug)]
pub struct Tx<'conn> {
    inner: rusqlite::Transaction<'conn>,
}

impl<'conn> Tx<'conn> {
    /// Enveloppe une transaction SQLite
    pub(crate) fn new(inner: rusqlite::Transaction<'conn>) -> Self {
        Self { inner }
    }

    /// Donne accès à la transaction SQLite aux dépôts du module de stockage
    pub(crate) fn sql(&self) -> &rusqlite::Transaction<'conn> {
        &self.inner
    }

    /// Valide la transaction
    pub(crate) fn commit(self) -> Result<(), rusqlite::Error> {
        self.inner.commit()
    }
}
