//! Macro qui instancie les scénarios des lignes de chat et de l'annulation pour un chemin donné.

/// Génère les tests de ce thème pour ce `Driver`.
macro_rules! chat_suite {
    ($driver:ty) => {
        /// La première ligne annonce l'opération.
        #[tokio::test(flavor = "multi_thread")]
        async fn first_line_is_started_with_operation_id() {
            $crate::contract::chat_lines::first_line_is_started_with_operation_id::<$driver>().await
        }

        /// Les lignes suivent l'ordre attendu et le texte se recompose.
        #[tokio::test(flavor = "multi_thread")]
        async fn lines_follow_started_text_usage_done() {
            $crate::contract::chat_lines::lines_follow_started_text_usage_done::<$driver>().await
        }

        /// Sans connexion désignée, la connexion par défaut sert.
        #[tokio::test(flavor = "multi_thread")]
        async fn default_connection_serves_when_none_is_named() {
            $crate::contract::chat_lines::default_connection_serves_when_none_is_named::<$driver>()
                .await
        }

        /// Un consommateur fermé annule l'opération.
        #[tokio::test(flavor = "multi_thread")]
        async fn a_closed_consumer_cancels_the_operation() {
            $crate::contract::chat_cancel::a_closed_consumer_cancels_the_operation::<$driver>()
                .await
        }
    };
}

pub(crate) use chat_suite;
