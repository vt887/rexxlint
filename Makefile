SHELL := /bin/sh

CARGO ?= cargo
PORTABLE_DIR ?= portable-c

.PHONY: help all ci clean build release run test lint format fmt-check clippy check \
        portable-build portable-test portable-clean portable-all

help:
	@echo "Available targets:"
	@echo "  make all            - format check + lint + test + build"
	@echo "  make ci             - CI-equivalent checks"
	@echo "  make clean          - clean Rust + portable-c artifacts"
	@echo "  make build          - build debug workspace"
	@echo "  make release        - build release rexxlint"
	@echo "  make run            - run target/debug/rexxlint"
	@echo "  make test           - run all Rust tests"
	@echo "  make lint           - run clippy with -D warnings"
	@echo "  make format         - apply rustfmt"
	@echo "  make fmt-check      - verify rustfmt"
	@echo "  make check          - cargo check workspace"
	@echo "  make portable-build - build C99 fallback"
	@echo "  make portable-test  - test C99 fallback"
	@echo "  make portable-clean - clean C99 artifacts"
	@echo "  make portable-all   - build + test C99 fallback"

all: fmt-check lint test build

ci: fmt-check lint test release portable-all

clean: portable-clean
	$(CARGO) clean

build:
	$(CARGO) build --workspace

release:
	$(CARGO) build --release -p rexx-cli

run:
	target/debug/rexxlint

test:
	$(CARGO) test --workspace

lint: clippy

clippy:
	$(CARGO) clippy --workspace --all-targets -- -D warnings

format:
	$(CARGO) fmt --all

fmt-check:
	$(CARGO) fmt --all -- --check

check:
	$(CARGO) check --workspace

portable-build:
	$(MAKE) -C $(PORTABLE_DIR) all

portable-test:
	$(MAKE) -C $(PORTABLE_DIR) test

portable-clean:
	$(MAKE) -C $(PORTABLE_DIR) clean

portable-all: portable-build portable-test
