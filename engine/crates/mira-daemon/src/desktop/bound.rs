//! Bornage du texte d'une notification : l'objet d'un mail vient de l'expéditeur, il ne doit pas déborder.

const MAX_BODY_CHARS: usize = 400;

/// Les 400 premiers caractères du corps.
pub fn bound_body(body: &str) -> String {
    body.chars().take(MAX_BODY_CHARS).collect()
}
