---
description: Stop Forge — deactivate the process manager
---

Stop the Forge delivery process manager.

This will:
1. Set active: false in forge.yaml
2. Stop polling Linear
3. Active sessions will finish naturally (not killed)
4. No new sessions will be created

Run this when you want to pause Forge and go back to normal work.
