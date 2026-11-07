//! Macro qui instancie les scénarios d'erreur du chat pour un chemin donné.

/// Génère les tests de ce thème pour ce `Driver`.
macro_rules! chat_errors_suite {
    ($driver:ty) => {
        /// Une connexion inconnue donne une seule ligne d'erreur.
        #[tokio::test(flavor = "multi_thread")]
        async fn local_error_is_a_single_error_line() {
            $crate::contract::chat_errors::local_error_is_a_single_error_line::<$driver>().await
        }

        /// Un consentement manquant donne une ligne d'erreur avec son code.
        #[tokio::test(flavor = "multi_thread")]
        async fn consent_required_is_an_error_line_with_code() {
            $crate::contract::chat_errors::consent_required_is_an_error_line_with_code::<$driver>()
                .await
        }

        /// Une demande illisible donne une ligne d'erreur sans son texte.
        #[tokio::test(flavor = "multi_thread")]
        async fn invalid_request_json_is_an_error_line() {
            $crate::contract::chat_errors::invalid_request_json_is_an_error_line::<$driver>().await
        }

        /// Sans moteur ouvert, une seule ligne d'erreur.
        #[tokio::test(flavor = "multi_thread")]
        async fn no_engine_open_is_an_error_line() {
            $crate::contract::chat_errors::no_engine_open_is_an_error_line::<$driver>().await
        }
    };
}

pub(crate) use chat_errors_suite;
