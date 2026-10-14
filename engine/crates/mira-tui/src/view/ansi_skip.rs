//! Saut des séquences d'échappement du terminal (CSI, OSC, DCS et assimilées).

use std::iter::Peekable;
use std::str::Chars;

/// Saute ce qui suit un ESC déjà lu : séquence CSI, chaîne de contrôle ou court code.
pub fn skip_escape(chars: &mut Peekable<Chars<'_>>) {
    match chars.peek().copied() {
        Some('[') => {
            chars.next();
            skip_csi(chars);
        }
        Some(']' | 'P' | '_' | '^' | 'X') => {
            chars.next();
            skip_string(chars);
        }
        Some(' '..='/') => skip_intermediates(chars),
        Some('\u{1b}') | None => {}
        Some(_) => {
            chars.next();
        }
    }
}

/// Saute une séquence CSI : paramètres, intermédiaires puis octet final.
pub fn skip_csi(chars: &mut Peekable<Chars<'_>>) {
    while chars.next_if(|c| matches!(c, ' '..='?')).is_some() {}
    chars.next_if(|c| matches!(c, '@'..='~'));
}

/// Saute une chaîne de contrôle jusqu'à BEL, ST (ESC puis antislash ou U+009C) ou la fin.
pub fn skip_string(chars: &mut Peekable<Chars<'_>>) {
    while let Some(c) = chars.next() {
        match c {
            '\u{7}' | '\u{9c}' => return,
            '\u{1b}' => return skip_string_end(chars),
            _ => {}
        }
    }
}

/// Après un ESC dans une chaîne : l'antislash la termine, tout autre octet démarre une séquence.
fn skip_string_end(chars: &mut Peekable<Chars<'_>>) {
    if chars.next_if_eq(&'\\').is_none() {
        skip_escape(chars);
    }
}

/// Saute les intermédiaires d'une séquence nF puis son octet final.
fn skip_intermediates(chars: &mut Peekable<Chars<'_>>) {
    while chars.next_if(|c| matches!(c, ' '..='/')).is_some() {}
    chars.next_if(|c| matches!(c, '0'..='~'));
}
