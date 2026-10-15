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
    hint("n", "ajouter un compte"),
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
    hint("r", "répondre"),
    hint("a", "répondre à tous"),
    hint("f", "transférer"),
    hint("c", "nouveau message"),
    hint("z", "défaire"),
    hint("Q", "file d'envoi"),
    hint("B", "brouillons"),
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
    hint("r", "répondre"),
    hint("a", "répondre à tous"),
    hint("f", "transférer"),
    hint("c", "nouveau message"),
    hint("z", "défaire"),
    hint("Q", "file d'envoi"),
    hint("B", "brouillons"),
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
    hint("r", "répondre"),
    hint("a", "répondre à tous"),
    hint("f", "transférer"),
    hint("c", "nouveau message"),
    hint("z", "défaire"),
    hint("Q", "file d'envoi"),
    hint("B", "brouillons"),
];

const SEARCH_INPUT: &[Hint] = &[
    hint("Entrée", "chercher"),
    hint("Échap", "annuler"),
    hint("Retour arrière", "effacer"),
];

const CREATING: &[Hint] = &[hint("Échap", "annuler")];

const COMPOSE: &[Hint] = &[
    hint("F5/Ctrl+Entrée", "envoyer"),
    hint("Ctrl+s", "enregistrer"),
    hint("Tab/Maj+Tab", "champ suivant/précédent"),
    hint("Échap", "fermer"),
];

const CONFIRM_CLOSE: &[Hint] = &[
    hint("s/Entrée", "enregistrer"),
    hint("D", "abandonner (à confirmer)"),
    hint("c/Échap", "continuer"),
];

const CONFIRM_DISCARD: &[Hint] = &[
    hint("D", "abandonner le brouillon"),
    hint("autre touche", "revenir"),
];

const CONFIRM_SEND: &[Hint] = &[hint("Entrée/y", "envoyer"), hint("Échap/n", "annuler")];

const RESOLVE_UNCERTAIN: &[Hint] = &[
    hint("r", "renvoyer"),
    hint("n", "ne pas renvoyer"),
    hint("Échap", "plus tard"),
];

const QUEUE: &[Hint] = &[
    hint("j/k", "fiche suivante/précédente"),
    hint("r", "réessayer"),
    hint("x", "annuler"),
    hint("u/Entrée", "décider d'un envoi incertain"),
    hint("Échap", "retour"),
];

const DRAFTS: &[Hint] = &[
    hint("Entrée", "rouvrir le brouillon"),
    hint("j/k", "brouillon suivant/précédent"),
    hint("D", "supprimer (à confirmer par D)"),
    hint("Échap", "retour"),
];

const CONFIRM_DELETE_DRAFT: &[Hint] = &[
    hint("D", "supprimer le brouillon"),
    hint("autre touche", "annuler"),
];

const CONFIRM: &[Hint] = &[hint("Entrée/y", "confirmer"), hint("Échap/n", "annuler")];

const MOVE_PICKER: &[Hint] = &[
    hint("Entrée", "déplacer"),
    hint("j/k", "choisir"),
    hint("Échap", "annuler"),
];

const ACCOUNT_FORM: &[Hint] = &[
    hint("Entrée", "valider / champ suivant"),
    hint("Tab/Maj+Tab", "champ suivant/précédent"),
    hint("Ctrl+t", "chiffrement du serveur"),
    hint("Échap", "fermer"),
];

const ACCOUNT_FORM_CONFIRM: &[Hint] = &[
    hint("Entrée/y", "abandonner la saisie"),
    hint("Échap/n", "continuer"),
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
        KeyContext::Creating => CREATING,
        KeyContext::Compose => COMPOSE,
        KeyContext::ConfirmClose => CONFIRM_CLOSE,
        KeyContext::ConfirmDiscard => CONFIRM_DISCARD,
        KeyContext::ConfirmSend => CONFIRM_SEND,
        KeyContext::ResolveUncertain => RESOLVE_UNCERTAIN,
        KeyContext::Queue => QUEUE,
        KeyContext::Drafts => DRAFTS,
        KeyContext::ConfirmDeleteDraft => CONFIRM_DELETE_DRAFT,
        KeyContext::Confirm => CONFIRM,
        KeyContext::MovePicker => MOVE_PICKER,
        KeyContext::Help => HELP,
        KeyContext::AccountForm => ACCOUNT_FORM,
        KeyContext::AccountFormConfirm => ACCOUNT_FORM_CONFIRM,
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
        KeyContext::Creating => "création du brouillon",
        KeyContext::Compose => "rédaction",
        KeyContext::ConfirmClose => "fermeture du brouillon",
        KeyContext::ConfirmDiscard => "abandon du brouillon",
        KeyContext::ConfirmSend => "confirmation d'envoi",
        KeyContext::ResolveUncertain => "envoi incertain",
        KeyContext::Queue => "file d'envoi et d'actions",
        KeyContext::Drafts => "brouillons",
        KeyContext::ConfirmDeleteDraft => "suppression du brouillon",
        KeyContext::Confirm => "confirmation",
        KeyContext::MovePicker => "déplacement",
        KeyContext::Help => "aide",
        KeyContext::AccountForm => "ajout de compte",
        KeyContext::AccountFormConfirm => "fermeture du formulaire de compte",
    }
}
