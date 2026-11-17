//! Navigateur simulé : lit l'adresse d'autorisation puis appelle la redirection comme le ferait le service.

use std::collections::HashMap;

/// Paramètres de l'adresse d'autorisation, décodés
pub fn query_of(authorize_url: &str) -> HashMap<String, String> {
    let url = reqwest::Url::parse(authorize_url).unwrap();
    url.query_pairs()
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect()
}

/// Rappelle `callback_url` avec la requête donnée et rend le statut HTTP
pub async fn redirect(callback_url: &str, query: &str) -> u16 {
    let client = reqwest::Client::builder().no_proxy().build().unwrap();
    let answer = client.get(format!("{callback_url}?{query}")).send().await;
    answer.unwrap().status().as_u16()
}

/// Approuve : redirection avec `code` et le `state` reçu dans l'adresse d'autorisation
pub async fn approve(authorize_url: &str, code: &str) -> u16 {
    let query = query_of(authorize_url);
    let state = query.get("state").unwrap();
    redirect(
        &query["callback_url"],
        &format!("code={code}&state={state}"),
    )
    .await
}
