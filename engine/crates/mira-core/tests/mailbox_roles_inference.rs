#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Rôle des dossiers : ordre SPECIAL-USE, INBOX, indice du provider, noms usuels (10.14).

#[allow(dead_code)]
mod provider_fakes;

use mira_vocabulary::MailboxRole;
use provider_fakes::role_inference::role;
use provider_fakes::{FakeFoldersProvider, FakeLabelsProvider};

/// Les sept attributs SPECIAL-USE donnent leur rôle, quelle que soit la casse
#[test]
fn special_use_attributes_give_their_role() {
    let table = [
        ("\\Sent", MailboxRole::Sent),
        ("\\Drafts", MailboxRole::Drafts),
        ("\\Trash", MailboxRole::Trash),
        ("\\Junk", MailboxRole::Junk),
        ("\\Archive", MailboxRole::Archive),
        ("\\All", MailboxRole::All),
        ("\\flagged", MailboxRole::Flagged),
    ];
    for (attribute, expected) in table {
        let found = role(
            &FakeLabelsProvider,
            "Nom quelconque",
            Some("/"),
            &["\\HasNoChildren", attribute],
        );
        assert_eq!(found, expected, "{attribute}");
    }
}

/// SPECIAL-USE gagne sur le nom usuel et sur l'indice du provider
#[test]
fn special_use_beats_names_and_provider_hint() {
    let provider = FakeFoldersProvider::new("fake_folders", "folders.test");
    assert_eq!(
        role(&provider, "Sent", Some("/"), &["\\Trash"]),
        MailboxRole::Trash
    );
    assert_eq!(
        role(&provider, "Corbeille", Some("/"), &["\\Trash"]),
        MailboxRole::Trash
    );
    assert_eq!(
        role(&provider, "INBOX", Some("/"), &["\\Archive"]),
        MailboxRole::Archive
    );
}

/// INBOX est reconnue sans tenir compte de la casse
#[test]
fn inbox_is_recognised_in_any_case() {
    for name in ["INBOX", "Inbox", "inbox"] {
        assert_eq!(
            role(&FakeLabelsProvider, name, Some("/"), &[]),
            MailboxRole::Inbox,
            "{name}"
        );
    }
}

/// L'indice du provider passe avant les noms usuels
#[test]
fn provider_hint_beats_common_names() {
    let folders = FakeFoldersProvider::new("fake_folders", "folders.test");
    assert_eq!(
        role(&folders, "Corbeille", Some("/"), &[]),
        MailboxRole::Junk
    );
    assert_eq!(
        role(&FakeLabelsProvider, "Corbeille", Some("/"), &[]),
        MailboxRole::Trash
    );
    assert_eq!(
        role(&FakeLabelsProvider, "Tout le courrier", Some("/"), &[]),
        MailboxRole::All
    );
}

/// Table des noms usuels en plusieurs langues, casse ignorée
#[test]
fn common_names_in_several_languages() {
    let table = [
        ("Sent Items", MailboxRole::Sent),
        ("Éléments envoyés", MailboxRole::Sent),
        ("GESENDET", MailboxRole::Sent),
        ("Enviados", MailboxRole::Sent),
        ("Brouillons", MailboxRole::Drafts),
        ("Entwürfe", MailboxRole::Drafts),
        ("Borradores", MailboxRole::Drafts),
        ("Deleted Messages", MailboxRole::Trash),
        ("Papierkorb", MailboxRole::Trash),
        ("Cestino", MailboxRole::Trash),
        ("Junk E-mail", MailboxRole::Junk),
        ("Courrier indésirable", MailboxRole::Junk),
        ("Spam", MailboxRole::Junk),
        ("Archives", MailboxRole::Archive),
        ("Archiv", MailboxRole::Archive),
    ];
    for (name, expected) in table {
        assert_eq!(
            role(&FakeLabelsProvider, name, Some("/"), &[]),
            expected,
            "{name}"
        );
    }
}

/// Le dernier segment du chemin décide ; un nom inconnu donne `Other`
#[test]
fn last_path_segment_decides_and_unknown_is_other() {
    assert_eq!(
        role(&FakeLabelsProvider, "INBOX.Sent", Some("."), &[]),
        MailboxRole::Sent
    );
    assert_eq!(
        role(&FakeLabelsProvider, "Perso/Brouillons", Some("/"), &[]),
        MailboxRole::Drafts
    );
    assert_eq!(
        role(&FakeLabelsProvider, "Archives/2023", Some("/"), &[]),
        MailboxRole::Other
    );
    assert_eq!(
        role(&FakeLabelsProvider, "Travail", None, &[]),
        MailboxRole::Other
    );
}
