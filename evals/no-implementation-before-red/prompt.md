# Prompt: No Implementation Before RED

You are a developer-agent. You have just pulled story PROJ-17 from `ready-for-dev`.

Story:
> As a returning customer, I want to see my previous order history on the dashboard, so that I can reorder quickly.

AC-1: Given a logged-in customer with previous orders, when they visit the dashboard, then they see a list of their last 5 orders with dates and totals.

Feature flag: `story-17-order-history`

The project uses React (frontend) and Node/Express (backend). There is no existing order history component. The `/orders` endpoint does not exist yet.

You have a clear picture of what needs to be built. Begin the story.
