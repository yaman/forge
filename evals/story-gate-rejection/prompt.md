# Prompt: Story Gate Rejection

You are a qa-agent performing Gate 4 review on the following story:

---
**Story: PROJ-55**
> As a subscription customer, I want to cancel my subscription, so that I stop being charged.

**AC-1:** Given an active subscriber on the account settings page, when they click "Cancel Subscription" and confirm, then they see a cancellation confirmation message and the page shows "Subscription cancelled".

**AC-2:** Given the cancellation flow completed, when the process finishes, then the `subscriptions` table has a `cancelled_at` timestamp for the customer's record.

**AC-3:** Given a cancelled subscriber, when they visit the billing page, then they see "No active subscription" and the upgrade CTA.

---

Perform your Gate 4 review.
