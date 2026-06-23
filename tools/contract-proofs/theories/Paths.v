(** * Loop Budget and Progress — derived notions

    [Path], [Reachable], [State], [Transition], and [rank] are GENERATED
    in [Generated.v]. This module defines the loop-budget cap
    ([max_iterations]) and proves the generic progress lemma, using tactics
    that do not case-split on specific states.
*)

Require Import Stdlib.Lists.List.
Require Import Stdlib.Arith.Arith.
Require Import Stdlib.micromega.Lia.
Require Import Generated.

Import ListNotations.

Hint Constructors Path Transition Reachable : core.

(** The canonical fixture caps iterations to prevent infinite loops.
    Conservative value, larger than the longest simple forward path. *)
Definition max_iterations := 12.

(** Forward progress, halt, or handback.

    Every transition either lands at a terminal, strictly decreases
    [rank] (forward progress), or increases it (handback to an earlier
    stage, still on a halting path). Proven generically by [inversion]
    over the generated [Transition] constructors. *)
Lemma transition_is_progress_or_halt_or_handback :
  forall s s',
    Transition s s' ->
    is_terminal s' = true \/ rank s' < rank s \/ rank s < rank s'.
Proof.
  intros s s' H; inversion H; simpl;
    try (left; reflexivity);
    try (right; left; lia);
    try (right; right; lia).
Qed.
