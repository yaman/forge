//! `gen_coq` — Generate the Coq formalization of the Forge Perfect Loop
//! state machine directly from the parsed fixture and skill handoff graph.
//!
//! This closes the "self-proving" gap: instead of hand-transcribing the
//! canonical states, terminal states, entry points, and transitions into
//! Coq (which drifts the moment a skill file changes), we emit
//! `theories/Generated.v` from the same `Repo::from_root` parse that the
//! contract tests use. Coq then re-checks every state and edge against the
//! generated model.
//!
//! ## A1 reachability proof
//!
//! For each entry point, the generator runs BFS in Rust to find a shortest
//! path to `done`, then emits that path verbatim as a `Reachable entry done`
//! witness. Coq verifies the witness edge-by-edge against `Transition`
//! constructors in `Generated.v`. If the fixture changes and the path no
//! longer exists, the witness fails to type-check and the build breaks —
//! drift is caught.
//!
//! ## Usage
//!
//!     cargo run -p forge-contract-tests --bin gen_coq [--root <path>] [--check]
//!
//! `--check` writes nothing and exits non-zero if any emitted file would
//! change (CI stale-check). Without it, writes `theories/Generated.v` and
//! `extraction/checker.ml`.
//!
//! ## Emitted artifacts (both gitignored — regenerate with `make gen`)
//!
//! - `tools/contract-proofs/theories/Generated.v` — the proof model.
//! - `tools/contract-proofs/extraction/checker.ml` — the runtime checker,
//!   which iterates the extracted `all_states` / `entry_points` rather than
//!   hard-coding the state list. (The checked-in `forge_state_machine.ml`
//!   and the `string_of_state` match in `checker.ml` are the only places
//!   state constructor names appear, and both are generated.)

use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use forge_contract_tests::types::{Repo, Transition};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    let mut root: Option<PathBuf> = None;
    let mut check = false;
    let mut iter = args.iter().skip(1);
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--root" => {
                root = iter.next().map(PathBuf::from);
            }
            "--check" => check = true,
            "-h" | "--help" => {
                print_help();
                return ExitCode::SUCCESS;
            }
            other => {
                eprintln!("gen_coq: unknown argument `{other}`");
                print_help();
                return ExitCode::FAILURE;
            }
        }
    }

    let root = match resolve_root(root) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("gen_coq: could not locate repo root: {e}");
            return ExitCode::FAILURE;
        }
    };

    let repo = match Repo::from_root(root.clone()) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("gen_coq: failed to load repo: {e}");
            return ExitCode::FAILURE;
        }
    };

    let generated = match emit_generated(&repo) {
        Ok(g) => g,
        Err(e) => {
            eprintln!("gen_coq: generation error: {e}");
            return ExitCode::FAILURE;
        }
    };
    let checker = match emit_checker_ml(&repo) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("gen_coq: checker generation error: {e}");
            return ExitCode::FAILURE;
        }
    };

    let out_generated = root.join("tools/contract-proofs/theories/Generated.v");
    let out_checker = root.join("tools/contract-proofs/extraction/checker.ml");

    if check {
        match std::fs::read_to_string(&out_generated) {
            Ok(existing) if existing == generated => {}
            Ok(_) => {
                eprintln!("gen_coq: Generated.v is STALE — run `make gen` to regenerate");
                return ExitCode::FAILURE;
            }
            Err(_) => {
                eprintln!("gen_coq: Generated.v is MISSING — run `make gen` to generate");
                return ExitCode::FAILURE;
            }
        }
        match std::fs::read_to_string(&out_checker) {
            Ok(existing) if existing == checker => {}
            Ok(_) => {
                eprintln!("gen_coq: checker.ml is STALE — run `make gen` to regenerate");
                return ExitCode::FAILURE;
            }
            Err(_) => {
                eprintln!("gen_coq: checker.ml is MISSING — run `make gen` to generate");
                return ExitCode::FAILURE;
            }
        }
        println!("gen_coq: Generated.v and checker.ml are up to date");
        return ExitCode::SUCCESS;
    }

    if let Err(e) = std::fs::write(&out_generated, &generated) {
        eprintln!("gen_coq: failed to write {}: {e}", out_generated.display());
        return ExitCode::FAILURE;
    }
    println!("gen_coq: wrote {}", out_generated.display());
    if let Err(e) = std::fs::write(&out_checker, &checker) {
        eprintln!("gen_coq: failed to write {}: {e}", out_checker.display());
        return ExitCode::FAILURE;
    }
    println!("gen_coq: wrote {}", out_checker.display());
    ExitCode::SUCCESS
}

