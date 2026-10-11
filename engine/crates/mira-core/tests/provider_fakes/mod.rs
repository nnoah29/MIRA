//! Faux providers des tests du cœur : un modèle libellés, un modèle dossiers.

pub mod folders;
pub mod labels;
pub mod registries;
pub mod role_inference;

pub use folders::FakeFoldersProvider;
pub use labels::FakeLabelsProvider;
