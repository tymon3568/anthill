# Task Template (folder-tasks format)

> **Reference:** [folder-tasks](https://github.com/tymon3568/folder-tasks)
> **Context7 ID:** `/tymon3568/folder-tasks`

---

## Quick Template (Copy & Paste)

```markdown
# Task: {Task Name}

**Task ID:** `V1_MVP/{Phase}/{Module}/{Sub-Module}/task_{NN.NN.NN.NN}_{description}.md`
**Version:** V1_MVP
**Phase:** {NN_Phase_Name}
**Module:** {N.NN_Module_Name}
**Sub-Module:** {N.NN.N_Sub_Module_Name}
**Priority:** High
**Status:** Todo
**Assignee:**
**Created Date:** YYYY-MM-DD
**Last Updated:** YYYY-MM-DD
**Dependencies:**
- `V1_MVP/{path}/task_{XX.XX.XX.XX}_{name}.md`
- None

## 1. Detailed Description

{Clear, concise description of what needs to be accomplished:
- Purpose and context
- What problem this solves
- How it fits into the larger module/sub-module}

## 2. Implementation Steps (Specific Sub-tasks)

- [ ] 1. {First actionable sub-task}
    - [ ] 1.1. {Optional nested sub-task}
- [ ] 2. {Second actionable sub-task}
- [ ] 3. {Third actionable sub-task}
- [ ] 4. {Write tests}
- [ ] 5. {Additional sub-tasks as needed}

## 3. Completion Criteria

- [ ] {Specific, measurable criterion 1}
- [ ] {Specific, measurable criterion 2}
- [ ] Code compiles without errors
- [ ] Tests pass (unit/integration as applicable)
- [ ] Documentation updated if applicable
- [ ] Git commits reference this task ID

## Related Documents

- Mini PRD: `./README.md`
- Database ERD: `docs/database-erd.dbml`
- API Spec: `shared/openapi/{service}.yaml`
- {Additional references}

## Notes / Discussion

{Area for questions, clarifications, or important decisions}

## AI Agent Log:

* YYYY-MM-DD HH:MM: {Agent} - {Action description}
    - {Details about what was done}
    - {Dependencies verified or concerns noted}

```

---

## Task ID Convention (4-Level)

```
Format: task_{Phase}.{Module}.{SubModule}.{Sequential}_{description}.md

Examples:
- task_08.10.01.01_product_list_page.md      # Phase 08, Module 10, SubModule 01, Task 01
- task_08.10.01.02_product_form.md           # Phase 08, Module 10, SubModule 01, Task 02
- task_04.02.00.05_warehouse_api.md          # No sub-module: use 00
```

| Level | Format | Example |
|-------|--------|---------|
| Phase | `{NN}` | `08` (Frontend) |
| Module | `{NN}` | `10` (Inventory_UI) |
| Sub-Module | `{NN}` | `01` (Product_Management), `00` if none |
| Task | `{NN}` | `01`, `02`, ... |

---

## Valid Status Values (STRICT)

| Status | Meaning |
|--------|---------|
| `Todo` | Ready to be claimed |
| `InProgress_By_[AgentName]` | Claimed and actively being worked |
| `Blocked_By_[Reason]` | Cannot proceed; must include reason |
| `NeedsReview` | Work complete; awaiting review |
| `Done` | Reviewed and approved |
| `Cancelled` | Intentionally stopped (with reason) |

❌ **Invalid (NEVER use):** `InProgress`, `In Progress`, `Completed`, `Pending`, `Waiting`, `Review`, `Done_By_AI`

---

## Status Progression

```
Todo → InProgress_By_[Agent] → NeedsReview → Done
         ↓
    Blocked_By_[Reason]
```

---

## Priority Guidelines

| Priority | When to use |
|----------|-------------|
| **High** | Blocking other work, critical path, security issues |
| **Medium** | Important but not blocking, normal feature work |
| **Low** | Nice to have, refactoring, documentation improvements |

---

## AI Agent Workflow

1. **Find Task**: Search for `Status: Todo` tasks
2. **Check Dependencies**: Verify all dependencies are `Done`
3. **Claim Task**: Update status to `InProgress_By_[YourAgentName]`
4. **Work Sub-tasks**: Mark checkboxes `[x]` as completed
5. **Log Progress**: Add entries to `AI Agent Log:`
6. **Quality Gates**: Run typecheck/lint/tests before completion
7. **Complete**: Set status to `NeedsReview`

---

## AI Agent Log Format

```markdown
## AI Agent Log:

* 2026-01-27 10:00: Backend_Agent_01 - Task claimed
    - Verified dependencies: task_08.10.00.01 is Done ✓
    - Starting work on sub-task 1

* 2026-01-27 12:30: Backend_Agent_01 - Sub-tasks 1-3 completed
    - Files created: ProductList.svelte, products.test.ts
    - Encountered: Type mismatch in API response
    - Resolution: Updated ProductDto interface

* 2026-01-27 14:00: Backend_Agent_01 - All work completed
    - Quality gates: typecheck ✓, lint ✓, tests ✓ (8 tests, 250ms)
    - Status changed to NeedsReview

* 2026-01-27 15:00: Reviewer_Agent - Reviewed and approved
    - Status changed to Done
```

---

## Directory Structure

**Simple Module (no sub-modules):**
```
PROJECT_TRACKING/V1_MVP/{Phase}/{Module}/
├── README.md                    # Module PRD
├── task_{NN.NN.00.01}_*.md     # Task 1 (use 00 for sub-module)
├── task_{NN.NN.00.02}_*.md     # Task 2
└── ...
```

**Complex Module (with sub-modules):**
```
PROJECT_TRACKING/V1_MVP/{Phase}/
├── {N.NN}_{Module_Name}/
│   ├── README.md                           # Module overview & sub-module index
│   │
│   ├── {N.NN.1}_{Sub_Module_1}/
│   │   ├── README.md                       # Sub-Module Mini PRD
│   │   ├── task_{NN.NN.01.01}_*.md
│   │   └── task_{NN.NN.01.02}_*.md
│   │
│   └── {N.NN.2}_{Sub_Module_2}/
│       ├── README.md
│       └── task_{NN.NN.02.01}_*.md
```

---

## Example: Complete Task File

```markdown
# Task: Product List Page

**Task ID:** `V1_MVP/08_Frontend/8.10_Inventory_UI/8.10.1_Product_Management/task_08.10.01.01_product_list_page.md`
**Version:** V1_MVP
**Phase:** 08_Frontend
**Module:** 8.10_Inventory_UI
**Sub-Module:** 8.10.1_Product_Management
**Priority:** High
**Status:** Todo
**Assignee:**
**Created Date:** 2026-01-27
**Last Updated:** 2026-01-27
**Dependencies:**
- `V1_MVP/04_Inventory_Service/task_04.01.00.01_product_api.md`

## 1. Detailed Description

Create the product list page that displays all products with pagination, search, and filtering capabilities. This is the main entry point for product management in the Inventory UI module.

## 2. Implementation Steps (Specific Sub-tasks)

- [ ] 1. Create TypeScript types for Product
- [ ] 2. Implement API client for products endpoint
- [ ] 3. Create Svelte store with pagination state
- [ ] 4. Build ProductList component with data table
- [ ] 5. Add search and filter functionality
- [ ] 6. Implement loading and error states
- [ ] 7. Write unit tests for store and components
- [ ] 8. Write E2E tests for list page

## 3. Completion Criteria

- [ ] Product list displays with correct data
- [ ] Pagination works (10, 25, 50 per page)
- [ ] Search filters products by name/SKU
- [ ] Loading spinner shows during API calls
- [ ] Error messages display on API failure
- [ ] All tests pass

## Related Documents

- Mini PRD: `./README.md`
- Database ERD: `docs/database-erd.dbml` (products table)
- API Spec: `shared/openapi/inventory.yaml`
- UI Architecture: `docs/ui-architecture-proposal.md`

## Notes / Discussion

- Use existing DataTable component from UI library
- Follow pagination pattern from Users module

## AI Agent Log:

* 2026-01-27 09:00: Waiting for assignment

```

---

## Quick Reference

| What | Where |
|------|-------|
| Full workflow | `docs/module-implementation-workflow.md` |
| Sub-module template | `docs/module-implementation-workflow.md` Section 2.5 |
| folder-tasks docs | Context7: `/tymon3568/folder-tasks` |
| Task helper script | `scripts/task-helper.sh` |
| Tasks overview | `PROJECT_TRACKING/TASKS_OVERVIEW.md` |
