//! Journalisation du binaire : WARN par défaut, `-v` pour INFO, `-vv` pour DEBUG, `MIRA_LOG` prioritaire.

use tracing_subscriber::EnvFilter;

/// Indique si l'argument est une série de `-v` (par exemple `-v` ou `-vv`) et combien il en compte.
fn short_verbose_count(argument: &str) -> usize {
    if argument == "--verbose" {
        return 1;
    }
    match argument.strip_prefix('-') {
        Some(letters) if !letters.is_empty() && letters.chars().all(|letter| letter == 'v') => {
            letters.len()
        }
        _ => 0,
    }
}

/// Compte les options `-v` présentes dans les arguments.
pub fn verbosity_from_args(args: &[String]) -> u8 {
    let total: usize = args
        .iter()
        .map(|argument| short_verbose_count(argument))
        .sum();
    u8::try_from(total).unwrap_or(u8::MAX)
}

/// Installe l'abonné `tracing` sur stderr ; sans effet s'il est déjà installé.
pub fn init_logging(verbosity: u8) {
    let level = match verbosity {
        0 => "warn",
        1 => "info",
        _ => "debug",
    };
    let filter = EnvFilter::try_from_env("MIRA_LOG").unwrap_or_else(|_| EnvFilter::new(level));
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .try_init();
}
