//! Macros qui instancient les scénarios de contrat pour un chemin donné.

/// Génère les tests du scénario courrier pour ce `Driver`.
macro_rules! mail_suite {
    ($driver:ty) => {
        /// Comptes, boîtes, messages et corps arrivent en JSON du contrat, sans secret.
        #[tokio::test(flavor = "multi_thread")]
        async fn the_engine_serves_real_values_as_json() {
            $crate::contract::mail_values::the_engine_serves_real_values_as_json::<$driver>().await
        }

        /// Une requête illisible ou un moteur fermé donnent une enveloppe d'erreur, jamais un plantage.
        #[tokio::test(flavor = "multi_thread")]
        async fn errors_are_reported_as_envelopes() {
            $crate::contract::errors::errors_are_reported_as_envelopes::<$driver>().await
        }
    };
}

pub(crate) use mail_suite;
