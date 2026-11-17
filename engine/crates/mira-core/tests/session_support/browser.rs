//! Navigateur simulé : lit l'adresse d'autorisation puis rappelle la redirection comme le ferait le service.

use std::collections::HashMap;

/// Paramètres de l'adresse d'autorisation, décodés
pub fn query_of(authorize_url: &str) -> HashMap<String, String> {
    let url = reqwest::Url::parse(authorize_url).unwrap();
    url.query_pairs()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

/// Appelle la redirection avec la requête donnée et rend le statut HTTP
pub async fn redirect(redirect_uri: &str, query: &str) -> u16 {
    let client = reqwest::Client::builder().no_proxy().build().unwrap();
    let answer = client.get(format!("{redirect_uri}?{query}")).send().await;
    answer.unwrap().status().as_u16()
}