fn print_help() {
    eprintln!("Usage: gen_coq [--root <path>] [--check]");
    eprintln!("  --root <path>  Repository root (default: discovered upwards from CWD)");
    eprintln!("  --check        Exit non-zero if theories/Generated.v is stale");
}

/// Discover the repo root by walking up from CWD until we find the
/// `tools/contract-tests` marker directory.
fn resolve_root(explicit: Option<PathBuf>) -> std::io::Result<PathBuf> {
    if let Some(p) = explicit {
        return Ok(p);
    }
    let mut dir = std::env::current_dir()?;
    loop {
        if dir.join("tools/contract-tests").is_dir() && dir.join("skills").is_dir() {
            return Ok(dir);
        }
        match dir.parent() {
            Some(parent) => dir = parent.to_path_buf(),
            None => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "no repo root found walking upward from CWD \
                     (looked for tools/contract-tests + skills)",
                ))
            }
        }
    }
}

/// Convert a fixture state name like `ready-for-dev` into a Coq constructor
/// identifier like `S_ready_for_dev`.
fn ctor_id(state: &str) -> String {
    format!("S_{}", state.replace('-', "_"))
}

/// Convert a (from, to) pair into a Coq Transition constructor name like
/// `T_ready_for_dev_in_dev`.
fn trans_ctor_id(from: &str, to: &str) -> String {
    format!("T_{}_{}", from.replace('-', "_"), to.replace('-', "_"))
}

/// Build a sorted, de-duplicated adjacency map from the handoff graph
/// edges. Edges whose `from` or `to` is not a canonical state (or that are
/// self-loops) are dropped.
fn adjacency(canonical: &HashSet<String>, edges: &[Transition]) -> HashMap<String, Vec<String>> {
    let mut adj: HashMap<String, Vec<String>> = HashMap::new();
    for t in edges {
        if canonical.contains(&t.from) && canonical.contains(&t.to) && t.from != t.to {
            adj.entry(t.from.clone()).or_default().push(t.to.clone());
        }
    }
    for dests in adj.values_mut() {
        dests.sort();
        dests.dedup();
    }
    adj
}

/// Conservative "distance to termination" rank. The exact numbers do not
/// matter for correctness — only that terminal states share the minimum
/// rank. We derive rank from the fixture's position in `canonical_states`
/// (intentionally ordered entry → terminal), reversed, so earlier states
/// have a larger rank.
fn rank_of(state: &str, canonical: &[String], canon_idx: &HashMap<String, usize>) -> u32 {
    canon_idx
        .get(state)
        .map(|&i| (canonical.len() - 1 - i) as u32)
        .unwrap_or(0)
}

/// Find a shortest path from `start` to any state in `targets` using BFS.
/// Returns the path including both endpoints, or `None` if unreachable.
fn shortest_path_to_any(
    start: &str,
    targets: &HashSet<String>,
    adj: &HashMap<String, Vec<String>>,
) -> Option<Vec<String>> {
    if targets.contains(start) {
        return Some(vec![start.to_string()]);
    }
    let mut prev: HashMap<String, String> = HashMap::new();
    let mut visited: HashSet<String> = HashSet::new();
    visited.insert(start.to_string());
    let mut queue: VecDeque<String> = VecDeque::new();
    queue.push_back(start.to_string());
    while let Some(node) = queue.pop_front() {
        let Some(neighbors) = adj.get(&node) else {
            continue;
        };
        for next in neighbors {
            if visited.contains(next) {
                continue;
            }
            visited.insert(next.clone());
            prev.insert(next.clone(), node.clone());
            if targets.contains(next) {
                let mut path: Vec<String> = vec![next.clone()];
                let mut cursor = next.clone();
                while let Some(p) = prev.get(&cursor) {
                    path.push(p.clone());
                    cursor = p.clone();
                }
                path.reverse();
                return Some(path);
            }
            queue.push_back(next.clone());
        }
    }
    None
}

