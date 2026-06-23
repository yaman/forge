(** * Extraction to OCaml

    This module extracts the verified state-machine definitions to OCaml so
    they can be run as a native checker or called from the Rust CLI.

    The extracted code preserves the finite-state definitions and the
    computable [successors] function, but not the proofs (which are erased
    during extraction).
*)

Require Import Extraction.
Require Import Generated.
Require Import Paths.

Extraction Language OCaml.

(** Inline booleans, lists, and natural numbers as OCaml ints. *)
Extract Inductive bool => "bool" [ "true" "false" ].
Extract Inductive list => "list" [ "[]" "(::)" ].
Extract Inductive nat => "int" [ "0" "succ" ] "(fun f0 fS n -> if n=0 then f0 () else fS (n-1))".

Set Extraction Output Directory "extraction".

Extraction "forge_state_machine.ml"
  State
  all_states
  is_terminal
  terminal_states
  entry_points
  successors
  rank
  max_iterations.
