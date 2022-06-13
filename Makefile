# rand-utf8 Makefile

.PHONY: all publish test static tools tool_rust tool_fmt tool_readme

SHELL = /usr/bin/env sh

all: test

publish: tools
	git diff --exit-code
	cargo publish
	VER="v$$(grep version Cargo.toml | head -1 | cut -d ' ' -f 3 | cut -d \" -f 2)"; git tag -a $$VER -m $$VER
	git push --tags

static: tools
	cargo fmt -- --check
	cargo clippy
	cargo readme -o README.md
	@if [ "${CI}x" != "x" ]; then git diff --exit-code; fi

test: static tools
	RUST_BACKTRACE=1 cargo build --all-features --all-targets
	RUST_BACKTRACE=1 cargo test --all-features

tools: tool_rust tool_fmt tool_clippy tool_readme

tool_rust:
	@if rustup --version >/dev/null 2>&1; then \
		echo "# Makefile # found rustup, setting override stable"; \
		rustup override set stable; \
	else \
		echo "# Makefile # rustup not found, hopefully we're on stable"; \
	fi;

tool_fmt: tool_rust
	@if ! (cargo fmt --version); \
	then \
		if rustup --version >/dev/null 2>&1; then \
			echo "# Makefile # installing rustfmt with rustup"; \
			rustup component add rustfmt-preview; \
		else \
			echo "# Makefile # rustup not found, cannot install rustfmt"; \
			exit 1; \
		fi; \
	else \
		echo "# Makefile # rustfmt ok"; \
	fi;

tool_clippy: tool_rust
	@if ! (cargo clippy --version); \
	then \
		if rustup --version >/dev/null 2>&1; then \
			echo "# Makefile # installing clippy with rustup"; \
			rustup component add clippy-preview; \
		else \
			echo "# Makefile # rustup not found, cannot install clippy"; \
			exit 1; \
		fi; \
	else \
		echo "# Makefile # clippy ok"; \
	fi;

tool_readme: tool_rust
	@if ! (cargo readme --version); \
	then \
		cargo install cargo-readme; \
	else \
		echo "# Makefile # readme ok"; \
	fi;
