SHELL := /bin/bash -O globstar

# built-in targets
.PHONY: help install run build build-dev test build-docs serve-docs benchmarks open-benchmarks format lint megalint typecheck check pre-commit all
.DEFAULT_GOAL := help

# targets
help: ## this help
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)

install: ## Install dependencies
	@echo "Use rustup"

run:  ## Run the example
	RUST_LOG=debug RUST_BACKTRACE=1 cargo run --example simple-example

build: ## Build the library
	cargo build --release

build-dev: ## Build the library in dev-mode (faster to build, slower to run)
	cargo build

test: ## Run tests (and doc examples as tests!)
	cargo test -- --test-threads=1 $(FILTER) $(ARGS)

build-docs: ## Build the docs
	cargo doc

serve-docs: ## Serve the docs (links are wroken when opening the docs using file://)
	@set -me; \
	cd target/doc ; \
	python -m http.server & \
   	xdg-open http://localhost:8000/miimetiq_amqp ; \
	fg 1

benchmarks: ## Run benchmarks and generate HTML report
	cargo bench $(FILTER) -- $(ARGS)

open-benchmarks: ## Open benchmakr's HTML report in browser
	xdg-open target/criterion/report/index.html


format:  ## Format source code
	cargo +nightly fmt

lint:  ## Run the linter(s)
	touch src/**/*.rs && cargo clippy

megalint:  ## Run the linter with the maximum strictness possible
	touch src/**/*.rs && cargo clippy -- -D clippy::all -D clippy::pedantic

typecheck:  ## Run type checker
	cargo check

check: typecheck ## Alias of typecheck

pre-commit: format typecheck lint  ## Run this command before every commit

all: install pre-commit  ## I'm feeling lucky
