#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Lecture du secret sur l'entrée standard : erreurs propagées, taille bornée, UTF-8 exigé (audit 18).

use std::io::{self, Cursor, Read};

use mira_assembly::ports::SequentialIds;
use mira_cli::argument_context::ArgumentContext;

/// Lecteur qui livre quelques octets puis échoue
struct BrokenInput(bool);

impl Read for BrokenInput {
    /// Livre `a` au premier appel puis simule une entrée coupée
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        if self.0 {
            return Err(io::Error::other("entrée coupée"));
        }
        self.0 = true;
        buffer[0] = b'a';
        Ok(1)
    }
}

/// Lit le secret et rend le code d'erreur, ou `None` si le secret est accepté
fn outcome(stdin: &mut dyn Read) -> Option<String> {
    let ids = SequentialIds::new();
    let mut context = ArgumentContext { stdin, ids: &ids };
    context.read_secret().err().map(|error| error.code)
}

/// Une erreur de lecture n'est pas prise pour une fin de fichier
#[test]
fn regression_read_error_is_propagated() {
    let code = outcome(&mut BrokenInput(false));
    assert_eq!(code.as_deref(), Some("invalid_input.parameter"));
}

/// Plus de 4096 octets sans fin de ligne : refus, pas de troncature silencieuse
#[test]
fn regression_oversized_secret_is_refused_not_truncated() {
    let mut long = Cursor::new(vec![b'a'; 5000]);
    assert_eq!(
        outcome(&mut long).as_deref(),
        Some("invalid_input.parameter")
    );
    let mut with_newline = Cursor::new([vec![b'a'; 4097], b"\n".to_vec()].concat());
    assert!(outcome(&mut with_newline).is_some());
    let mut exact = Cursor::new([vec![b'a'; 4096], b"\n".to_vec()].concat());
    assert!(outcome(&mut exact).is_none());
}

/// Un secret qui n'est pas de l'UTF-8 valide est refusé au lieu d'être déformé
#[test]
fn regression_non_utf8_secret_is_refused() {
    let mut invalid = Cursor::new(vec![b'a', 0xFF, b'b', b'\n']);
    assert_eq!(
        outcome(&mut invalid).as_deref(),
        Some("invalid_input.parameter")
    );
}

/// Seule la première ligne compte, sans le retour chariot final
#[test]
fn first_line_is_the_secret_and_carriage_return_is_dropped() {
    let mut input = Cursor::new(b"mot-de-passe\r\nsuite\n".to_vec());
    let ids = SequentialIds::new();
    let mut context = ArgumentContext {
        stdin: &mut input,
        ids: &ids,
    };
    let secret = context.read_secret().unwrap();
    assert_eq!(secret.into_secret_value().expose(), "mot-de-passe");
}
