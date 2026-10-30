//! Entropie de Shannon d'une chaîne, en bits par caractère.

use std::collections::HashMap;

/// Entropie de Shannon du texte en bits par caractère : 0 pour une chaîne constante ou vide.
pub fn shannon(text: &str) -> f64 {
    let total = text.chars().count();
    if total == 0 {
        return 0.0;
    }
    let mut counts: HashMap<char, usize> = HashMap::new();
    for character in text.chars() {
        *counts.entry(character).or_insert(0) += 1;
    }
    let total = total as f64;
    let sum: f64 = counts
        .values()
        .map(|count| {
            let probability = *count as f64 / total;
            -probability * probability.log2()
        })
        .sum();
    sum.abs()
}
