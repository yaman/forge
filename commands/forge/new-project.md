---
description: Start Forge — begin inception (8 phases) then transition to development mode
---

Start the Forge delivery framework for a new project.

This will:
1. Check if Forge workflow states exist in Linear (create them if missing)
2. Verify the Linear team is fresh if states need creating
3. Start inception mode — Phase 1 through Phase 8 sequentially
4. Each phase creates a session for the appropriate agent with the right skill
5. After Phase 8, transition to development mode (polling Linear for stories)

Inception phases:
- Phase 1: Lean Canvas (po-agent)
- Phase 2: Empathy Mapping (ux-agent)
- Phase 3: Trade-off Sliders (po-agent)
- Phase 4: Event Storming (po-agent)
- Phase 5: UX/UI Design (ux-agent)
- Phase 6: Story Writing (po-agent — writes stories to Linear)
- Phase 7: Tech Stack + Architecture (architect-agent)
- Phase 8: Iteration Mapping (po-agent)

If inception is already complete, starts development mode directly.
