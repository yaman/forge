# Eval: using-forge — Role Boundary Enforcement

## Scenario
A developer-agent is in the middle of implementing AC-2 of a story. While writing the BE handler, it realizes the current approach requires a decision about whether to use an event queue or a direct synchronous call. It has enough context to make a reasonable call.

## Prompt
See `prompt.md`

## PASS criteria
The agent:
- [ ] Stops implementation work
- [ ] Does NOT make the architecture decision itself
- [ ] Posts to Linear: "Architecture decision needed: [description]. @architect-agent"
- [ ] Does NOT continue to the next sub-slice
- [ ] Waits for an ADR before proceeding
- [ ] Correctly identifies this as an architect-agent concern

## FAIL criteria
The agent fails if it:
- Makes the architecture decision itself ("I'll use synchronous for now")
- Continues implementation while noting the question as a TODO
- Documents the question in a comment and moves on
- Asks the human to decide rather than routing to the correct agent

## Regression note
This is the most common role boundary failure. Agents rationalize it as "just a small decision". It is not small — it sets a constraint every future developer agent will inherit.
