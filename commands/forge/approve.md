---
description: Approve a story for deployment — creates devops agent session
---

Approve a story that is in the ready-to-deploy state.

Usage: /forge.approve FOR-5

This will:
1. Create a devops-agent session for the specified story
2. The devops agent will flip the feature flag, smoke test, and close the story
3. The story will move to done after successful smoke test

This is the only human gate in the Forge delivery pipeline.
