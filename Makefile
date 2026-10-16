q# MIRA — point d'entrée unique : moteur Rust, CLI, TUI, pont FFI et application Flutter.
# `make` ou `make help` liste les cibles. Variables : MODE, BUILD_MODE, MIRA_HOME, DEVICE, JOBS, FRB_CODEGEN.

SHELL        := /bin/bash
.DEFAULT_GOAL := help

ENGINE       := engine
BRIDGE       := bridge
APP          := app
RAPPORTS     := target/rapports
JOBS         ?= 4
DEVICE       ?= linux
# Profil du moteur utilisé par les clients (dossier de données) ; vide = profil par défaut du système.
MIRA_HOME    ?=
# Mode de `make run` : debug | profile | release (la fluidité se juge en profile ou release).
MODE         ?= profile
# Mode de `make app-build`.
BUILD_MODE   ?= release
# Codegen flutter_rust_bridge (version 2.11.1 exigée, voir bridge/README.md).
FRB_CODEGEN  ?= flutter_rust_bridge_codegen
FRB_VERSION  := 2.11.1
LIB          := $(BRIDGE)/target/release/libmira_bridge.so

export CARGO_BUILD_JOBS := $(JOBS)
ifneq ($(MIRA_HOME),)
export MIRA_HOME
endif

.PHONY: help
help: ## Affiche cette aide
	@awk 'BEGIN{FS=":.*## "} /^[a-zA-Z0-9_-]+:.*## /{printf "  \033[1m%-16s\033[0m %s\n",$$1,$$2} /^##@/{printf "\n%s\n",substr($$0,5)}' $(MAKEFILE_LIST)

##@ Lancer
.PHONY: run run-debug run-release demo tui cli
run: bridge app-deps ## Lance l'application Flutter sur le moteur réel (MODE=profile|debug|release)
	cd $(APP) && flutter run --$(MODE) -d $(DEVICE)

run-debug: ## Lance l'application en debug (hot reload, lent)
	$(MAKE) run MODE=debug

run-release: ## Lance l'application en release
	$(MAKE) run MODE=release

demo: app-deps ## Lance l'application sur les données de démonstration (sans moteur)
	cd $(APP) && MIRA_DEMO=1 flutter run --$(MODE) -d $(DEVICE)

tui: ## Lance le client plein écran (TUI) ; ferme-le avant l'application : le profil est verrouillé
	cd $(ENGINE) && cargo run --release -p mira-tui

cli: ## Lance le CLI : make cli ARGS="account list --json"
	cd $(ENGINE) && cargo run --release -p mira-cli -- $(ARGS)

##@ Construire
.PHONY: build engine bridge app-deps app-build install-linux
build: engine bridge ## Construit le moteur, le CLI, le TUI et le pont

engine: ## Construit tout le workspace du moteur (release)
	cd $(ENGINE) && cargo build --workspace --release

bridge: ## Construit la bibliothèque FFI (bridge/target/release/libmira_bridge.so)
	cd $(BRIDGE) && cargo build --release

app-deps: ## Récupère les dépendances Flutter
	cd $(APP) && flutter pub get

app-build: bridge app-deps ## Construit l'application Linux dans app/build (BUILD_MODE=release|profile|debug)
	cd $(APP) && flutter build linux --$(BUILD_MODE)
	@echo "Bibliothèque du moteur à placer à côté de l'exécutable, ou MIRA_BRIDGE_LIB=$(abspath $(LIB))"

##@ Générer
.PHONY: codegen codegen-check
codegen: ## Régénère les liaisons Dart/Rust (flutter_rust_bridge 2.11.1)
	@$(MAKE) --no-print-directory codegen-check
	cd $(APP) && RUSTC_BOOTSTRAP=1 $(FRB_CODEGEN) generate

codegen-check: ## Vérifie la version du codegen
	@v=$$($(FRB_CODEGEN) --version 2>/dev/null | awk '{print $$2}'); \
	 if [ "$$v" != "$(FRB_VERSION)" ]; then \
	   echo "flutter_rust_bridge_codegen $(FRB_VERSION) requis (trouvé : $${v:-aucun})."; \
	   echo "  cargo install flutter_rust_bridge_codegen --version $(FRB_VERSION) --locked --root ~/.local/frb"; \
	   echo "  puis : make codegen FRB_CODEGEN=~/.local/frb/bin/flutter_rust_bridge_codegen"; exit 1; fi

##@ Vérifier
.PHONY: check check-all test test-all fmt lint norms bridge-test app-test app-analyze
check: ## Toute la CI du moteur (formatage, clippy, tests, architecture, normes)
	./ci.sh

check-all: check bridge-test app-analyze app-test ## CI du moteur + pont + application

test: ## Tests du moteur
	cd $(ENGINE) && cargo test --workspace

test-all: test bridge-test app-test ## Tests du moteur, du pont et de l'application

fmt: ## Formate le moteur, le pont et l'application
	cd $(ENGINE) && cargo fmt --all
	cd $(BRIDGE) && cargo fmt
	cd $(APP) && dart format lib test

lint: ## Clippy du moteur et du pont, analyse Dart
	cd $(ENGINE) && cargo clippy --workspace --all-targets --all-features -- -D warnings
	cd $(BRIDGE) && cargo clippy --all-targets -- -D warnings
	cd $(APP) && flutter analyze

norms: ## Normes de codage du moteur et du code écrit à la main du pont (hors code généré)
	tools/check-norms $(ENGINE)/crates
	tools/check-norms $$(find $(BRIDGE)/src -name '*.rs' ! -name frb_generated.rs)

bridge-test: ## Tests du pont (JSON → moteur → faux serveur IMAP local)
	cd $(BRIDGE) && cargo test

app-analyze: ## Analyse statique Dart
	cd $(APP) && flutter analyze

app-test: bridge ## Tests de l'application (dont le moteur réel par FFI, sur un profil jetable)
	cd $(APP) && MIRA_HOME=$$(mktemp -d) flutter test -j 2

##@ Entretien
.PHONY: setup clean clean-app clean-all
setup: ## Vérifie les outils nécessaires (cargo, flutter, codegen)
	@for t in cargo flutter dart python3; do \
	   command -v $$t >/dev/null && printf "  ok  %s\n" "$$t" || { printf "  MANQUANT  %s\n" "$$t"; exit 1; }; done
	@cd $(ENGINE) && rustc --version
	@flutter --version | head -1
	@$(MAKE) --no-print-directory codegen-check >/dev/null 2>&1 \
	   && echo "  ok  codegen $(FRB_VERSION)" || echo "  (codegen $(FRB_VERSION) absent : voir 'make codegen-check')"

clean: ## Supprime les builds Rust (moteur et pont)
	cd $(ENGINE) && cargo clean
	cd $(BRIDGE) && cargo clean

clean-app: ## Supprime les builds Flutter (à ne pas lancer pendant un flutter run)
	cd $(APP) && flutter clean

clean-all: clean clean-app ## Nettoie tout
