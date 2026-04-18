.PHONY: all check fmt lint test build clean dry-run publish help

CARGO := cargo
CRATE_NAME := watch-and-commit
BIN_NAME := wac
VERSION := $(shell grep '^version' Cargo.toml | sed 's/version = "\(.*\)"/\1/' | tr -d ' ')

all: help

## Verify all required tools are installed
check-tools:
	@command -v cargo >/dev/null 2>&1 || { echo "cargo is not installed"; exit 1; }
	@command -v git >/dev/null 2>&1 || { echo "git is not installed"; exit 1; }
	@echo "All required tools found"

## Format code
fmt:
	$(CARGO) fmt --all

## Run clippy lints
lint:
	$(CARGO) clippy -- -D warnings

## Run tests
test:
	$(CARGO) test

## Build release binary
build:
	$(CARGO) build --release

## Run all checks (fmt, lint, test, build)
check: fmt lint test build
	@echo "All checks passed for v$(VERSION)"

## Verify Cargo.toml is valid and crate is ready
verify:
	$(CARGO) verify-project
	@echo "Verified: $(CRATE_NAME) v$(VERSION)"

## Dry run publish (no upload)
dry-run: check verify
	$(CARGO) publish --dry-run
	@echo "Dry run successful for $(CRATE_NAME) v$(VERSION)"

## Tag the current version in git
tag:
	@if git rev-parse "v$(VERSION)" >/dev/null 2>&1; then \
		echo "Tag v$(VERSION) already exists"; \
		exit 1; \
	fi
	git tag -a "v$(VERSION)" -m "Release v$(VERSION)"
	git push origin "v$(VERSION)"
	@echo "Tagged and pushed v$(VERSION)"

## Publish to crates.io
publish: dry-run tag
	$(CARGO) publish
	@echo "Published $(CRATE_NAME) v$(VERSION) to crates.io"

## Remove build artifacts
clean:
	$(CARGO) clean

help:
	@echo "Usage: make [target]"
	@echo ""
	@echo "  $(CRATE_NAME) v$(VERSION)"
	@echo ""
	@echo "Targets:"
	@echo "  fmt        Format source code"
	@echo "  lint       Run clippy lints"
	@echo "  test       Run test suite"
	@echo "  build      Build release binary"
	@echo "  check      Run fmt + lint + test + build"
	@echo "  verify     Validate Cargo.toml"
	@echo "  dry-run    Simulate publishing without uploading"
	@echo "  tag        Create and push git version tag"
	@echo "  publish    Full publish pipeline to crates.io"
	@echo "  clean      Remove build artifacts"