open Forge_state_machine

let () =
  (* Invariants that hold by construction and do not depend on the exact
     rank numbering the generator derives from canonical_states ordering. *)
  assert (is_terminal S_done);
  assert (not (is_terminal S_in_dev));
  assert (successors S_done = []);
  assert (max_iterations = 12);
  (* A forward edge exists in the generated graph. *)
  assert (List.length (successors S_in_qa) > 0);
  Printf.printf "Extracted checker sanity tests passed.\n"
