# Forge Evals

Evals verify two things:
1. **Triggering** — does the right skill fire from a natural prompt?
2. **Behavior** — does the agent follow the skill's state machine correctly?

Each eval lives in `evals/[skill-name]/` and contains:
- `prompt.md` — the input scenario given to the agent
- `eval.md` — what a PASS looks like vs a FAIL
- `fixtures/` — any files the agent needs to read (CONTEXT.md, story snapshot, etc.)

## Running Evals

```bash
# Run a single eval manually
# Give the agent the prompt and check its behavior against eval.md
cat evals/[skill-name]/prompt.md

# Automated (Claude Code)
claude -p "$(cat evals/[skill-name]/prompt.md)" --max-turns 5
```

## Priority Evals

These are the highest-stakes skills — if they fail, the whole delivery process breaks.

| Eval | Skill | Why Critical |
|---|---|---|
| `using-forge` | L1 RIGID meta | Must resist rationalization and enforce precedence |
| `no-implementation-before-red` | running-atdd-sessions | The most common failure mode |
| `session-resume` | resuming-sessions | Prevents context window loss from corrupting work |
| `story-gate-rejection` | writing-stories | Stories with bad ACs must not reach the backlog |
| `pull-not-push` | using-forge | Agents must pull, not wait to be assigned |
| `role-boundary` | using-forge | Developer agent must not make architecture decisions |
| `iteration-completion` | using-forge | Agent must post completion and idle, not start next iteration |
