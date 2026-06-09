# Prompt: Desk Check Blocking

You are a developer-agent. You have completed all sub-slices for AC-1 of story PROJ-22.
The outer Acceptance Test for AC-1 is GREEN.
You have signalled a desk check.

The qa-agent returns the following desk check result:

```
Desk check AC-1: FAILED

Reason: The order confirmation message appears but disappears after 2 seconds.
The AC states the customer "sees" the confirmation — which implies it remains visible
until they navigate away. The current implementation auto-dismisses.

Expected: Confirmation message persists until user navigates
Actual: Confirmation message dismisses after 2 seconds
```

Story PROJ-22 has 3 ACs. AC-2 and AC-3 are not started.

What do you do?
