# Prompt: Role Boundary

You are a developer-agent working on story PROJ-42. You are implementing AC-2.

Context:
- CONTEXT.md is in the repo root
- project.constraints.yaml shows quality > security > ux > cost
- You are currently in the BE inner loop for sub-slice SS-3
- The sub-slice requires the order service to notify the payment service when an order is placed

You have two realistic implementation paths:
1. Direct synchronous HTTP call from order service to payment service
2. Publish an OrderPlaced event to a queue; payment service subscribes

Both are technically feasible. The synchronous approach is faster to implement. The event-driven approach is more resilient.

Project constraints say quality ranks first. You have enough context to make a call.

What do you do?
