#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! L'icône du vrai démon sur un bus de session privé : enregistrement, menu, clic gauche, « Quitter », hôte absent.

mod support;

use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::time::Duration;

use support::fake_watcher::serve_watcher;
use support::private_bus::PrivateBus;
use support::process::RunningDaemon;
use support::process_args::base_args;
use support::tray_probe::{activate, click_entry, menu_entries};

/// Script qui crée `marker` quand l'app est « lancée ».
fn fake_app(dir: &Path, marker: &Path) -> String {
    let script = dir.join("fake-mira");
    std::fs::write(
        &script,
        format!("#!/bin/sh\ntouch '{}'\n", marker.display()),
    )
    .unwrap();
    std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
    script.to_string_lossy().into_owned()
}

/// Démon lancé sur ce bus, en français, avec ce lanceur d'app.
fn spawn_on(bus: &PrivateBus, dir: &Path, app: &str) -> RunningDaemon {
    let args = base_args(dir);
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    let env = [
        ("DBUS_SESSION_BUS_ADDRESS", bus.address.as_str()),
        ("LANG", "fr_FR.UTF-8"),
        ("MIRA_APP_BIN", app),
    ];
    let mut daemon = RunningDaemon::spawn_with_env(&refs, &env);
    daemon.wait_ready(Duration::from_secs(60));
    daemon
}

/// Attend, au plus dix secondes, qu'une icône s'enregistre et rend son service.
async fn registered(items: &std::sync::Mutex<Vec<String>>) -> String {
    for _ in 0..200 {
        if let Some(service) = items.lock().unwrap().first() {
            return service.clone();
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    panic!("aucune icône enregistrée");
}

/// L'icône s'enregistre, affiche trois entrées et le clic gauche lance l'app.
#[tokio::test(flavor = "multi_thread")]
async fn the_tray_registers_shows_its_menu_and_a_left_click_launches_the_app() {
    let bus = PrivateBus::start();
    let (connection, items) = serve_watcher(&bus.address).await;
    let dir = tempfile::tempdir().unwrap();
    let marker = dir.path().join("lancée");
    let app = fake_app(dir.path(), &marker);
    let _daemon = spawn_on(&bus, dir.path(), &app);
    let service = registered(&items).await;
    let entries = menu_entries(&connection, &service).await;
    let labels: Vec<_> = entries.iter().map(|(label, _)| label.as_str()).collect();
    assert_eq!(labels, ["Ouvrir MIRA", "À jour", "Quitter MIRA"]);
    assert_eq!(
        entries.iter().map(|(_, on)| *on).collect::<Vec<_>>(),
        [true, false, true]
    );
    activate(&connection, &service).await;
    tokio::time::sleep(Duration::from_millis(500)).await;
    assert!(marker.exists());
}

/// « Quitter MIRA » arrête le démon avec le code 0.
#[tokio::test(flavor = "multi_thread")]
async fn the_quit_entry_stops_the_daemon() {
    let bus = PrivateBus::start();
    let (connection, items) = serve_watcher(&bus.address).await;
    let dir = tempfile::tempdir().unwrap();
    let app = fake_app(dir.path(), &dir.path().join("inutile"));
    let mut daemon = spawn_on(&bus, dir.path(), &app);
    let service = registered(&items).await;
    click_entry(&connection, &service, 3).await;
    let status = daemon.wait_exit(Duration::from_secs(30));
    assert_eq!(status.code(), Some(0), "{:?}", daemon.output);
}

/// Sans hôte de zone de notification, le démon le dit une fois et continue.
#[tokio::test(flavor = "multi_thread")]
async fn without_a_notification_area_host_the_daemon_carries_on() {
    let bus = PrivateBus::start();
    let dir = tempfile::tempdir().unwrap();
    let app = fake_app(dir.path(), &dir.path().join("inutile"));
    let daemon = spawn_on(&bus, dir.path(), &app);
    let said = daemon
        .output
        .iter()
        .filter(|l| l.contains("icône de zone de notification indisponible"));
    assert_eq!(said.count(), 1, "{:?}", daemon.output);
}
