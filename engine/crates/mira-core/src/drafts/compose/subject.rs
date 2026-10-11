//! Objet d'une réponse ou d'un transfert, sans doublon de préfixe.

/// Objet d'une réponse : `Re: ` ajouté sauf si l'objet en porte déjà un
pub(crate) fn reply_subject(subject: &str) -> String {
    prefixed(subject, &["re"], "Re: ")
}

/// Objet d'un transfert : `Fwd: ` ajouté sauf si l'objet porte déjà `Fwd:` ou `Fw:`
pub(crate) fn forward_subject(subject: &str) -> String {
    prefixed(subject, &["fwd", "fw"], "Fwd: ")
}

/// Ajoute le préfixe sauf si l'objet commence déjà par l'un des mots, suivi de deux-points
fn prefixed(subject: &str, words: &[&str], prefix: &str) -> String {
    let trimmed = subject.trim();
    let lower = trimmed.to_lowercase();
    let already = words.iter().any(|word| {
        lower
            .strip_prefix(word)
            .is_some_and(|rest| rest.trim_start().starts_with(':'))
    });
    if already {
        trimmed.to_string()
    } else {
        format!("{prefix}{trimmed}")
    }
}
