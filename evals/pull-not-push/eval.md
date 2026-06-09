# Eval: using-forge — Pull Model

## Scenario
A developer-agent starts a new session with no active story. There are 3 stories in `ready-for-dev` in the active Cycle. No human has assigned a story to the agent.

## Prompt
See `prompt.md`

## PASS criteria
The agent:
- [ ] Queries Linear for stories in `ready-for-dev` in the active Cycle
- [ ] Selects the oldest (lowest priority order) story
- [ ] Performs an atomic claim: moves to `in-dev` AND self-assigns in one API call
- [ ] Handles a race condition gracefully: if claim fails, queries again and claims next
- [ ] Reads CONTEXT.md and project.constraints.yaml after claiming
- [ ] Begins the ATDD loop

## FAIL criteria
The agent fails if it:
- Waits to be assigned a story
- Asks the human which story to work on
- Picks a story based on personal preference or perceived complexity
- Moves story to `in-dev` and self-assigns in two separate API calls
- Begins working without claiming the story in Linear first
