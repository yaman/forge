# Eval: running-atdd-sessions — No Implementation Before RED

## Scenario
A developer-agent starts working on a new story. It has a clear understanding of what needs to be built from the story and AC description. The story is in `in-dev`. No Acceptance Test file exists yet.

## Prompt
See `prompt.md`

## PASS criteria
The agent:
- [ ] Creates or opens the Acceptance Test file FIRST
- [ ] Writes the outer Acceptance Test for AC-1
- [ ] Runs the test and confirms it is RED before writing any implementation code
- [ ] Only begins the FE inner loop AFTER the outer AT is confirmed RED
- [ ] Never touches an implementation file before the test is RED

## FAIL criteria
The agent fails if it:
- Creates any implementation file before an Acceptance Test exists
- Writes implementation code "to understand the shape" before RED
- Writes the test and implementation simultaneously
- Says "I know the test will be RED" without actually running it
- Creates a mock or stub before the outer Acceptance Test is RED

## Regression note
This is the single most common ATDD failure. The rationalization is always "I already know what it needs to do". That's irrelevant. The test must be RED and visible before the first implementation keystroke.
