# Task: {Task Name}

> **Template Type:** Full Task Template (for complex tasks)
> For simpler tasks within sub-modules, see the Quick Task Template in `docs/module-implementation-workflow.md` Section 2.5.6

**Task ID:** `V1_MVP/{Phase}/{Module}/{Sub-Module}/task_{NN.NN.NN.NN}_{description}.md`
**Version:** V1_MVP
**Phase:** {NN_Phase_Name}
**Module:** {N.NN_Module_Name}
**Sub-Module:** {N.NN.N_Sub_Module_Name}
**Priority:** {High | Medium | Low}
**Status:** Todo
**Assignee:**
**Created Date:** {YYYY-MM-DD}
**Last Updated:** {YYYY-MM-DD}
**Dependencies:**
- `V1_MVP/{path}/task_{XX.XX.XX}_{description}.md`
- None (if no dependencies)

---

## 1. Detailed Description

{Comprehensive description of what needs to be built. Include:
- Purpose and context
- What problem this solves
- How it fits into the larger module/sub-module
- Any important background information}

---

## 2. Implementation Steps (Specific Sub-tasks)

- [ ] 1. {First actionable sub-task}
    - [ ] 1.1. {Optional nested sub-task if needed}
    - [ ] 1.2. {Optional nested sub-task if needed}
- [ ] 2. {Second actionable sub-task}
- [ ] 3. {Third actionable sub-task}
- [ ] 4. {Fourth actionable sub-task}
- [ ] 5. {Additional sub-tasks as needed}

---

## 3. Completion Criteria

- [ ] {Specific, measurable criterion 1}
- [ ] {Specific, measurable criterion 2}
- [ ] {Specific, measurable criterion 3}
- [ ] Code compiles without errors
- [ ] Tests pass (unit/integration as applicable)
- [ ] Documentation updated if applicable
- [ ] Git commits reference this task ID

---

## 4. Technical Specifications

### Architecture/Design:

```
{Include relevant diagrams, file structure, or code architecture}
```

### Key Implementation Details:

1. **{Component/Feature 1}**
   - Implementation approach
   - Key decisions

2. **{Component/Feature 2}**
   - Implementation approach
   - Key decisions

### API Endpoints (if applicable):

| Method | Endpoint | Purpose |
|--------|----------|---------|
| GET | `/api/v1/{endpoint}` | {Purpose} |
| POST | `/api/v1/{endpoint}` | {Purpose} |

### Files to Create/Modify:

| File Path | Action | Description |
|-----------|--------|-------------|
| `path/to/file.ts` | Create | {Description} |
| `path/to/existing.ts` | Modify | {Description} |

---

## 5. UI/UX Specifications (Frontend tasks only)

### Layout:

```
+--------------------------------------------------+
|  {Wireframe or ASCII layout}                     |
+--------------------------------------------------+
```

### Responsive Behavior:

| Breakpoint | Layout Changes |
|------------|----------------|
| Mobile | {Description} |
| Tablet | {Description} |
| Desktop | {Description} |

---

## 6. Related Documents

- `docs/{path}/PRD.md` - Module/Sub-Module PRD
- `docs/database-erd.dbml` - Database schema
- `shared/openapi/{service}.yaml` - API specification
- {Additional relevant documentation}

---

## 7. Notes / Discussion

---

- **Open Question**: {Question that needs resolution}
- **Decision Made**: {Important decision and rationale}
- **Future Consideration**: {Something to consider for later}

---

## AI Agent Log:

---

* {YYYY-MM-DD HH:MM}: {Agent} - {Action description}
    - {Detail about what was done}
    - {Any dependencies verified or concerns noted}

* {YYYY-MM-DD HH:MM}: {Agent} - {Action description}
    - {Detail about what was done}
    - {Files created/modified}

---

## Quality Gate Results (before NeedsReview):

| Check | Status | Notes |
|-------|--------|-------|
| TypeCheck | ☐ Pass / ☐ Fail | {Notes} |
| Lint | ☐ Pass / ☐ Fail | {Notes} |
| Unit Tests | ☐ Pass / ☐ Fail | {X tests, Xms} |
| Build | ☐ Pass / ☐ Fail | {Notes} |

---

## Template Usage Instructions

### Task ID Numbering Convention (4-Level):