/// For each non-terminal state, the shortest path to `done`.
fn done_witnesses(
    canonical: &[String],
    terminal: &HashSet<String>,
    adj: &HashMap<String, Vec<String>>,
) -> Vec<(String, Vec<String>)> {
    let done = "done".to_string();
    let mut out = Vec::new();
    for s in canonical {
        if terminal.contains(s) {
            continue;
        }
        match shortest_path_to_any(s, &HashSet::from([done.clone()]), adj) {
            Some(p) => out.push((s.clone(), p)),
            None => eprintln!(
                "gen_coq: WARNING: no path {s} -> done; witness will not be emitted"
            ),
        }
    }
    out
}

/// Emit a Coq list literal `[ a; b; c ]` from constructor IDs.
fn coq_list(items: &[String]) -> String {
    if items.is_empty() {
        "[]".to_string()
    } else {
        format!("[ {} ]", items.join("; "))
    }
}

fn emit_generated(repo: &forge_contract_tests::types::Repo) -> std::io::Result<String> {
    let canonical: Vec<String> = repo.fixture.canonical_states.clone();
    let terminal: Vec<String> = repo.fixture.terminal_states.clone();
    let entry: Vec<String> = repo.fixture.entry_points.clone();

    let canonical_set: HashSet<String> = canonical.iter().cloned().collect();
    let terminal_set: HashSet<String> = terminal.iter().cloned().collect();

    let canon_idx: HashMap<String, usize> = canonical
        .iter()
        .enumerate()
        .map(|(i, s)| (s.clone(), i))
        .collect();

    // Deduplicate edges by (from, to); keep one Transition constructor per
    // unique directed edge (triggers and conditions are explanatory and do
    // not change the reachability graph).
    let mut unique_edges: Vec<(String, String)> = Vec::new();
    let mut seen: HashSet<(String, String)> = HashSet::new();
    for t in &repo.handoff_graph.edges {
        let pair = (t.from.clone(), t.to.clone());
        if canonical_set.contains(&t.from)
            && canonical_set.contains(&t.to)
            && t.from != t.to
            && seen.insert(pair.clone())
        {
            unique_edges.push(pair);
        }
    }
    unique_edges.sort();

    // Adjacency used both for `successors` and for the A1 witnesses — they
    // MUST agree, since the witnesses are checked against `Transition`.
    let rel_adj = adjacency(&canonical_set, &repo.handoff_graph.edges);

    let mut out = String::new();
    out.push_str("(** * Generated Forge State Machine\n\n");
    out.push_str("    DO NOT EDIT BY HAND. This file is produced by\n");
    out.push_str("        tools/contract-tests/src/bin/gen_coq.rs\n");
    out.push_str("    from the parsed fixture and skill handoff graph.\n");
    out.push_str("    Regenerate with `make gen` (from tools/contract-proofs).\n\n");
    out.push_str("    Source of truth:\n");
    out.push_str("      - tools/contract-tests/fixtures/loop-contract.yaml\n");
    out.push_str("      - skills/<*>/<*>/HANDOFFS.md (parsed via Repo::from_root)\n");
    out.push_str("*)\n\n");

    out.push_str("Require Import Stdlib.Lists.List.\n");
    out.push_str("Import ListNotations.\n\n");

    // --- State ---------------------------------------------------------
    out.push_str("(** Canonical delivery states (one per canonical_states entry). *)\n");
    out.push_str("Inductive State : Set :=\n");
    for (i, s) in canonical.iter().enumerate() {
        let sep = if i + 1 == canonical.len() { "." } else { "" };
        out.push_str(&format!("  | {}{}\n", ctor_id(s), sep));
    }
    out.push('\n');
    out.push_str("Scheme Equality for State.\n\n");
    out.push_str("Hint Constructors State : core.\n\n");

    // --- terminal_states ----------------------------------------------
    out.push_str("(** Terminal states: the loop must end here. *)\n");
    let terminal_ids: Vec<String> = terminal.iter().map(|s| ctor_id(s)).collect();
    out.push_str(&format!(
        "Definition terminal_states : list State := {}.\n\n",
        coq_list(&terminal_ids)
    ));

    // --- is_terminal ---------------------------------------------------
    out.push_str("(** Decidable membership in [terminal_states]. *)\n");
    out.push_str("Definition is_terminal (s : State) : bool :=\n");
    out.push_str("  match s with\n");
    for s in &canonical {
        let b = if terminal_set.contains(s) { "true" } else { "false" };
        out.push_str(&format!("  | {} => {}\n", ctor_id(s), b));
    }
    out.push_str("  end.\n\n");

    // NOTE: [is_terminal] and [terminal_states] are aligned by construction,
    // but no proof currently needs the [is_terminal s = true <-> In s
    // terminal_states] equivalence, so we do not emit it. (Previously a
    // generated spec trio lived here; it was unused and has been removed.)

    // --- all_states (exhaustive list, for the runtime checker) ----------
    out.push_str("(** All canonical states, in fixture order. Used by the\n");
    out.push_str("    extracted checker to iterate exhaustively. *)\n");
    let all_ids: Vec<String> = canonical.iter().map(|s| ctor_id(s)).collect();
    out.push_str(&format!(
        "Definition all_states : list State := {}.\n\n",
        coq_list(&all_ids)
    ));

    // --- entry_points --------------------------------------------------
    out.push_str("(** Entry points of the Perfect Loop. *)\n");
    let entry_ids: Vec<String> = entry.iter().map(|s| ctor_id(s)).collect();
    out.push_str(&format!(
        "Definition entry_points : list State := {}.\n\n",
        coq_list(&entry_ids)
    ));

    // --- Transition relation ------------------------------------------
    out.push_str("(** Allowed one-step state transitions.\n");
    out.push_str("    One constructor per unique directed edge in the handoff graph. *)\n");
    out.push_str("Inductive Transition : State -> State -> Prop :=\n");
    if unique_edges.is_empty() {
        // Vacuous relation — still syntactically valid in Coq.
        out.push_str("  | T_none : False -> Transition S_done S_done.\n");
    } else {
        for (i, (from, to)) in unique_edges.iter().enumerate() {
            let sep = if i + 1 == unique_edges.len() { "." } else { "" };
            out.push_str(&format!(
                "  | {} : Transition {} {}{}\n",
                trans_ctor_id(from, to),
                ctor_id(from),
                ctor_id(to),
                sep,
            ));
        }
    }
    out.push('\n');
    out.push_str("Hint Constructors Transition : core.\n\n");

    // --- successors ----------------------------------------------------
    out.push_str("(** Successor function: post-image of each state.\n");
    out.push_str("    Derived from the [Transition] constructors above. *)\n");
    out.push_str("Definition successors (s : State) : list State :=\n");
    out.push_str("  match s with\n");
    for s in &canonical {
        let dests: Vec<String> = rel_adj
            .get(s)
            .map(|v| v.iter().map(|d| ctor_id(d)).collect())
            .unwrap_or_default();
        out.push_str(&format!("  | {} => {}\n", ctor_id(s), coq_list(&dests)));
    }
    out.push_str("  end.\n\n");

    // --- successors ↔ Transition agreement (generated, exact) ----------
    // [successors] is emitted from the SAME edges as [Transition], so they
    // agree by construction. We prove each direction with an explicit,
    // generated proof that names positions and constructors — no hand
    // case analysis to drift.
    //
    // Map each source state to the ordered list of its destinations, so we
    // can emit positional membership tactics.
    let mut succ_by_src: HashMap<&String, Vec<&String>> = HashMap::new();
    for e in &unique_edges {
        succ_by_src.entry(&e.0).or_default().push(&e.1);
    }

    // successors_complete: for each Transition constructor T_x_y, prove
    // In y (successors x) by positioning y in the literal successor list.
    out.push_str("Lemma successors_complete_generated :\n");
    out.push_str("  forall s s', Transition s s' -> In s' (successors s).\n");
    out.push_str("Proof.\n");
    out.push_str("  intros s s' H. inversion H; subst; simpl.\n");
    for (from, to) in &unique_edges {
        let dests = succ_by_src.get(from).cloned().unwrap_or_default();
        let pos = dests.iter().position(|d| *d == to).unwrap_or(0);
        let mut tac = String::new();
        for _ in 0..pos {
            tac.push_str("right; ");
        }
        tac.push_str("left; reflexivity");
        out.push_str(&format!("  - {}.\n", tac));
    }
    out.push_str("Qed.\n\n");

    // successors_sound: for each source state x, [In s' (successors x)]
    // unfolds to the literal successor list; destruct to depth = list
    // length, each branch [subst; constructor] closes with the matching
    // T_x_y constructor. For empty-list sources (terminals), the
    // membership is [False].
    out.push_str("Lemma successors_sound_generated :\n");
    out.push_str("  forall s s', In s' (successors s) -> Transition s s'.\n");
    out.push_str("Proof.\n");
    out.push_str("  intros s s' H. destruct s; simpl in H.\n");
    for st in &canonical {
        let dests = succ_by_src.get(st).cloned().unwrap_or_default();
        if dests.is_empty() {
            out.push_str("  - inversion H.\n");
        } else {
            // Pattern: [? | [? | ... | []]]  (n-1 nested, depth n).
            let mut pat = String::from("[?");
            for _ in 1..dests.len() {
                pat.push_str(" | [?");
            }
            pat.push_str(" | []");
            for _ in 0..dests.len() {
                pat.push(']');
            }
            out.push_str(&format!("  - destruct H as {}.\n", pat));
            for d in &dests {
                let _ = d;
                out.push_str("    subst; constructor.\n");
            }
        }
    }
    out.push_str("Qed.\n\n");

    // --- rank ----------------------------------------------------------
    out.push_str("(** Well-founded measure: distance to termination.\n");
    out.push_str("    Derived from canonical_states ordering (entry -> terminal). *)\n");
    out.push_str("Definition rank (s : State) : nat :=\n");
    out.push_str("  match s with\n");
    for s in &canonical {
        let r = rank_of(s, &canonical, &canon_idx);
        out.push_str(&format!("  | {} => {}\n", ctor_id(s), r));
    }
    out.push_str("  end.\n\n");

    // --- Reachability inductives (after State and Transition, so the
    // --- references resolve; shared with Paths.v / Properties.v). --------
    out.push_str("(** A path: non-empty list, each adjacent pair connected by [Transition]. *)\n");
    out.push_str("Inductive Path : list State -> Prop :=\n");
    out.push_str("  | path_singleton : forall s, Path [s]\n");
    out.push_str("  | path_step : forall s1 s2 ss,\n");
    out.push_str("      Transition s1 s2 -> Path (s2 :: ss) -> Path (s1 :: s2 :: ss).\n\n");
    out.push_str("(** Reachability: zero or more transitions. *)\n");
    out.push_str("Inductive Reachable : State -> State -> Prop :=\n");
    out.push_str("  | reach_refl : forall s, Reachable s s\n");
    out.push_str("  | reach_step : forall s m t, Transition s m -> Reachable m t -> Reachable s t.\n\n");
    out.push_str("Hint Constructors Path Reachable : core.\n\n");

    // --- A1 reachability witnesses ------------------------------------
    // For each entry point, emit a `Reachable entry done` lemma with a
    // one-`apply`-per-hop proof term. Coq re-checks every hop.
    let witnesses = done_witnesses(&canonical, &terminal_set, &rel_adj);
    out.push_str("(** ** A1 reachability witnesses (generated)\n\n");
    out.push_str("    For each entry point, a shortest path to [done] is emitted as a\n");
    out.push_str("    [Reachable entry done] proof term. Coq re-checks every hop against\n");
    out.push_str("    the [Transition] constructors above. If the fixture changes and a\n");
    out.push_str("    hop no longer exists, the witness fails to type-check. *)\n\n");

    for (state, path) in &witnesses {
        let lemma_name = format!("{}_reaches_done", state.replace('-', "_"));
        if path.len() < 2 {
            // State reaches itself trivially (only happens if state == done).
            out.push_str(&format!(
                "Lemma {} : Reachable {} {}.\n",
                lemma_name,
                ctor_id(state),
                ctor_id("done")
            ));
            out.push_str("Proof. apply reach_refl. Qed.\n\n");
            continue;
        }
        out.push_str(&format!(
            "Lemma {} : Reachable {} {}.\n",
            lemma_name,
            ctor_id(state),
            ctor_id("done")
        ));
        out.push_str("Proof.\n");
        for w in path.windows(2) {
            out.push_str(&format!(
                "  apply (reach_step _ {}). apply {}.\n",
                ctor_id(&w[1]),
                trans_ctor_id(&w[0], &w[1]),
            ));
        }
        out.push_str("  apply reach_refl.\n");
        out.push_str("Qed.\n\n");
    }

    // --- Covering theorem: every entry point reaches [done] --------------
    // Data-driven: [simpl] reduces [In s entry_points] to a disjunction
    // with one branch per entry point, and each branch is closed by the
    // matching witness lemma above. If an entry point is added without a
    // witness, the corresponding [apply] fails and the build breaks.
    out.push_str("(** Covering theorem: every entry point reaches [done]. *)\n");
    out.push_str("Theorem entry_points_reach_done :\n");
    out.push_str("  forall s, In s entry_points -> Reachable s S_done.\n");
    out.push_str("Proof.\n");
    out.push_str("  intros s H. simpl in H.\n");
    if entry.is_empty() {
        out.push_str("  inversion H.\n");
    } else {
        // [a;b;c;d] simplifies (In s) to [a=s \/ (b=s \/ (c=s \/ (d=s \/ False)))].
        // That is n disjuncts plus a trailing False, so the destruct pattern is
        // [H | [H | ... | [H | _]]] with n openings and a final `_`.
        let mut binds = String::new();
        for (i, _) in entry.iter().enumerate() {
            if i == 0 {
                binds.push_str("[H");
            } else {
                binds.push_str(" | [H");
            }
        }
        binds.push_str(" | Hf");
        for _ in 0..entry.len() {
            binds.push(']');
        }
        out.push_str(&format!("  destruct H as {}.\n", binds));
        for s in &entry {
            let lemma = format!("{}_reaches_done", s.replace('-', "_"));
            out.push_str(&format!("  - subst. apply {}.\n", lemma));
        }
        // The trailing [False] branch from [In] over a finite list.
        out.push_str("  - inversion Hf.\n");
    }
    out.push_str("Qed.\n\n");

    // --- Covering theorem: every non-terminal state reaches [done] ------
    // Same shape, over all non-terminal states.
    let non_terminal: Vec<&String> = canonical
        .iter()
        .filter(|s| !terminal_set.contains(*s))
        .collect();
    out.push_str("(** Covering theorem: every non-terminal state reaches [done]. *)\n");
    out.push_str("Theorem non_terminal_states_reach_done :\n");
    out.push_str("  forall s, is_terminal s = false -> Reachable s S_done.\n");
    out.push_str("Proof.\n");
    out.push_str("  intros s Hne. destruct s; simpl in Hne;\n");
    out.push_str("    try discriminate.\n");
    for s in &non_terminal {
        let lemma = format!("{}_reaches_done", s.replace('-', "_"));
        out.push_str(&format!("  - apply {}.\n", lemma));
    }
    out.push_str("Qed.\n\n");

    out.push_str("Hint Constructors Transition : core.\n");
    out.push_str("Hint Constructors Reachable : core.\n");

    out.push_str(&format!(
        "\n(** [{}] canonical states, [{}] terminal, [{}] entry, [{}] edges. *)\n",
        canonical.len(),
        terminal.len(),
        entry.len(),
        unique_edges.len()
    ));
    let _ = Path::new("");
    Ok(out)
}

