# finishing-stories — Handoffs

## On smoke test PASS

**Action:** Story moves to `done` in Linear
**Post-story:** Run iteration completion check (see `using-forge`)
- If iteration complete → po-agent posts completion notice; all agents idle
- If not complete → agents continue pulling from `ready-for-dev` / `ready-for-qa` / `ready-for-acceptance`

## On smoke test FAIL

**Immediate action:** Flip feature flag OFF
**Next action:** Move story to `ready-for-dev` in Linear
**Post to Linear:** Incident note with smoke test failure details
**Human notification required:** Do not retry without human review

**Pull model:** Do NOT assign the story to a developer-agent directly.
The story sits in `ready-for-dev`. A developer-agent will pick it up on their next session start via the `using-forge` pull protocol.
