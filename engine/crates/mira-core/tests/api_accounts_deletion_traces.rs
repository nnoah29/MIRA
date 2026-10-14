#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Suppression d'un compte (S14, vie privée) : ni la base, ni son journal WAL, ni son fichier de verrou ne gardent ses mots.

mod support;
mod support_api;
mod support_mirror;
mod support_read;

use std::fs;
use std::path::Path;

use mira_core::api::{AccountRequest, Request, Response};
use mira_vocabulary::MessageId;
use support_read::calls::run;
use support_read::world::{World, start_world};

const SUBJECT: &str = "Quetzalcoatl-sujet-secret";
const ADDRESS: &str = "xylophone.unique@efface.test";
const BODY: &str = "obsidienne-mot-du-corps";

/// Dit si un fichier du dossier (récursivement) contient ces octets
fn directory_contains(folder: &Path, needle: &str) -> bool {
    fs::read_dir(folder).unwrap().any(|entry| {
        let path = entry.unwrap().path();
        if path.is_dir() {
            return directory_contains(&path, needle);
        }
        let bytes = fs::read(&path).unwrap();
        bytes
            .windows(needle.len())
            .any(|window| window == needle.as_bytes())
    })
}

/// Garnit le miroir de messages indexés dont le sujet, l'adresse et le corps sont uniques
fn index_unique_words(world: &World) {
    world.seed_messages(40);
    world
        .mirror
        .write(|tx| {
            for index in 1..=40 {
                let id = MessageId::from_raw(format!("m{index}"));
                let subject = format!("{SUBJECT}-{index}");
                tx.index_message(&id, &subject, ADDRESS, &format!("{BODY} {index}"))?;
            }
            Ok(())
        })
        .unwrap();
}

/// Après la suppression du compte, aucun fichier du profil ne contient plus son sujet, son adresse ni son corps
#[tokio::test]
async fn regression_deleting_an_account_leaves_no_word_of_it_in_the_profile_files() {
    let world = start_world().await;
    index_unique_words(&world);
    let profile = world.fixture.home.path();
    assert!(directory_contains(profile, SUBJECT), "amorce du test");
    assert!(
        directory_contains(profile, "obsidienne"),
        "amorce de l'index"
    );
    let delete = Request::Account(AccountRequest::DeleteAccount {
        account: world.account.clone(),
    });
    assert_eq!(
        run(&world.fixture.engine, delete).await.unwrap(),
        Response::Done
    );
    for word in [
        SUBJECT,
        ADDRESS,
        BODY,
        "quetzalcoatl",
        "xylophone",
        "obsidienne",
    ] {
        assert!(!directory_contains(profile, word), "{word} gardé");
    }
}
