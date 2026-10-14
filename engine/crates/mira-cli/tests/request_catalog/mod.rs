//! Catalogue des requêtes du contrat et des lignes de commande qui les produisent.

pub mod examples;
pub mod names_accounts;
pub mod names_simple;
pub mod names_write;

use std::io::Cursor;

use clap::Parser;
use mira_assembly::api::Request;
use mira_assembly::ports::SequentialIds;
use mira_cli::argument_context::ArgumentContext;
use mira_cli::arguments::Cli;
use mira_cli::commands::request_builder::build_request;

use names_accounts::{account_name, auth_name, identity_name};
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
        Request::Mailbox(_)
        | Request::Message(_)
        | Request::Thread(_)
        | Request::Search(_)
        | Request::Maintenance(_)
        | Request::Sync(_) => "read.cli_pending",
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

/// Requête produite par une ligne de commande (sans le nom du programme), secret lu sur stdin.
pub fn request_of_line(line: &[&str]) -> Request {
    let mut args = vec!["mira"];
    args.extend_from_slice(line);
    let cli = Cli::try_parse_from(args).unwrap();
    let mut stdin = Cursor::new(b"un-secret\n".to_vec());
    let ids = SequentialIds::new();
    let mut context = ArgumentContext {
        stdin: &mut stdin,
        ids: &ids,
    };
    build_request(&cli.command, &mut context).unwrap()
}
