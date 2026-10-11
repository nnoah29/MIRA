//! Fichier conforme : doc, attributs, qualificatifs, chaînes piégeuses.

/// Additionne deux nombres
pub(crate) const fn add(a: u32, b: u32) -> u32 {
    a + b
}

/// Rend une chaîne qui contient de faux commentaires
#[must_use]
pub async fn tricky<'a>(x: &'a str) -> String {
    let s = "// pas un commentaire /* non plus */";
    let r = r#"{ "a": "}" }"#;
    let c = '{';
    let q = '\'';
    format!("{s}{r}{c}{q}{x}")
}

/// Trait avec une signature sans corps et une méthode par défaut
pub trait Shape {
    /// Aire de la forme
    fn area(&self) -> f64;
    /// Périmètre par défaut
    fn perimeter(&self) -> f64 {
        0.0
    }
}

/// Pointeur de fonction en paramètre
extern "C" fn apply(f: fn(u32) -> u32) -> u32 {
    f(1)
}
