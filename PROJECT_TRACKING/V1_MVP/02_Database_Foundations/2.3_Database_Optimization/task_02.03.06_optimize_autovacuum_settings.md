# Task: Optimize Autovacuum Settings for Performance

**Task ID:** V1_MVP/02_Database_Foundations/2.3_Database_Optimization/task_02.03.06_optimize_autovacuum_settings.md
**Version:** V1_MVP
**Phase:** 02_Database_Foundations
**Module:** 2.3_Database_Optimization
**Priority:** Medium
**Status:** Todo
**Assignee:**
**Created Date:** 2025-01-21
**Last Updated:** 2025-01-21

## Detailed Description:
Configure optimal autovacuum settings for tables with high write activity to maintain database performance and prevent table bloat.

## Specific Sub-tasks:
- [ ] 1. Identify high-write tables (stock_moves, sessions, audit_logs)
- [ ] 2. Configure table-specific autovacuum settings
- [ ] 3. Set autovacuum_vacuum_scale_factor for aggressive vacuum
- [ ] 4. Set autovacuum_analyze_scale_factor for frequent analyze
- [ ] 5. Configure autovacuum_max_workers for parallel processing
- [ ] 6. Test autovacuum performance on large tables
- [ ] 7. Monitor table bloat and vacuum effectiveness

## Acceptance Criteria:
- [ ] Autovacuum settings optimized for high-write tables
- [ ] Table bloat prevented through aggressive vacuum settings
- [ ] No performance degradation from table bloat
- [ ] Autovacuum workers configured optimally
- [ ] Monitoring in place for vacuum effectiveness

## Dependencies:
- V1_MVP/02_Database_Foundations/2.3_Database_Optimization/task_02.03.04_enable_pg_stat_statements.md

## Related Documents:
- `migrations/20250110000008_optimize_autovacuum.sql` (file to be created)
- `infra/docker_compose/docker-compose.yml` (postgresql.conf updates)

## Notes / Discussion:
---
* Stock_moves table needs aggressive autovacuum due to high INSERT volume
* Balance between vacuum frequency and performance impact
* Monitor dead tuples and last vacuum time
* Consider manual VACUUM for very large tables during maintenance windows

## AI Agent Log:
---
* (Log will be automatically updated by AI agent when starting and executing task)