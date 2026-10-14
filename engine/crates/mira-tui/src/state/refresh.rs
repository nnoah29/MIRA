//! Rechargements déjà demandés au moteur et pas encore répondus, pour éviter les rafales.

/// Un drapeau par cible rechargée sur événement.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RefreshState {
    pub mailboxes: bool,
    pub status: bool,
    pub sync: bool,
}
