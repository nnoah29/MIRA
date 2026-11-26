#!/usr/bin/env bash
# Intégration continue locale de MIRA : tout doit être vert avant chaque commit.
set -euo pipefail
root="$(cd "$(dirname "$0")" && pwd)"
mkdir -p "$root/target/rapports"
step() { printf '\n== %s\n' "$1"; }

step "Auto-tests des garde-fous"
"$root/tools/test-check-norms"
"$root/tools/test-check-architecture"
"$root/tools/test-check-ci-pins"
"$root/tools/test-check-lock-sync"
"$root/tools/test-generate-sbom"

step "Normes de codage (§11)"
"$root/tools/check-norms" "$root/engine/crates"

step "Architecture (5.3, invariants 1-3)"
"$root/tools/check-architecture" "$root/engine"

step "Motifs interdits"
"$root/tools/check-forbidden-patterns"

step "Chaîne d'outillage de la CI épinglée (constat 10)"
"$root/tools/check-ci-pins"

step "Verrous Cargo du moteur et du pont alignés (constat 12)"
"$root/tools/check-lock-sync"

step "Traductions de l'app (couverture de chaque langue)"
"$root/tools/i18n-audit"

cd "$root/engine"
step "Formatage"
cargo fmt --all --check

step "Clippy (-D warnings)"
cargo clippy --workspace --all-targets --all-features --quiet -- -D warnings

step "Tests"
if cargo nextest --version >/dev/null 2>&1; then
    cargo nextest run --workspace --all-features --no-fail-fast --status-level fail --final-status-level fail --no-tests=pass 2>&1 | tee "$root/target/rapports/tests.log" | tail -n 25
    test "${PIPESTATUS[0]}" -eq 0
else
    cargo test --workspace --all-features --quiet 2>&1 | tee "$root/target/rapports/tests.log" | grep -E "test result|FAILED|panicked|error" | tail -n 40
    test "${PIPESTATUS[0]}" -eq 0
fi

if cargo deny --version >/dev/null 2>&1; then
    step "cargo-deny (licences, interdits, sources ; avis de sécurité hors ligne exclus)"
    cargo deny --log-level error check bans licenses sources
fi

if cargo deny --version >/dev/null 2>&1; then
    step "Avis de sécurité du pont (constat 12 ; ignoré hors ligne)"
    if timeout 15 git ls-remote https://github.com/rustsec/advisory-db HEAD >/dev/null 2>&1; then
        cargo deny --log-level error --manifest-path "$root/bridge/Cargo.toml" --config "$root/engine/deny.toml" check advisories
    else
        echo "réseau indisponible : avis du pont non contrôlés"
    fi
fi

printf '\n== ci.sh : TOUT EST VERT\n'
