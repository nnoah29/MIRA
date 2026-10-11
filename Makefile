# Point d'entrée prévu par docs/GIT_WORKFLOW.md : `make check` exécute toute la CI locale.
.PHONY: check build test fmt
check:
	./ci.sh
build:
	cd engine && cargo build --workspace
test:
	cd engine && cargo test --workspace
fmt:
	cd engine && cargo fmt --all
