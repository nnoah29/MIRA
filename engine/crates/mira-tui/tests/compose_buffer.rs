//! Tampon d'édition maison : curseur en indices de caractères, jamais en octets.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use mira_tui::state::compose::TextBuffer;

/// Taper é, 日, 本 et un émoji avance le curseur d'une position par caractère.
#[test]
fn typing_multibyte_chars_moves_cursor_by_char() {
    let mut buffer = TextBuffer::new("");
    for (typed, c) in ['é', '日', '本', '😀'].into_iter().enumerate() {
        buffer.insert_char(c);
        assert_eq!(buffer.cursor(), (0, typed + 1), "{c}");
    }
    assert_eq!(buffer.text(), "é日本😀");
    buffer.left();
    buffer.left();
    buffer.insert_char('x');
    assert_eq!(buffer.text(), "é日x本😀");
    assert_eq!(buffer.cursor(), (0, 3));
    buffer.backspace();
    buffer.end();
    assert_eq!(buffer.cursor(), (0, 4));
    buffer.delete();
    assert_eq!(buffer.text(), "é日本😀");
}

/// Une séquence d'émoji faite de plusieurs caractères se parcourt caractère par caractère sans panique.
#[test]
fn joined_emoji_sequence_is_walked_char_by_char() {
    let family = "👨\u{200d}👩\u{200d}👧";
    let mut buffer = TextBuffer::new(family);
    buffer.end();
    assert_eq!(buffer.cursor(), (0, 5));
    buffer.backspace();
    assert_eq!(buffer.text().chars().count(), 4);
    buffer.home();
    buffer.delete();
    assert_eq!(buffer.cursor(), (0, 0));
}

/// Retour arrière au tout début ne fait rien, sans panique, y compris tampon vide.
#[test]
fn backspace_at_start_is_noop() {
    let mut empty = TextBuffer::new("");
    empty.backspace();
    empty.delete();
    assert_eq!((empty.text(), empty.cursor()), (String::new(), (0, 0)));
    let mut filled = TextBuffer::new("abc\ndef");
    filled.backspace();
    assert_eq!(
        (filled.text(), filled.cursor()),
        ("abc\ndef".to_string(), (0, 0))
    );
}

/// Haut et bas gardent la colonne souhaitée à travers une ligne plus courte.
#[test]
fn vertical_moves_keep_wanted_column() {
    let mut buffer = TextBuffer::new("abcdef\nab\nabcdef");
    buffer.end();
    buffer.down();
    assert_eq!(buffer.cursor(), (1, 2));
    buffer.down();
    assert_eq!(buffer.cursor(), (2, 6));
    buffer.up();
    buffer.left();
    buffer.up();
    assert_eq!(buffer.cursor(), (0, 1));
    buffer.up();
    assert_eq!(buffer.cursor(), (0, 1));
}

/// Entrée coupe la ligne au curseur, Retour arrière au début d'une ligne la recolle.
#[test]
fn newline_splits_and_backspace_joins() {
    let mut buffer = TextBuffer::new("日本語");
    buffer.right();
    buffer.insert_newline();
    assert_eq!(buffer.text(), "日\n本語");
    assert_eq!(buffer.cursor(), (1, 0));
    buffer.backspace();
    assert_eq!(buffer.text(), "日本語");
    assert_eq!(buffer.cursor(), (0, 1));
    buffer.end();
    buffer.delete();
    assert_eq!(buffer.text(), "日本語");
}

/// Les caractères de contrôle ne s'insèrent pas et les retours chariot sont normalisés au chargement.
#[test]
fn control_chars_are_not_inserted_and_crlf_is_normalized() {
    let mut buffer = TextBuffer::new("a\r\nb\rc");
    assert_eq!(buffer.lines(), ["a", "b", "c"]);
    for c in ['\u{1b}', '\t', '\u{7}', '\r', '\u{85}'] {
        buffer.insert_char(c);
    }
    assert_eq!(buffer.text(), "a\nb\nc");
    assert_eq!(buffer.cursor(), (0, 0));
}