```
Format: task_{Phase}.{Module}.{SubModule}.{Sequential}_{description}.md

Examples:
- task_08.10.01.01_product_list_page.md      (Phase 08, Module 10, SubModule 01, Task 01)
- task_08.10.01.02_product_form.md           (Phase 08, Module 10, SubModule 01, Task 02)
- task_04.02.00.05_warehouse_api.md          (No sub-module: use 00)
```

| Level | Format | Example |
|-------|--------|---------|
| Phase | `{NN}` | `08` (Frontend) |
| Module | `{NN}` | `10` (Inventory_UI) |
| Sub-Module | `{NN}` | `01` (Product_Management) |
| Task | `{NN}` | `01`, `02`, ... |

### Valid Status Values (STRICT - folder-tasks):

| Status | Meaning |
|--------|---------|
| `Todo` | Ready to be claimed |
| `InProgress_By_[AgentName]` | Claimed and actively being worked |
| `Blocked_By_[Reason]` | Cannot proceed; must include reason |
| `NeedsReview` | Work complete; awaiting review |
| `Done` | Reviewed and approved |
| `Cancelled` | Intentionally stopped (with reason) |

❌ **Invalid examples (NEVER use):** `InProgress`, `In Progress`, `Completed`, `Pending`, `Waiting`, `Done_By_AI`

### Priority Guidelines:

- **High**: Blocking other work, critical path, security issues
- **Medium**: Important but not blocking, normal feature work
- **Low**: Nice to have, refactoring, documentation improvements

### Status Progression:

```
Todo → InProgress_By_[Agent] → NeedsReview → Done
         ↓
    Blocked_By_[Reason]
```

### AI Agent Workflow:

1. **Claim Task**: Update status to `InProgress_By_[YourAgentName]`
2. **Check Dependencies**: Verify all dependencies are `Done`
3. **Work Sub-tasks**: Mark checkboxes as completed
4. **Log Progress**: Add entries to AI Agent Log
5. **Quality Gates**: Run typecheck/lint/tests before completion
6. **Complete**: Set status to `NeedsReview`

### Example AI Agent Log Entry:

```markdown
* 2026-01-27 14:30: Claude_Agent_01 - Task claimed
    - Verified dependencies: task_08.10.01.00 is Done ✓
    - Starting work on sub-task 1

* 2026-01-27 16:00: Claude_Agent_01 - All sub-tasks completed
    - Files created: ProductList.svelte, product.test.ts
    - Quality gates: typecheck ✓, lint ✓, tests ✓ (5 tests, 120ms)
    - Status changed to NeedsReview
```

---

## When to Use Which Template

| Template | Location | When to Use |
|----------|----------|-------------|
| **Quick Template** | `docs/module-implementation-workflow.md` Section 2.5.6 | Most tasks in sub-modules. Simple, follows folder-tasks standard. |
| **Full Template** | This file (`docs/templates/task-template.md`) | Complex standalone tasks requiring detailed specs, UI/UX design, or extensive technical documentation. |
| **PROJECT_TRACKING Template** | `PROJECT_TRACKING/TASK_TEMPLATE.md` | Quick reference when creating tasks. Contains both quick format and usage instructions. |

**Decision Guide:**

```
Is task in a sub-module?
├── Yes → Use Quick Template (Section 2.5.6)
└── No → Is task complex (>5 sub-tasks, needs UI/UX specs, multiple files)?
          ├── Yes → Use Full Template (this file)
          └── No → Use Quick Template
```

**Key Differences:**

| Feature | Quick Template | Full Template |
|---------|----------------|---------------|
| Sections | 3 main (Description, Steps, Criteria) | 7+ (adds Tech Specs, UI/UX, Tests, etc.) |
| Use case | Standard feature tasks | Complex features, integrations, refactors |
| Time to fill | 5-10 min | 15-30 min |
| AI Agent friendly | ✓ Optimized | ✓ Supported |

---

## Related Resources

| Resource | Location |
|----------|----------|
| folder-tasks docs | Context7: `/tymon3568/folder-tasks` |
| Workflow guide | `docs/module-implementation-workflow.md` |
| Task helper script | `scripts/task-helper.sh` |
| Tasks overview | `PROJECT_TRACKING/TASKS_OVERVIEW.md` |
| AI Agent guide | `docs/AI_AGENT_AND_FOLDER_TASKS_GUIDE.md` |
