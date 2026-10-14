#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Option `-v` : comptage des niveaux de bavardage.

use mira_cli::logging::verbosity_from_args;

/// Convertit des littéraux en arguments.
fn args(list: &[&str]) -> Vec<String> {
    list.iter().map(ToString::to_string).collect()
}

/// Sans option, le niveau est 0 ; `-v` donne 1 ; `-vv` ou deux `-v` donnent 2
#[test]
fn counts_verbose_flags() {
    assert_eq!(verbosity_from_args(&args(&["mira", "engine", "info"])), 0);
    assert_eq!(verbosity_from_args(&args(&["mira", "-v", "engine"])), 1);
    assert_eq!(verbosity_from_args(&args(&["mira", "-vv", "engine"])), 2);
    assert_eq!(verbosity_from_args(&args(&["mira", "-v", "-v"])), 2);
}

/// Les autres options et leurs valeurs ne comptent pas
#[test]
fn ignores_other_arguments() {
    assert_eq!(
        verbosity_from_args(&args(&["mira", "--json", "-x", "--profile"])),
        0
    );
}

/// `--verbose` équivaut à `-v`
#[test]
fn long_option_counts_as_one() {
    assert_eq!(verbosity_from_args(&args(&["mira", "--verbose"])), 1);
}
