(** * Forge Canonical Delivery Transitions — lemmas

    The [Transition] inductive and [successors] function are GENERATED from
    the handoff graph by [Generated.v]. This module proves they agree:
    [successors s] is exactly the post-image of [s] under [Transition].

    The proofs use generic tactics ([inversion] + [auto] with the
    [Hint Constructors Transition] database) so they remain valid as the
    generator adds or removes edges — there is no per-edge case analysis
    to drift.
*)

Require Import Stdlib.Lists.List.
Require Import Generated.
Import ListNotations.

Hint Constructors Transition : core.

(** If [Transition s s'] then [s'] is in [successors s].

    [inversion H] case-splits on which [Transition] constructor proves
    [s s']; each branch fixes [s] and [s'] to the constructor's endpoints,
    and [simpl] reduces [successors s] to the literal list the generator
    emitted for that source. [auto] then proves membership. *)
Lemma successors_complete :
  forall s s', Transition s s' -> In s' (successors s).
Proof.
  apply successors_complete_generated.
Qed.

Lemma successors_sound :
  forall s s', In s' (successors s) -> Transition s s'.
Proof.
  apply successors_sound_generated.
Qed.

(** Terminal states have no successors. *)
Lemma terminal_no_successors :
  forall s, is_terminal s = true -> successors s = [].
Proof.
  intros s H. destruct s; simpl in *; auto; discriminate.
Qed.

(** No transition leaves a terminal state. *)
Lemma terminal_no_outgoing :
  forall s s', is_terminal s = true -> ~ Transition s s'.
Proof.
  intros s s' Hterm Htrans.
  pose proof (successors_complete _ _ Htrans) as Hin.
  pose proof (terminal_no_successors _ Hterm) as Heq.
  rewrite Heq in Hin. inversion Hin.
Qed.

(** Every non-terminal state has at least one outgoing transition. *)
Lemma non_terminal_has_successor :
  forall s, is_terminal s = false -> exists s', Transition s s'.
Proof.
  intros s Hne. destruct s; simpl in *; try discriminate.
  all: eauto.
Qed.
