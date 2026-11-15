//! Vecteurs de souvenirs en octets : f32 little-endian, dimension contrôlée au décodage.

/// Écrit les valeurs en octets f32 little-endian
pub fn encode_vector(values: &[f32]) -> Vec<u8> {
    values.iter().flat_map(|v| v.to_le_bytes()).collect()
}

/// Relit un vecteur ; rend `None` si la longueur ne vaut pas quatre fois la dimension
pub fn decode_vector(bytes: &[u8], dim: u32) -> Option<Vec<f32>> {
    let expected = usize::try_from(dim).ok()?.checked_mul(4)?;
    if bytes.len() != expected {
        return None;
    }
    Some(
        bytes
            .chunks_exact(4)
            .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
            .collect(),
    )
}
