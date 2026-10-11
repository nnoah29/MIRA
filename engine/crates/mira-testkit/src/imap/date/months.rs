//! Noms abrégés des mois utilisés par IMAP.

const MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

/// Nom abrégé du mois (1 à 12)
pub(crate) fn month_name(month: u32) -> &'static str {
    MONTHS
        .get((month as usize).wrapping_sub(1))
        .copied()
        .unwrap_or("Jan")
}

/// Numéro du mois (1 à 12) pour un nom abrégé, sans tenir compte de la casse
pub(crate) fn month_number(name: &str) -> Option<u32> {
    MONTHS
        .iter()
        .position(|known| known.eq_ignore_ascii_case(name))
        .map(|index| index as u32 + 1)
}
