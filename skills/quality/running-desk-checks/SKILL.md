---
name: running-desk-checks
level: L2-GUIDED
owner: qa-agent
trigger: developer-agent completes all sub-slices for an AC and signals desk check ready
---

# running-desk-checks

## Description

Verifies a completed AC against its acceptance criteria through the UI — exactly as an outside customer would. Produces a desk check artifact. The developer agent may not proceed to the next AC until the desk check is approved. This is a human-visible checkpoint, not a rubber stamp.

---

## Protocol

### Step 1 — Verify locally
```
1. Pull the branch (trunk — there are no feature branches)
2. Run the outer Acceptance Test for this AC → must be GREEN
3. Open the application locally
4. Execute each step of the AC manually, as a customer would:
   - Use only the UI
   - No database inspection
   - No API calls
   - No internal state checks
5. For each AC step: does the UI behave exactly as the AC states?
```

### Step 2 — Verify on test environment
```
1. Confirm story is deployed to test environment (devops-agent confirms)
2. Repeat Step 1 on the test environment URL
3. Check feature flag is OFF on test environment (story not yet accepted)
```

### Step 3 — Produce desk check artifact

Update `stories/[STORY-ID].md` AC section:
```yaml
ac:
  - id: AC-1
    desk_check:
      status: approved          # or: failed
      checked_by: qa-agent
      checked_at: 2026-06-09T21:00:00Z
      environment: local + test
      notes: ""
      # if failed:
      failure_reason: ""
      failure_screenshot: ""
```

### Step 4 — Signal result

**If approved:**
```
Post to Linear story comment: "Desk check AC-[N]: APPROVED ✓"
Update story snapshot
Notify developer-agent: proceed to next AC
```

**If failed:**
```
Post to Linear story comment:
  "Desk check AC-[N]: FAILED
   Reason: [specific description]
   Expected: [what AC says]
   Actual: [what UI shows]"
Move story back to `in-dev`
Developer agent must fix and re-trigger desk check — do not move to next AC
```

---

## What a Desk Check Is Not

- Not a code review
- Not a performance test
- Not a security audit
- Not a regression test (that's running-regression-suite)

A desk check is: does this AC work, right now, through the UI, as a customer would use it.
