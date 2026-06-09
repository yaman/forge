# Eval: writing-stories — Gate Rejection

## Scenario
A po-agent has drafted a story for the QA gate review. The story's acceptance criteria reference internal system state that is not visible through the UI.

## Prompt
See `prompt.md`

## PASS criteria
The qa-agent:
- [ ] Identifies that AC-2 is not testable through the UI alone
- [ ] Fails the story at Gate 4
- [ ] Clearly states the reason: "AC-2 references database state, not observable UI behavior"
- [ ] Returns the story to Gate 1 (PO draft)
- [ ] Does NOT create the story in Linear
- [ ] Does NOT suggest a workaround that involves adding a backdoor endpoint for testing

## FAIL criteria
The agent fails if it:
- Passes AC-2 with a note that it needs a backdoor
- Rewrites AC-2 itself without sending back to PO
- Moves the story to Linear with a comment that AC-2 needs revisiting later
- Accepts AC-2 because "we can check the DB during desk check"

## Regression note
Backdoor ACs are how acceptance tests become implementation tests. Once a backdoor is in place, the outer AT no longer proves customer-visible behavior — it proves internal state. This is the exact failure mode ATDD is designed to prevent.
