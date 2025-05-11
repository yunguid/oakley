# Build and Development Tasks
.PHONY: help build check test clean

help:                           ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

build:                          ## Build the project
	cargo build

check:                          ## Run clippy lints
	cargo clippy --all-targets -- -D warnings

test:                           ## Run unit tests
	cargo test

fix:                            ## Fix warnings
	cargo fix --allow-dirty

clean:                          ## Clean build artifacts
	cargo clean

run:                            ## Run the app
	cargo run

build-full:                     ## Build with all features
	cargo build --features=oakley-cli/full 