/// Emit `extraction/checker.ml` — the runtime checker that consumes the
/// extracted `forge_state_machine` module. Iterates `all_states` and
/// `entry_points` rather than hard-coding the state list, so it cannot drift
/// from the fixture. The only place individual state constructor names appear
/// is `string_of_state` (a derived `show` for human-readable FAIL messages),
/// and that match is generated exhaustively from `canonical_states`.
fn emit_checker_ml(repo: &forge_contract_tests::types::Repo) -> std::io::Result<String> {
    let canonical: &[String] = &repo.fixture.canonical_states;
    let terminal: &HashSet<String> = &repo
        .fixture
        .terminal_states
        .iter()
        .cloned()
        .collect::<HashSet<_>>();

    let mut out = String::new();
    out.push_str("(** Runtime checker for the Forge Perfect Loop.\n\n");
    out.push_str("    DO NOT EDIT BY HAND. This file is produced by\n");
    out.push_str("        tools/contract-tests/src/bin/gen_coq.rs\n");
    out.push_str("    from the same fixture as Generated.v. Iterates the\n");
    out.push_str("    extracted `all_states` / `entry_points` — no hard-coded state list.\n");
    out.push_str("    Regenerate with `make gen`.\n");
    out.push_str("*)\n\n");
    out.push_str("open Forge_state_machine\n\n");

    // string_of_state: derived show, one arm per canonical state. OCaml will
    // warn on non-exhaustive match if the generator and the State inductive
    // (both driven by canonical_states) ever disagree.
    out.push_str("let string_of_state = function\n");
    for s in canonical {
        out.push_str(&format!("  | {} -> \"{}\"\n", ctor_id(s), s));
    }
    out.push('\n');

    out.push_str("let check_no_dead_ends () =\n");
    out.push_str("  let failures = List.fold_left (fun acc s ->\n");
    out.push_str("    if not (is_terminal s) && successors s = [] then s :: acc else acc\n");
    out.push_str("  ) [] all_states in\n");
    out.push_str("  match failures with\n");
    out.push_str("  | [] -> Printf.printf \"OK: every non-terminal state has at least one successor.\\n\"\n");
    out.push_str("  | _ ->\n");
    out.push_str("      Printf.printf \"FAIL: dead-end non-terminal states: %s\\n\"\n");
    out.push_str("        (String.concat \", \" (List.map string_of_state failures));\n");
    out.push_str("      exit 1\n\n");

    out.push_str("let check_terminals_are_sinks () =\n");
    out.push_str("  let failures = List.fold_left (fun acc s ->\n");
    out.push_str("    if is_terminal s && successors s <> [] then s :: acc else acc\n");
    out.push_str("  ) [] terminal_states in\n");
    out.push_str("  match failures with\n");
    out.push_str("  | [] -> Printf.printf \"OK: terminal states have no outgoing edges.\\n\"\n");
    out.push_str("  | _ ->\n");
    out.push_str("      Printf.printf \"FAIL: terminal states with outgoing edges: %s\\n\"\n");
    out.push_str("        (String.concat \", \" (List.map string_of_state failures));\n");
    out.push_str("      exit 1\n\n");

    out.push_str("let check_reachability target entry =\n");
    out.push_str("  (* Level-based BFS: [steps] tracks graph depth, not node count.\n");
    out.push_str("     Returns true if [target] is reachable within [max_iterations] hops. *)\n");
    out.push_str("  let module S = Set.Make (struct\n");
    out.push_str("    type t = state\n");
    out.push_str("    let compare = compare\n");
    out.push_str("  end) in\n");
    out.push_str("  let rec bfs visited frontier steps =\n");
    out.push_str("    if steps > max_iterations then false\n");
    out.push_str("    else match frontier with\n");
    out.push_str("    | [] -> false\n");
    out.push_str("    | _ ->\n");
    out.push_str("        let frontier' = List.filter (fun s -> not (S.mem s visited)) frontier in\n");
    out.push_str("        if List.exists (fun s -> s = target) frontier' then true\n");
    out.push_str("        else begin\n");
    out.push_str("          let visited' =\n");
    out.push_str("            List.fold_left (fun acc s -> S.add s acc) visited frontier'\n");
    out.push_str("          in\n");
    out.push_str("          let next_frontier =\n");
    out.push_str("            List.concat_map (fun s -> successors s) frontier'\n");
    out.push_str("          in\n");
    out.push_str("          let next_frontier' =\n");
    out.push_str("            List.filter (fun s -> not (S.mem s visited')) next_frontier\n");
    out.push_str("          in\n");
    out.push_str("          bfs visited' next_frontier' (steps + 1)\n");
    out.push_str("        end\n");
    out.push_str("  in\n");
    out.push_str("  bfs S.empty [entry] 0\n\n");

    out.push_str("let check_entry_points () =\n");
    out.push_str("  let failures = List.fold_left (fun acc s ->\n");
    out.push_str("    if not (check_reachability S_done s) then s :: acc else acc\n");
    out.push_str("  ) [] entry_points in\n");
    out.push_str("  match failures with\n");
    out.push_str("  | [] -> Printf.printf \"OK: every entry point can reach done.\\n\"\n");
    out.push_str("  | _ ->\n");
    out.push_str("      Printf.printf \"FAIL: entry points that cannot reach done: %s\\n\"\n");
    out.push_str("        (String.concat \", \" (List.map string_of_state failures));\n");
    out.push_str("      exit 1\n\n");

    out.push_str("let () =\n");
    out.push_str("  check_no_dead_ends ();\n");
    out.push_str("  check_terminals_are_sinks ();\n");
    out.push_str("  check_entry_points ();\n");
    out.push_str("  Printf.printf \"All Forge Perfect Loop checks passed.\\n\"\n");

    let _ = terminal;
    Ok(out)
}
