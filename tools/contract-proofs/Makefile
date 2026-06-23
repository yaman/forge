.PHONY: all build proofs check test clean extract gen check-clean

ROOT_DIR := $(abspath $(CURDIR))
REPO_ROOT := $(abspath $(CURDIR)/../..)
CONTRACT_TESTS := $(REPO_ROOT)/tools/contract-tests

# Generated artifacts (gitignored): Generated.v, checker.ml, and the
# extracted forge_state_machine.ml{,i}. `make` regenerates all of them in
# dependency order: gen -> extract -> build -> check -> test.
all: gen extract build check test

## Generate theories/Generated.v and extraction/checker.ml from the
## fixture + skill handoff graph (Rust, no Python).
gen:
	cd $(CONTRACT_TESTS) && cargo run --bin gen_coq -- --root $(REPO_ROOT)

## CI stale-check: exit non-zero if Generated.v or checker.ml are stale.
check-clean:
	cd $(CONTRACT_TESTS) && cargo run --bin gen_coq -- --root $(REPO_ROOT) --check

## Regenerate the extracted OCaml module from the Coq proofs.
## First builds the Coq theories (dune, scoped to theories/) so Generated.vo
## and friends exist under _build/default/theories, then runs coqc on
## Extraction.v pointing at that loadpath to produce
## forge_state_machine.ml{,i} into extraction/, then copies into test/.
extract:
	dune build theories/
	coqc -R _build/default/theories Forge.Contracts -Q $(COQ_STDLIB)/Stdlib Stdlib theories/Extraction.v
	cp extraction/forge_state_machine.ml extraction/forge_state_machine.mli test/
	chmod u+w extraction/forge_state_machine.ml extraction/forge_state_machine.mli \
	              test/forge_state_machine.ml test/forge_state_machine.mli

## Build the Coq theories and extracted OCaml checker.
build:
	dune build

## Alias for build (compiles all .v proofs).
proofs: build

## Run the extracted, verified state-machine checker.
check:
	dune exec --root . -- forge-state-machine

## Run the OCaml sanity tests for the extracted checker.
test:
	dune runtest --root .

COQ_STDLIB ?= /opt/homebrew/lib/ocaml/coq/user-contrib

clean:
	dune clean
