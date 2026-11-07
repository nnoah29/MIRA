#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Composition des notifications d'un compte : une par message jusqu'à trois, un résumé au-delà, aperçu respecté.

use mira_assembly::mira_vocabulary::NotificationSettings;
use mira_host::notifier::{Arrival, Texts, compose};

/// Arrivée numéro `n` du compte `a1`.
fn arrival(n: usize) -> Arrival {
    Arrival {
        account: "a1".into(),
        address: "moi@exemple.test".into(),
        message: format!("a1/m{n}"),
        sender: format!("Expéditeur {n}"),
        subject: format!("Sujet {n}"),
    }
}

/// Jusqu'à trois messages : une notification chacune, avec expéditeur et objet.
#[test]
fn up_to_three_arrivals_get_one_notification_each() {
    let arrivals: Vec<_> = (1..=3).map(arrival).collect();
    let shown = compose(
        &arrivals,
        &NotificationSettings::default(),
        &Texts::for_locale("fr"),
    );
    assert_eq!(shown.len(), 3);
    assert_eq!(shown[0].title, "Expéditeur 1");
    assert_eq!(shown[0].body, "Sujet 1");
    assert_eq!(shown[0].message.as_deref(), Some("a1/m1"));
    assert_ne!(shown[0].id, shown[1].id);
}

/// Plus de trois messages : un seul résumé qui nomme le compte et ouvre le premier message.
#[test]
fn more_than_three_arrivals_collapse_into_one_summary() {
    let arrivals: Vec<_> = (1..=5).map(arrival).collect();
    let shown = compose(
        &arrivals,
        &NotificationSettings::default(),
        &Texts::for_locale("fr"),
    );
    assert_eq!(shown.len(), 1);
    assert_eq!(shown[0].title, "5 nouveaux messages");
    assert_eq!(shown[0].body, "moi@exemple.test");
    assert_eq!(shown[0].message.as_deref(), Some("a1/m1"));
}

/// Sans aperçu, ni l'expéditeur ni l'objet n'apparaissent.
#[test]
fn without_preview_neither_sender_nor_subject_appears() {
    let settings = NotificationSettings {
        preview: false,
        ..NotificationSettings::default()
    };
    let shown = compose(&[arrival(1)], &settings, &Texts::for_locale("fr"));
    let text = format!("{} {}", shown[0].title, shown[0].body);
    assert!(
        !text.contains("Expéditeur") && !text.contains("Sujet"),
        "{text}"
    );
    assert_eq!(shown[0].title, "Nouveau message");
}

/// Le son suit le réglage, et aucune arrivée ne donne aucune notification.
#[test]
fn sound_follows_the_setting_and_no_arrival_gives_nothing() {
    let quiet = NotificationSettings {
        sound: false,
        ..NotificationSettings::default()
    };
    let texts = Texts::for_locale("fr");
    assert!(!compose(&[arrival(1)], &quiet, &texts)[0].sound);
    assert!(compose(&[], &quiet, &texts).is_empty());
}
