# Forge Architecture

> 24 skills, 7 agents, 1 opencode plugin, 2 delivery modes (inception + development).

## Overview

Forge is a lean software delivery framework for AI agents. It runs as an
[opencode](https://opencode.ai) plugin that coordinates 7 specialized agents
through a lean software delivery pipeline — from project inception to
production deployment.

The plugin is the **coordinator**. It does NOT write code, make decisions, or
interact with humans beyond toasts. It:
1. Polls Linear for stories in pull states
2. Claims stories by moving them to active states
3. Creates agent sessions with prompts that include handoff comments
4. Detects session completion and creates the next agent session
5. Recovers from crashes by checking session liveness and Linear state

### Two Modes

| Mode | Trigger | Flow |
|------|---------|------|
| **Inception** | `/forge new project` | 8 sequential phases, driven by `session.idle` |
| **Development** | After Phase 8, or when `forge.yaml` has `active: true` and inception artifacts exist | Polls Linear, creates agent sessions, handles handoffs |

---

## System Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     opencode (TUI + Server)                   │
│                                                               │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │              Forge Plugin (coordinator)                   │ │
│  │                                                           │ │
│  │  ┌─────────────┐  ┌──────────────┐  ┌────────────────┐ │ │
│  │  │ Inception    │  │ Development   │  │ Crash Recovery  │ │ │
│  │  │ Mode         │  │ Mode          │  │                 │ │ │
│  │  │              │  │               │  │                 │ │ │
│  │  │ • 8 phases   │  │ • Poll Linear │  │ • session.list()│ │ │
│  │  │ • session.idle│ │ • Claim story │  │ • Check state   │ │ │
│  │  │ • Artifacts  │  │ • Create sess│  │ • Recover/Halt  │ │ │
│  │  │              │  │ • Handoff     │  │                 │ │ │
│  │  └─────────────┘  └──────────────┘  └────────────────┘ │ │
│  │                                                           │ │
│  │  Hooks: session.idle, session.error,                     │ │
│  │         experimental.session.compacting,                  │ │
│  │         tui.command.execute                              │ │
│  │                                                           │ │
│  │  Persistence: .forge/sessions.json, .forge/project-state.json │ │
│  └─────────────────────────────────────────────────────────┘ │
│                                                               │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │              Linear MCP Server                           │ │
│  │  (agents use this for writes: save_issue, save_comment,  │ │
│  │   list_issues, get_issue, etc.)                          │ │
│  └─────────────────────────────────────────────────────────┘ │
│                                                               │
│  ┌────────────┐  ┌────────────┐  ┌────────────────────────┐ │
│  │ Agent Sessions (LLM-powered)                              │ │
│  │                                                            │ │
│  │  po-agent   ux-agent   architect-agent   developer-agent  │ │
│  │  qa-agent   devops-agent   secops-agent                   │ │
│  └──────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
         │
         ▼
┌─────────────────┐
│   Linear (API)   │
│                  │
│  Workflow States │
│  Stories         │
│  Comments        │
│  Projects/Cycles │
└─────────────────┘
```

---

## Functional Areas

### 1. Plugin Coordinator (`src/plugin.ts`)

The core orchestrator. Runs inside opencode as a plugin.

**Inception Mode:**
- Triggered by `/forge new project`
- Creates 8 sequential sessions (one per phase)
- Each phase: create session with inception prompt → wait for idle → check artifact exists → next phase
- After Phase 8: transition to development mode, start polling

**Development Mode:**
- `setInterval` polls Linear every N seconds
- Finds stories in pull states (ready-for-dev, ready-for-qa, etc.)
- Claims story: `updateStoryState(storyId, activeState)` in Linear
- Reads last handoff comment from Linear
- Creates agent session with prompt (includes handoff comment)
- On `session.idle`: check Linear state → create next agent or toast for human gate

**Failsafe (on session.idle):**
- If story state unchanged (agent forgot to move it):
  - Check for recent handoff comment (posted during this session)
  - If comment exists → auto-advance to next pull state with warning
  - If no comment → halt as `halted-ambiguous`

**Crash Recovery (on plugin startup):**
- Read `.forge/sessions.json`
- Call `client.session.list()` to find dead sessions
- Check Linear state for orphaned stories
- Create recovery sessions for stories still in active states

### 2. Linear Client (`src/linear-client.ts`)

Read-only polling + state claim + comment read. ~350 lines.

| Method | Purpose |
|--------|---------|
| `pollStories(states)` | Find stories in pull states |
| `getStoryState(id)` | Check current story state |
| `updateStoryState(id, state)` | Claim story (move to active state) |
| `getLastComment(id)` | Read handoff comment for next agent's prompt |
| `getLastCommentWithDate(id)` | Read comment with timestamp (for failsafe) |
| `postComment(id, body)` | Post handoff comment (used in simulation) |
| `ensureWorkflowStates()` | Create 14 Forge workflow states in Linear |
| `hasForgeStates()` | Check if Forge states exist |
| `isFreshTeam()` | Check if team has no issues (fresh) |

### 3. Prompt Builder (`src/prompt-builder.ts`)

Generates agent prompts. Three prompt types:

| Function | When | Contains |
|----------|------|----------|
| `buildPrompt()` | Development sessions | Story details, handoff comment, skill instructions, loop contract, cost budget, handoff protocol |
| `buildInceptionPrompt()` | Inception phases | Phase name, skill, expected output artifact, special instructions (Phase 6/8) |
| `buildLoopPrompt()` | Recovery sessions | Resume protocol: run AT first, read loop state file, continue from last sub-slice |

### 4. Config (`src/config.ts`)

Loads/saves/validates `forge.yaml`.

- `loadConfig()` — parse YAML to typed config
- `saveConfig()` — write partial updates (active flag, max_concurrent_stories)
- `validateConfig()` — check required fields

### 5. Skills (`skills/` — 24 skills)

Each skill is a pair: `SKILL.md` (instructions) + `LOOP.md` (state machine contract).

| Category | Skills |
|----------|--------|
| Meta | using-forge, resuming-sessions, guarding-loops |
| Discovery | facilitating-inception, facilitating-event-storming, establishing-ubiquitous-language, designing-ux, writing-stories, building-iteration-map |
| Architecture | selecting-tech-stack, establishing-architecture, deciding-architecture |
| Iteration Zero | bootstrapping-project, validating-test-harness, securing-pipeline |
| Development | running-atdd-sessions, running-tdd-loops, managing-feature-flags |
| Quality | running-desk-checks, writing-acceptance-tests, running-regression-suite |
| Acceptance | approving-stories, finishing-stories, modeling-threats |

### 6. Agent Definitions (`agents/` — 7 agents)

Each agent has a `.md` file with:
- Model selection
- Skill permission boundaries (explicit allow/deny)
- Role description
- Session start protocol

| Agent | Skills Allowed |
|-------|---------------|
| po-agent | using-forge, facilitating-inception, facilitating-event-storming, establishing-ubiquitous-language, writing-stories, building-iteration-map, approving-stories |
| ux-agent | using-forge, facilitating-inception, facilitating-event-storming, designing-ux |
| architect-agent | using-forge, deciding-architecture, selecting-tech-stack, establishing-architecture |
| developer-agent | using-forge, resuming-sessions, guarding-loops, running-atdd-sessions, running-tdd-loops, managing-feature-flags |
| qa-agent | using-forge, resuming-sessions, guarding-loops, writing-acceptance-tests, running-desk-checks, running-regression-suite, validating-test-harness |
| devops-agent | using-forge, bootstrapping-project, managing-feature-flags, securing-pipeline, finishing-stories |
| secops-agent | using-forge, modeling-threats, securing-pipeline |

---

## Delivery State Machine

Linear is the single source of truth for story state. The plugin coordinator
owns pull→active transitions (claiming). Agents own active→next-pull transitions
(completion).

```
                        ┌──────────────────┐
                        │  in-analysis     │
                        │  (PO refines)    │
                        └────────┬─────────┘
                                 │
                    ┌────────────▼────────────┐
                    │  ready-for-dev           │
                    │  (plugin polls, claims)  │
                    └────────────┬────────────┘
                                 │ plugin claims
                    ┌────────────▼────────────┐
                    │  in-dev                  │
                    │  (developer-agent ATDD)  │
                    └────────────┬────────────┘
                                 │ agent moves
                    ┌────────────▼────────────┐
                    │  ready-for-qa            │
                    │  (plugin polls, claims)  │
                    └────────────┬────────────┘
                                 │ plugin claims
                    ┌────────────▼────────────┐
                    │  in-qa                   │
                    │  (qa-agent regression)   │
                    └────────────┬────────────┘
                                 │ agent moves
                    ┌────────────▼────────────┐
                    │  ready-for-acceptance    │
                    │  (plugin polls, claims)  │
                    └────────────┬────────────┘
                                 │ plugin claims
                    ┌────────────▼────────────┐
                    │  in-acceptance           │
                    │  (po-agent smoke test)   │
                    └────────────┬────────────┘
                                 │ agent moves
                    ┌────────────▼────────────┐
                    │  ready-to-deploy         │
                    │  (HUMAN GATE)            │
                    └────────────┬────────────┘
                                 │ /forge.approve
                    ┌────────────▼────────────┐
                    │  done                    │
                    │  (devops deploys)        │
                    └─────────────────────────┘

          Failsafe (state unchanged + no comment):
          ┌─────────────────────────────┐
          │  halted-ambiguous            │
          │  halted-stall                │
          │  halted-human-gate            │
          │  halted-unsafe               │
          └─────────────────────────────┘
```

---

## Inception Flow (8 Phases)

```
Phase 1: Lean Canvas (po-agent)        → docs/lean-canvas.md
    │
Phase 2: Empathy Mapping (ux-agent)    → docs/empathy-map.md
    │
Phase 3: Trade-off Sliders (po-agent)  → project.constraints.yaml
    │
Phase 4: Event Storming (po-agent)     → docs/event-storm.yaml + CONTEXT.md
    │
Phase 5: UX/UI Design (ux-agent)       → design-system/MASTER.md
    │
Phase 6: Story Writing (po-agent)      → Stories in Linear (ready-for-dev)
    │
Phase 7: Tech Stack + Architecture     → docs/adr/ADR-001-platform.md
    │      (architect-agent)             docs/adr/ADR-002-code-architecture.md
    │
Phase 8: Iteration Mapping (po-agent)   → Linear Projects + Cycles
    │
    ▼
Development Mode (polling starts)
```

---

## File Layout

```
forge/
├── src/
│   ├── plugin.ts          # Forge plugin (coordinator) — ~900 lines
│   ├── linear-client.ts   # Read-only polling + claim + comment read — ~350 lines
│   ├── prompt-builder.ts  # Prompt generation (dev, inception, recovery) — ~220 lines
│   ├── config.ts          # forge.yaml load/save/validate — ~142 lines
│   └── types.ts           # Shared types — ~150 lines
├── bin/
│   └── forge.ts           # forge init CLI — ~100 lines
├── tests/
│   ├── plugin.test.ts     # 41 tests (prompts, failsafe, recovery, types)
│   ├── config.test.ts     # 18 tests (load, validate, save)
│   ├── linear-client.test.ts # 15 tests (poll, state, workflow)
│   └── prompt-builder.test.ts # 14 tests (buildPrompt, buildLoopPrompt)
├── agents/                # 7 agent definitions (.md with skill permissions)
├── skills/                # 24 skills (SKILL.md + LOOP.md each)
├── integrations/          # 4 external tool installers (.sh)
├── .opencode/
│   ├── commands/forge/    # 4 slash commands
│   ├── plugins/           # forge.ts (copied by forge init)
│   ├── opencode.json      # Plugin registration + Linear MCP
│   └── agents/            # Installed by forge init
├── forge.yaml             # PM config (gitignored — has Linear API key)
├── opencode.json          # MCP config + plugin registration
├── package.json           # @loopworx/forge npm package
├── tsconfig.json          # TypeScript config (strict, ESNext)
├── .github/workflows/
│   ├── ci.yml             # typecheck + test + build + verify init
│   └── release.yml        # publish to npm on tag
└── README.md
```

---

## Testing

87 tests across 4 files, all passing.

```bash
bun test          # run all tests
bun run typecheck # tsc --noEmit (src/ + bin/)
bun run build     # compile plugin to dist/plugin.js
```

| Test File | Tests | Coverage |
|-----------|-------|----------|
| plugin.test.ts | 41 | Prompt building, handoff comments, inception prompts, failsafe logic, crash recovery, session persistence, command routing |
| config.test.ts | 18 | Config loading, validation, saving, defaults |
| linear-client.test.ts | 15 | Poll stories, get state, workflow states, freshness check |
| prompt-builder.test.ts | 14 | Dev prompts, QA prompts, PO prompts, recovery prompts, LOOP.md reading |

---

## Key Design Decisions

1. **Plugin, not daemon** — The coordinator runs inside opencode as a plugin. No separate process, no SQLite, no separate TUI.

2. **Linear as state spine** — All delivery state lives in Linear. The plugin polls it. Agents write to it via Linear MCP.

3. **Coordinator owns claims** — The plugin moves stories from pull states to active states (e.g., ready-for-dev → in-dev). Agents move stories from active states to next pull states (e.g., in-dev → ready-for-qa).

4. **Handoff comments** — Agents post compact comments on Linear before ending. The plugin reads the last comment and includes it in the next agent's prompt. This gives instant context without plan files or conversation summaries.

5. **Failsafe via comment check** — If an agent ends its session without moving the Linear state, the plugin checks for a recent handoff comment. If one exists, it auto-advances. If not, it halts.

6. **Crash recovery via session.list()** — On plugin startup, the plugin reads `.forge/sessions.json`, calls `client.session.list()` to find dead sessions, checks Linear state, and creates recovery sessions for orphaned stories.

7. **Inception is sequential** — 8 phases, one at a time. Each phase produces an artifact. `session.idle` triggers the next phase. No polling during inception.

8. **Development is parallel** — Up to `max_concurrent_stories` can be active. Polling catches new stories. Session idle triggers handoffs.
