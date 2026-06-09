# PROJ-28: Order History Dashboard

## Story
As a returning customer, I want to see my previous order history on the dashboard.

## ACs

### AC-1: List last 5 orders
Given logged-in customer with orders, when they visit dashboard, then they see last 5 orders.

**Desk check:** approved
**Status:** done

### AC-2: Order detail rows
Given order list visible, when customer views a row, then they see date, total, and item count.

**Status:** in-progress

#### Sub-slices
- SS-1: Render order row component
  - fe_status: done
  - be_status: done
- SS-2: Render order date and total from API response
  - fe_status: in-progress
  - be_status: pending
