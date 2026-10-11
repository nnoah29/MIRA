//! Découpage d'une ligne logique en jetons, tolérant aux réponses mal formées.

use super::token::Token;

const MAX_DEPTH: usize = 32;

/// Position de lecture dans une ligne logique.
#[derive(Debug)]
pub(crate) struct Cursor<'a> {
    pub(super) bytes: &'a [u8],
    pub(super) pos: usize,
}

impl<'a> Cursor<'a> {
    /// Place le curseur au début de la ligne.
    pub(crate) fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, pos: 0 }
    }

    /// Octet courant, sans avancer.
    pub(super) fn peek(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    /// Avance au-delà des espaces.
    fn skip_spaces(&mut self) {
        while self.peek() == Some(b' ') {
            self.pos += 1;
        }
    }

    /// Lit le jeton suivant ; `None` en fin de ligne. Une `)` isolée au premier niveau est ignorée.
    pub(crate) fn next_token(&mut self, depth: usize) -> Option<Token> {
        self.skip_spaces();
        while depth == 0 && self.peek() == Some(b')') {
            self.pos += 1;
            self.skip_spaces();
        }
        match self.peek()? {
            b'(' if depth < MAX_DEPTH => Some(self.list(depth)),
            b'"' => Some(self.quoted()),
            b'{' => self.literal().or_else(|| Some(self.atom())),
            _ => Some(self.atom()),
        }
    }

    /// Lit une liste entre parenthèses, même non fermée.
    fn list(&mut self, depth: usize) -> Token {
        self.pos += 1;
        let mut items = Vec::new();
        loop {
            self.skip_spaces();
            if matches!(self.peek(), None | Some(b')')) {
                break;
            }
            match self.next_token(depth + 1) {
                Some(token) => items.push(token),
                None => break,
            }
        }
        self.pos += 1;
        Token::List(items)
    }
}

/// Découpe toute une ligne logique en jetons de premier niveau.
pub(crate) fn tokenize(line: &[u8]) -> Vec<Token> {
    let mut cursor = Cursor::new(line);
    let mut tokens = Vec::new();
    while let Some(token) = cursor.next_token(0) {
        tokens.push(token);
    }
    tokens
}
