# Makefile — development conveniences.
#
# Targets:
#   make test         — run the Rust test suite (release)
#   make fresh-test   — regenerate resources/dumps/latex.dump.txt from ambient
#                       TeX Live (via tools/make_formats.sh), then run tests.
#                       Use this after a TeX Live upgrade.
#   make dump         — regenerate resources/dumps/latex.dump.txt only
#   make build        — release build
#   make clean        — cargo clean
#
# Rationale: cargo aliases can't chain shell commands, so the fresh-test
# orchestration lives here instead. CI runs fresh-test so tests always
# execute against the current TeX Live. Locally, developers run plain
# `cargo test` during iteration and `make fresh-test` after a TeX Live
# upgrade (the build.rs loader emits a loud warning on stamp mismatch,
# so you'll notice).

.PHONY: test fresh-test dump build clean

test:
	cargo test --release --tests

dump:
	tools/make_formats.sh

fresh-test: dump test

build:
	cargo build --release

clean:
	cargo clean
