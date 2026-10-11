//! Paramètres de SELECT : CONDSTORE et QRESYNC.

use crate::imap::parser::token::Token;
use crate::imap::response::error::ImapError;

/// Paramètre `(QRESYNC (uidvalidity modseq [uids]))`.
#[derive(Debug, Clone)]
pub(crate) struct QresyncParams {
    pub validity: u32,
    pub modseq: u64,
    pub known: Option<String>,
}

/// Paramètres de SELECT ou EXAMINE.
#[derive(Debug, Default)]
pub(crate) struct SelectParams {
    pub condstore: bool,
    pub qresync: Option<QresyncParams>,
}

/// Lit la liste de paramètres facultative
pub(crate) fn parse_select_params(token: Option<&Token>) -> Result<SelectParams, ImapError> {
    let mut params = SelectParams::default();
    let mut items = token.and_then(Token::list).unwrap_or(&[]).iter();
    while let Some(item) = items.next() {
        match item.keyword().as_deref() {
            Some("CONDSTORE") => params.condstore = true,
            Some("QRESYNC") => {
                let list = items.next().and_then(Token::list);
                params.qresync = Some(parse_qresync(list.unwrap_or(&[]))?);
            }
            _ => {}
        }
    }
    Ok(params)
}

/// Lit les valeurs de QRESYNC
fn parse_qresync(list: &[Token]) -> Result<QresyncParams, ImapError> {
    let number = |index: usize| list.get(index).and_then(Token::text);
    let invalid = || ImapError::bad("Invalid QRESYNC parameters");
    Ok(QresyncParams {
        validity: number(0)
            .and_then(|text| text.parse().ok())
            .ok_or_else(invalid)?,
        modseq: number(1)
            .and_then(|text| text.parse().ok())
            .ok_or_else(invalid)?,
        known: number(2),
    })
}
