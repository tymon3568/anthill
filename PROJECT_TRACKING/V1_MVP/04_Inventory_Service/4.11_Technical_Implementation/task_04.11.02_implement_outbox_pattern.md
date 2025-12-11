# Task: Implement Outbox Pattern for Reliable Events

**Task ID:** V1_MVP/04_Inventory_Service/4.11_Technical_Implementation/task_04.11.02_implement_outbox_pattern.md
**Version:** V1_MVP
**Phase:** 04_Inventory_Service
**Module:** 4.11_Technical_Implementation
**Priority:** High
**Status:** InProgress_By_Grok_Agent
**Assignee:** Grok_Agent
**Created Date:** 2025-10-21
**Last Updated:** 2025-12-11

## Detailed Description:
Implement the Transactional Outbox pattern to ensure reliable, at-least-once delivery of events to NATS.

## Specific Sub-tasks:
- [ ] 1. Create a SQL migration for the `event_outbox` table.
- [ ] 2. Modify business logic (e.g., in GRN validation, DO shipping) to write events to the `event_outbox` table within the same database transaction as the main data change.
- [ ] 3. Create a background worker (e.g., a separate thread or process) that periodically polls the `event_outbox` table for `pending` events.
- [ ] 4. The worker should attempt to publish the event to NATS. On success, it updates the event's status to `published`.

## Acceptance Criteria:
- [ ] An `event_outbox` table migration is created.
- [ ] Business logic is updated to write events to the outbox table transactionally.
- [ ] A background worker is implemented to poll the table and publish events.
- [ ] The system can recover and send events even if the message broker was temporarily down.
- [ ] Integration tests verify the reliability of event publishing.

## Dependencies:
*   (Requires NATS to be part of the infrastructure)

## Related Documents:
*   `inventory_service/src/main.rs`

## Notes / Discussion:
---
*   (Khu vực dành cho các câu hỏi, thảo luận, hoặc ghi chú trong quá trình thực hiện)

## AI Agent Log:
---
*   2025-12-11 00:53: Starting work on task_04.11.02_implement_outbox_pattern.md
*   (Log sẽ được AI agent tự động cập nhật khi bắt đầu và thực hiện task)
