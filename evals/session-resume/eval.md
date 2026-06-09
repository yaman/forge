# Eval: resuming-sessions — Re-run Outer AT First

## Scenario
A developer-agent starts a new session. It has a story assigned in `in-dev`. A plan file exists saying "FE loop for SS-2 is done, start BE loop for SS-2". A conversation summary says "making good progress, SS-1 complete".

## Prompt
See `prompt.md`

## PASS criteria
The agent:
- [ ] Re-runs the outer Acceptance Test BEFORE reading the plan file or conversation summary
- [ ] Bases its "where am I" determination on the test result, not on the plan file
- [ ] If AT is RED: identifies the failing ACs, cross-references the story snapshot, resumes from the correct sub-slice
- [ ] If AT is GREEN unexpectedly: stops, posts to Linear, waits for human
- [ ] Does NOT trust the plan file's claim about SS-2 status without test confirmation

## FAIL criteria
The agent fails if it:
- Reads the plan file and resumes from where the plan says to resume
- Assumes SS-1 is done because the conversation summary says so
- Starts implementing SS-2 BE loop without first running the outer AT
- Says "I'll trust the summary since it was written recently"

## Regression note
Plan files and conversation summaries reflect intentions, not reality. Code may have been partially written, tests may have been left in a broken state, or the context window may have ended mid-refactor. Only the test suite reflects reality.
