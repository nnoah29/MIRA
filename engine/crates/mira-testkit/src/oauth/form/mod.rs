//! Formulaires `application/x-www-form-urlencoded` et chaînes de requête.

mod percent;

pub use percent::{decode, encode};

/// Paires nom et valeur décodées d'un formulaire ou d'une chaîne de requête.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Form {
    pairs: Vec<(String, String)>,
}

impl Form {
    /// Analyse un texte `a=1&b=2` en décodant les noms et les valeurs
    pub fn parse(text: &str) -> Self {
        let pairs = text
            .split('&')
            .filter(|pair| !pair.is_empty())
            .map(|pair| {
                let (name, value) = pair.split_once('=').unwrap_or((pair, ""));
                (decode(name), decode(value))
            })
            .collect();
        Self { pairs }
    }

    /// Première valeur associée à ce nom
    pub fn get(&self, name: &str) -> Option<&str> {
        let found = self.pairs.iter().find(|(key, _)| key == name);
        found.map(|(_, value)| value.as_str())
    }
}

/// Assemble des paires en chaîne de requête encodée
pub fn build_query(pairs: &[(&str, &str)]) -> String {
    let encoded: Vec<String> = pairs
        .iter()
        .map(|(name, value)| format!("{}={}", encode(name), encode(value)))
        .collect();
    encoded.join("&")
}
