//! Lecture des atomes, chaînes entre guillemets et littéraux.

use super::token::Token;
use super::tokenizer::Cursor;

impl Cursor<'_> {
    /// Avance jusqu'au crochet fermant, crochet compris.
    fn skip_bracket(&mut self) {
        while let Some(byte) = self.peek() {
            self.pos += 1;
            if byte == b']' {
                break;
            }
        }
    }

    /// Lit un atome ; une section entre crochets en fait partie, espaces compris.
    pub(super) fn atom(&mut self) -> Token {
        let start = self.pos;
        while let Some(byte) = self.peek() {
            match byte {
                b' ' | b'(' | b')' => break,
                b'[' => self.skip_bracket(),
                _ => self.pos += 1,
            }
        }
        if self.pos == start {
            self.pos += 1;
        }
        Token::Atom(String::from_utf8_lossy(&self.bytes[start..self.pos]).into_owned())
    }

    /// Lit une chaîne entre guillemets en décodant les échappements.
    pub(super) fn quoted(&mut self) -> Token {
        self.pos += 1;
        let mut out = Vec::new();
        while let Some(byte) = self.peek() {
            self.pos += 1;
            match byte {
                b'"' => break,
                b'\\' => {
                    if let Some(next) = self.peek() {
                        out.push(next);
                        self.pos += 1;
                    }
                }
                other => out.push(other),
            }
        }
        Token::Text(out)
    }

    /// Lit un littéral `{n}` CRLF suivi de n octets ; `None` si la forme est invalide.
    pub(super) fn literal(&mut self) -> Option<Token> {
        let rest = &self.bytes[self.pos..];
        let close = rest.iter().position(|byte| *byte == b'}')?;
        let size: usize = std::str::from_utf8(&rest[1..close]).ok()?.parse().ok()?;
        let after = self.pos + close + 1;
        let body = self.bytes[after..].strip_prefix(b"\r\n")?;
        let taken = size.min(body.len());
        self.pos = after + 2 + taken;
        Some(Token::Text(body[..taken].to_vec()))
    }
}
