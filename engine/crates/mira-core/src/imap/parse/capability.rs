//! Capacités annoncées par le serveur (réponse CAPABILITY ou code de réponse).

use super::status::{ResponseCode, untagged_status};

/// Mots d'une liste de capacités, en majuscules.
fn words_of(text: &str) -> Vec<String> {
    text.split_whitespace()
        .map(str::to_ascii_uppercase)
        .collect()
}

/// Capacités portées par un code de réponse `[CAPABILITY ...]`.
pub(crate) fn capabilities_from_code(code: &Option<ResponseCode>) -> Option<Vec<String>> {
    let code = code.as_ref().filter(|code| code.name == "CAPABILITY")?;
    Some(words_of(&code.args))
}

/// Capacités de la première ligne `CAPABILITY ...` ou du premier statut qui en porte le code.
pub(crate) fn capabilities_of(untagged: &[Vec<u8>]) -> Option<Vec<String>> {
    untagged.iter().find_map(|line| {
        let text = String::from_utf8_lossy(line);
        let list = text.strip_prefix("CAPABILITY ").map(words_of);
        list.or_else(|| {
            let (_, code, _) = untagged_status(line)?;
            capabilities_from_code(&code)
        })
    })
}
