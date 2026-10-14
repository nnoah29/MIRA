//! Rappel des touches de chaque contexte, pour l'aide et le pied de page.

use super::context::KeyContext;

/// Une touche (ou un groupe de touches) et ce qu'elle fait.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hint {
    pub keys: &'static str,
    pub action: &'static str,
}

/// Raccourci de construction d'un rappel.
const fn hint(keys: &'static str, action: &'static str) -> Hint {
    Hint { keys, action }
}

const SIDEBAR: &[Hint] = &[
    hint("Entrée", "ouvrir"),
    hint("j/k", "boîte suivante/précédente"),
    hint("]/[", "compte suivant/précédent"),
    hint("g/G", "début/fin"),
];

const LIST: &[Hint] = &[
    hint("Entrée", "ouvrir"),
    hint("e", "archiver"),
    hint("d", "corbeille"),
    hint("D", "supprimer définitivement"),
    hint("s", "étoile"),
    hint("u", "lu/non lu"),
    hint("m", "déplacer"),
    hint("z", "défaire"),
    hint("j/k", "message suivant/précédent"),
    hint("g/G", "début/fin"),
    hint("Échap", "retour"),
];

const SEARCH_RESULTS: &[Hint] = &[
    hint("Échap", "quitter la recherche"),
    hint("Entrée", "ouvrir"),
    hint("e", "archiver"),
    hint("d", "corbeille"),
    hint("D", "supprimer définitivement"),
    hint("s", "étoile"),
    hint("u", "lu/non lu"),
    hint("m", "déplacer"),
    hint("z", "défaire"),
    hint("j/k", "résultat suivant/précédent"),
];

const READER: &[Hint] = &[
    hint("Échap", "retour"),
    hint("j/k", "défiler"),
    hint("Entrée", "télécharger le corps"),
    hint("e", "archiver"),
    hint("d", "corbeille"),
    hint("D", "supprimer définitivement"),
    hint("s", "étoile"),
    hint("u", "lu/non lu"),
    hint("m", "déplacer"),
    hint("z", "défaire"),
];

const SEARCH_INPUT: &[Hint] = &[
    hint("Entrée", "chercher"),
    hint("Échap", "annuler"),
    hint("Retour arrière", "effacer"),
];

const CONFIRM: &[Hint] = &[hint("Entrée/y", "confirmer"), hint("Échap/n", "annuler")];

const MOVE_PICKER: &[Hint] = &[
    hint("Entrée", "déplacer"),
    hint("j/k", "choisir"),
    hint("Échap", "annuler"),
];

const HELP: &[Hint] = &[hint("Échap/?/q", "fermer l'aide")];

pub const GLOBAL: &[Hint] = &[
    hint("Tab/Maj+Tab", "changer de panneau"),
    hint("/", "recherche"),
    hint("Ctrl+r", "synchroniser"),
    hint("?", "aide"),
    hint("q", "quitter"),
];

/// Touches propres à ce contexte, les plus utiles d'abord.
pub fn hints_for(context: KeyContext) -> &'static [Hint] {
    match context {
        KeyContext::Sidebar => SIDEBAR,
        KeyContext::List => LIST,
        KeyContext::SearchResults => SEARCH_RESULTS,
        KeyContext::Reader => READER,
        KeyContext::SearchInput => SEARCH_INPUT,
        KeyContext::Confirm => CONFIRM,
        KeyContext::MovePicker => MOVE_PICKER,
        KeyContext::Help => HELP,
    }
}

/// Nom du contexte, en titre de l'aide.
pub fn context_name(context: KeyContext) -> &'static str {
    match context {
        KeyContext::Sidebar => "comptes et boîtes",
        KeyContext::List => "liste des messages",
        KeyContext::SearchResults => "résultats de recherche",
        KeyContext::Reader => "lecture",
        KeyContext::SearchInput => "saisie de recherche",
        KeyContext::Confirm => "confirmation",
        KeyContext::MovePicker => "déplacement",
        KeyContext::Help => "aide",
    }
}
