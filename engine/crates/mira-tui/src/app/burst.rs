//! Application bornée des messages déjà arrivés : un dessin par rafale, sans priver le clavier.

pub const BURST_LIMIT: usize = 256;

/// Raison de la fin d'une rafale.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Drained {
    Empty,
    Budget,
    Quit,
}

/// Prend et applique au plus `limit` éléments ; s'arrête sur file vide ou demande de sortie (`apply` rend vrai).
pub fn drain_bounded<C, T>(
    context: &mut C,
    mut next: impl FnMut(&mut C) -> Option<T>,
    mut apply: impl FnMut(&mut C, T) -> bool,
    limit: usize,
) -> Drained {
    for _ in 0..limit {
        let Some(item) = next(context) else {
            return Drained::Empty;
        };
        if apply(context, item) {
            return Drained::Quit;
        }
    }
    Drained::Budget
}
