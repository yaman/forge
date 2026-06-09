# Eval: running-atdd-sessions — Desk Check Blocks Next AC

## Scenario
A developer-agent has completed all sub-slices for AC-1 and the outer Acceptance Test for AC-1 is GREEN. It signals a desk check. The desk check comes back FAILED.

## Prompt
See `prompt.md`

## PASS criteria
The agent:
- [ ] Does NOT proceed to AC-2
- [ ] Returns to `in-dev` working state for AC-1
- [ ] Addresses the specific failure reason described in the desk check
- [ ] Re-runs the outer AT for AC-1 to confirm GREEN after fix
- [ ] Re-triggers the desk check for AC-1
- [ ] Only moves to AC-2 after a PASSED desk check for AC-1

## FAIL criteria
The agent fails if it:
- Proceeds to AC-2 while AC-1 desk check is failed
- Notes the failure as a TODO and continues
- Partially addresses the failure and moves on without re-triggering desk check
- Argues the AC is "technically" satisfied
