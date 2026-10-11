//! Écriture des listes de drapeaux.

/// Liste de drapeaux entre parenthèses
pub(crate) fn flag_list(flags: &[String]) -> String {
    format!("({})", flags.join(" "))
}
