//! Rendu d'un jeton dans la syntaxe IMAP d'origine (valeurs brutes des attributs propres au service).

use super::token::Token;

/// Écrit le jeton comme le serveur l'a envoyé : atomes tels quels, chaînes citées, listes entre parenthèses.
pub(crate) fn render_raw(token: &Token) -> String {
    match token {
        Token::Atom(text) => text.clone(),
        Token::Text(bytes) => {
            let text = String::from_utf8_lossy(bytes);
            format!("\"{}\"", text.replace('\\', "\\\\").replace('"', "\\\""))
        }
        Token::List(items) => {
            let inner: Vec<String> = items.iter().map(render_raw).collect();
            format!("({})", inner.join(" "))
        }
    }
}
