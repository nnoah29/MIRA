//! Dessin du formulaire d'ajout de compte : ce que propose le moteur, mot de passe en pastilles, erreurs, OAuth indisponible.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod account_form_fixtures;
mod input_fixtures;

use account_form_fixtures::{
    SECRET, added_reply, fallback_provider, hit, known_provider, login_error, oauth_only_provider,
    ready_state, screen, token_of, type_text,
};
use crossterm::event::KeyCode;
use mira_tui::update::update;

/// Le mot de passe saisi n'apparaît que sous forme de pastilles, une par caractère.
#[test]
fn password_is_rendered_as_bullets_only() {
    let mut state = ready_state(known_provider());
    type_text(&mut state, SECRET);
    let text = screen(&state, 100, 30);
    assert!(text.contains(&"•".repeat(SECRET.chars().count())), "{text}");
    assert!(!text.contains(SECRET), "{text}");
    assert!(!text.contains("s3cret"), "{text}");
}

/// Après l'adresse, le formulaire montre le fournisseur, les serveurs proposés et les méthodes.
#[test]
fn form_shows_the_provider_servers_and_methods() {
    let state = ready_state(known_provider());
    let text = screen(&state, 100, 30);
    for wanted in [
        "Ajouter un compte",
        "Fournisseur : Fournisseur acme",
        "IMAP imap.example.org:993 (TLS)",
        "SMTP smtp.example.org:465 (TLS)",
        "Authentification : mot de passe, mot de passe d'application",
        "‹ mot de passe ›",
        "alice@acme.test",
    ] {
        assert!(text.contains(wanted), "{wanted} absent :\n{text}");
    }
}

/// Le fournisseur de repli montre les quatre champs de serveurs, préremplis, avec le chiffrement.
#[test]
fn fallback_provider_shows_prefilled_server_fields() {
    let state = ready_state(fallback_provider());
    let text = screen(&state, 100, 30);
    for wanted in [
        "Serveur IMAP :",
        "imap.example.org",
        "Port IMAP :",
        "993",
        "Serveur SMTP :",
        "465",
        "TLS",
    ] {
        assert!(text.contains(wanted), "{wanted} absent :\n{text}");
    }
}

/// Un fournisseur qui ne propose qu'OAuth le dit clairement, sans champ de mot de passe.
#[test]
fn oauth_only_provider_says_it_is_unavailable() {
    let state = ready_state(oauth_only_provider());
    let text = screen(&state, 100, 30);
    assert!(
        text.contains("OAuth : non disponible dans cette version"),
        "{text}"
    );
    assert!(!text.contains("Mot de passe :"), "{text}");
}

/// L'état de la requête et l'erreur du moteur sont lisibles dans la fenêtre.
#[test]
fn busy_and_error_lines_are_displayed() {
    let mut state = ready_state(known_provider());
    type_text(&mut state, SECRET);
    let sent = hit(&mut state, KeyCode::Enter);
    let busy = screen(&state, 100, 30);
    assert!(
        busy.contains("Test de la connexion et ajout du compte"),
        "{busy}"
    );
    update(&mut state, added_reply(token_of(&sent), Err(login_error())));
    let failed = screen(&state, 100, 30);
    assert!(
        failed.contains("identifiants refusés par le serveur"),
        "{failed}"
    );
    assert!(
        failed.contains("saisissez de nouveau le mot de passe"),
        "{failed}"
    );
}

/// Avant toute préparation la fenêtre invite à saisir l'adresse et la confirmation de fermeture se lit.
#[test]
fn fresh_form_prompts_for_the_address_and_close_is_confirmed() {
    let mut state = mira_tui::state::AppState::new(100, 30, 1);
    state.account_form.open();
    assert!(screen(&state, 100, 30).contains("Saisissez l'adresse puis Entrée"));
    type_text(&mut state, "a");
    hit(&mut state, KeyCode::Esc);
    assert!(screen(&state, 100, 30).contains("Abandonner la saisie ?"));
}
