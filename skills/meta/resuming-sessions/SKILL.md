---
name: resuming-sessions
level: L1-RIGID
owner: all-agents
trigger: any session where Linear shows an in-progress story assigned to you
---

# resuming-sessions

## Description

How to safely resume an in-progress story after a context window ends, a session crashes, or an agent restarts. The rule is simple: re-run the outer Acceptance Test before reading anything else. The test tells you where you are. Everything else lies.

---

## The Problem This Solves

Context windows end mid-story. When an agent restarts, it has:
- A plan file that says "implement the handler next"
- A conversation summary that says "FE loop done, starting BE loop"
- A Linear card that says `in-dev`
- No reliable memory of what code actually exists

The plan file and conversation summary reflect *intentions*, not *reality*. Only the test suite reflects reality.

---

## Resume Protocol

```
1. Query Linear → confirm story is still assigned to you and in `in-dev`

2. Run the outer Acceptance Test — do not read anything else first
   → RED:   note which ACs are still failing
             cross-reference with story snapshot in stories/[STORY-ID].md
             identify the last completed sub-slice
             resume ATDD loop from next sub-slice
   → GREEN: STOP. Do not proceed.
             Something unexpected happened.
             Post to Linear: "Outer AT is unexpectedly GREEN on resume — needs human review"
             Wait for human instruction.

3. Read CONTEXT.md
4. Read project.constraints.yaml
5. Continue ATDD loop
```

---

## What "Last Completed Sub-slice" Means

Check the story snapshot at `stories/[STORY-ID].md`.
Each AC has sub-slices. Each sub-slice has a status: `pending`, `in-progress`, `done`.

The last `done` sub-slice is where you resume from — start the *next* `pending` sub-slice.
If a sub-slice is `in-progress`, treat it as incomplete — restart it from scratch.

**Never assume a sub-slice is done because the plan file says so.**
Only the test result determines done.
