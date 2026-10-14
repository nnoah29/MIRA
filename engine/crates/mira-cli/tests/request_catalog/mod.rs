//! Catalogue des requêtes du contrat et des lignes de commande qui les produisent.

pub mod examples;
pub mod examples_read;
pub mod examples_write;
pub mod lines_read_write;
pub mod names_accounts;
pub mod names_read;
pub mod names_simple;
pub mod names_write;

use std::io::Cursor;

use clap::Parser;
use mira_assembly::api::Request;
use mira_assembly::mira_vocabulary::MiraError;
use mira_assembly::ports::SequentialIds;
use mira_cli::argument_context::ArgumentContext;
use mira_cli::argument_error::invalid_argument;
use mira_cli::arguments::Cli;
use mira_cli::commands::request_builder::build_request;

use names_accounts::{account_name, auth_name, identity_name};
use names_read::{mailbox_name, maintenance_name, message_name, search_name, thread_name};
use names_simple::{engine_name, event_name, operation_name, provider_name};
use names_write::{action_name, compose_name, queue_name};

/// Nom `domaine.opération` d'une requête, par `match` exhaustif sur tous les domaines.
pub fn variant_name(request: &Request) -> &'static str {
    match request {
        Request::Engine(inner) => engine_name(inner),
        Request::Provider(inner) => provider_name(inner),
        Request::Account(inner) => account_name(inner),
        Request::Identity(inner) => identity_name(inner),
        Request::Auth(inner) => auth_name(inner),
        Request::Operation(inner) => operation_name(inner),
        Request::Event(inner) => event_name(inner),
        Request::Mailbox(inner) => mailbox_name(inner),
        Request::Message(inner) => message_name(inner),
        Request::Thread(inner) => thread_name(inner),
        Request::Search(inner) => search_name(inner),
        Request::Maintenance(inner) => maintenance_name(inner),
        Request::Action(inner) => action_name(inner),
        Request::Compose(inner) => compose_name(inner),
        Request::Queue(inner) => queue_name(inner),
    }
}

/// Nom `domaine.opération` lu dans la forme sérialisée du contrat.
pub fn serialized_name(request: &Request) -> String {
    let value = serde_json::to_value(request).unwrap();
    format!(
        "{}.{}",
        value["domain"].as_str().unwrap(),
        value["request"]["op"].as_str().unwrap()
    )
}

/// Requête produite par une ligne de commande, ou l'erreur d'analyse ou de construction.
pub fn try_request_of_line(line: &[&str]) -> Result<Request, MiraError> {
    let mut args = vec!["mira"];
    args.extend_from_slice(line);
    let cli = Cli::try_parse_from(args).map_err(|error| invalid_argument(&error.to_string()))?;
    let mut stdin = Cursor::new(b"un-secret\n".to_vec());
    let ids = SequentialIds::new();
    let mut context = ArgumentContext {
        stdin: &mut stdin,
        ids: &ids,
    };
    build_request(&cli.command, &mut context)
}

/// Requête produite par une ligne de commande (sans le nom du programme), secret lu sur stdin.
pub fn request_of_line(line: &[&str]) -> Request {
    try_request_of_line(line).unwrap()
}

/// Requête d'une ligne de commande, sous sa forme JSON du contrat.
pub fn json_of_line(line: &[&str]) -> serde_json::Value {
    serde_json::to_value(request_of_line(line)).unwrap()
}
