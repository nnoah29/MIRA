#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Règles pures des libellés système : expéditeurs automatiques, mentions explicites, verdict sur le dernier message.

use mira_core::system_labels::automated::is_automated_address;
use mira_core::system_labels::classify::{LABEL_HORIZON_MS, Verdict, WAITING_AFTER_MS, classify};
use mira_core::system_labels::mention::mentions_me;
use support_label_rules::{NOW, me, mine, received};

mod support_label_rules;

/// Les adresses d'envoi automatique sont reconnues, les adresses humaines non
#[test]
fn automated_addresses_are_recognised() {
    for a in [
        "noreply@x.com",
        "No-Reply@x.com",
        "do_not_reply@x.com",
        "notifications@tally.so",
        "notification+abc@x.com",
        "mailer-daemon@x.com",
        "bounce-123@x.com",
        "postmaster@x.com",
    ] {
        assert!(is_automated_address(a), "{a}");
    }
    for a in [
        "ana@ex.org",
        "reply@x.com",
        "noah.reply@x.com",
        "bob",
        "info@x.com",
    ] {
        assert!(!is_automated_address(a), "{a}");
    }
}

/// `@prénom`, `@nom` et l'adresse sont des mentions ; une simple salutation ne l'est pas
#[test]
fn explicit_mentions_only() {
    assert!(mentions_me("Merci @Noah de regarder", &me()));
    assert!(mentions_me("avis de @noah toffa ?", &me()));
    assert!(mentions_me("Écris à alias@mira.dev.", &me()));
    assert!(!mentions_me("Bonjour Noah, ça va ?", &me()));
}

/// Adresse ou nom collés à autre chose, texte cité et signature : pas une mention
#[test]
fn glued_quoted_and_signature_text_is_not_a_mention() {
    assert!(!mentions_me("voir joe@noahcorp.com", &me()));
    assert!(!mentions_me("voir xnoah@mira.dev", &me()));
    assert!(!mentions_me("voir noah@mira.dev.fr", &me()));
    assert!(!mentions_me("> @noah regarde\nOK", &me()));
    assert!(!mentions_me("OK merci\n--\n@noah toffa", &me()));
    assert!(!mentions_me("mail@@noah", &me()));
}

/// Mon message n'est « en attente » qu'après le délai, et seulement s'il vise un humain
#[test]
fn waiting_needs_the_delay_and_a_human_recipient() {
    let day = 24 * 3600 * 1000;
    let early = classify(Some(&mine(day)), &me(), NOW);
    assert_eq!(
        early,
        Verdict {
            waiting: false,
            mention: false,
            recheck_at: Some(NOW - day + WAITING_AFTER_MS)
        }
    );
    let due = classify(Some(&mine(WAITING_AFTER_MS)), &me(), NOW);
    assert!(due.waiting);
    let mut bots = mine(3 * day);
    bots.awaits_human = false;
    assert_eq!(
        classify(Some(&bots), &me(), NOW),
        Verdict {
            waiting: false,
            mention: false,
            recheck_at: None
        }
    );
}

/// Un message reçu qui me cite donne `mention` ; un expéditeur automatique, non
#[test]
fn received_mention_is_flagged_unless_automated() {
    let cited = received(1000, "@noah un avis ?");
    assert!(classify(Some(&cited), &me(), NOW).mention);
    let mut bot = received(1000, "@noah un avis ?");
    bot.sender_automated = true;
    assert!(!classify(Some(&bot), &me(), NOW).mention);
}

/// Au-delà de 30 jours, ou sans message vivant, rien n'est signalé ; la fin de ce délai est réévaluée
#[test]
fn horizon_and_missing_message_clear_everything() {
    let old = classify(Some(&mine(LABEL_HORIZON_MS + 1)), &me(), NOW);
    assert_eq!(
        old,
        Verdict {
            waiting: false,
            mention: false,
            recheck_at: None
        }
    );
    assert_eq!(classify(None, &me(), NOW).recheck_at, None);
    let waiting = classify(Some(&mine(WAITING_AFTER_MS)), &me(), NOW);
    assert_eq!(
        waiting.recheck_at,
        Some(NOW - WAITING_AFTER_MS + LABEL_HORIZON_MS)
    );
}
