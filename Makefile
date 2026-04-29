# ccze (Rust port) Makefile.
# Run from rust/. Targets are self-documenting — `make help`.

.PHONY: help build release test test-release clean lint fmt fmt-check \
        run cssdump list-plugins demo \
        docker-image baseline baselines

.DEFAULT_GOAL := help

help: ## Show this help.
	@awk 'BEGIN { FS = ":.*##"; printf "Usage:\n  make <target>\n\nTargets:\n" } \
	     /^[a-zA-Z_-]+:.*?##/ { printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2 }' $(MAKEFILE_LIST)

# ---- build & test --------------------------------------------------------

build: ## Debug build (target/debug/ccze).
	cargo build

release: ## Release build (target/release/ccze, ~2.9 MB, optimised).
	cargo build --release

test: ## Run all tests (unit + snapshot, debug profile).
	cargo test

test-release: ## Run all tests under the release profile.
	cargo test --release

clean: ## cargo clean — wipe target/.
	cargo clean

lint: ## Run clippy (informational; warnings are not errors here).
	cargo clippy --all-targets

fmt: ## rustfmt across the crate.
	cargo fmt

fmt-check: ## Verify formatting without rewriting (CI-style).
	cargo fmt --check

# ---- one-shot binary invocations -----------------------------------------

run: build ## Pipe stdin through ccze in ANSI mode (e.g. `tail -f log | make run`).
	./target/debug/ccze -A -F /dev/null

cssdump: build ## Print the embedded CSS class block (alias for `ccze-cssdump`).
	./target/debug/ccze --cssdump

list-plugins: build ## List every registered plugin (Name | Type | Description).
	./target/debug/ccze -l

demo: build ## Render a synthetic syslog line in ANSI to stdout.
	@printf 'Apr 28 17:00:00 host sshd[1234]: Accepted publickey for malcolm from 1.2.3.4 port 22 ssh2\n' \
	  | ./target/debug/ccze -A -F /dev/null -p syslog

# ---- C reference Docker image -------------------------------------------

docker-image: ## Build the ccze:reference image used to mint snapshot baselines.
	./scripts/build-c-ref.sh

baseline: ## Regenerate one baseline. Args: NAME=snap-<plugin> PLUGINS=<csv>
	@if [ -z "$(NAME)" ] || [ -z "$(PLUGINS)" ]; then \
	  echo "usage: make baseline NAME=snap-<plugin> PLUGINS=<plugin1>[,<plugin2>...]"; \
	  exit 64; \
	fi
	./scripts/generate-baseline.sh "$(NAME)" "$(PLUGINS)"

baselines: ## Regenerate every snap-* baseline (16 plugins).
	./scripts/generate-all-baselines.sh
