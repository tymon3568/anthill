# Task Template

**Task ID:** V1_MVP/[Phase]/[Module]/task_[XX.YY.ZZ]_[description].md
**Version:** V1_MVP
**Phase:** [Phase Number and Name]
**Module:** [Module Number and Name]
**Priority:** [High/Medium/Low]
**Status:** Todo
**Assignee:** 
**Created Date:** YYYY-MM-DD
**Last Updated:** YYYY-MM-DD

## Detailed Description:
[Clear, concise description of what needs to be accomplished in this task. Include context, background, and why this task is important.]

[If applicable, mention which components/services are affected, what patterns to follow, or what standards to adhere to.]

## Specific Sub-tasks:
- [ ] 1. [First actionable sub-task]
    - [ ] 1.1. [Optional nested sub-task if needed]
    - [ ] 1.2. [Optional nested sub-task if needed]
- [ ] 2. [Second actionable sub-task]
- [ ] 3. [Third actionable sub-task]
- [ ] 4. [Fourth actionable sub-task]
- [ ] 5. [Additional sub-tasks as needed]

## Acceptance Criteria:
- [ ] [Specific, measurable criterion 1]
- [ ] [Specific, measurable criterion 2]
- [ ] [Specific, measurable criterion 3]
- [ ] [Code compiles without errors: `cargo check --workspace`]
- [ ] [Tests pass if applicable: `cargo test`]
- [ ] [Documentation updated if applicable]
- [ ] [Git commits reference this task ID]

## Dependencies:
*   Task: `task_[XX.YY.ZZ]_description.md` (Status: [check actual status])
*   Task: `task_[XX.YY.ZZ]_description.md` (Status: [check actual status])
*   [List all tasks that must be completed before this one can start]
*   [If no dependencies, write "None"]

## Related Documents:
*   `[path/to/architecture/doc.md]`
*   `[path/to/relevant/code/file.rs]`
*   `[link to API spec or database schema]`
*   `[external documentation URL]`

## Notes / Discussion:
---
*   [Area for questions, clarifications, or important decisions made during planning]
*   [Design considerations or alternative approaches considered]
*   [Any risks or concerns identified]

## AI Agent Log:
---
*   YYYY-MM-DD HH:MM: [First action or planning note]
    - [Detail about initial assessment or planning]
    - [Any dependencies verified or concerns noted]

---

## Template Usage Instructions

### When Creating a New Task:

1. **Copy this template** to the appropriate location:
   ```
   PROJECT_TRACKING/V1_MVP/[Phase]/[Module]/task_XX.YY.ZZ_description.md
   ```

2. **Fill in the header fields**:
   - Task ID: Follow pattern `task_XX.YY.ZZ_short_description.md`
   - Version: Usually `V1_MVP` for current development
   - Phase: e.g., `03_User_Service`
   - Module: e.g., `3.2_Casbin_Authorization`
   - Priority: Assess based on project needs
   - Status: Always start with `Todo`
   - Assignee: Leave empty initially
   - Dates: Use current date for Created Date

3. **Write clear description**:
   - Explain WHAT needs to be done
   - Explain WHY it's important
   - Provide necessary context

4. **Break down into sub-tasks**:
   - Make each sub-task actionable and clear
   - Estimate if sub-tasks can be completed in 1-4 hours each
   - Order sub-tasks logically
   - Use nested sub-tasks for complex items

5. **Define acceptance criteria**:
   - Make criteria specific and measurable
   - Include technical requirements (compilation, tests, etc.)
   - Include quality requirements (documentation, code review, etc.)

6. **List dependencies accurately**:
   - Check status of each dependency
   - Verify dependencies are logical and necessary
   - Consider both code and knowledge dependencies

7. **Add related documents**:
   - Link to architecture docs
   - Link to related code files
   - Link to API specs or schemas
   - Add external references if needed

### Task Numbering Convention:

```
Format: task_[Phase].[Module].[Sequential]_description.md

Examples:
- task_03.02.01_create_casbin_model.md
- task_03.02.10_integration_tests.md
- task_04.01.05_implement_product_repository.md
```

**Phase**: 2-digit number (01-12)
**Module**: 2-digit number within phase (01-99)
**Sequential**: 2-digit task number (01-99)

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

### Example Sub-tasks (Reference):

**For API Implementation**:
- [ ] 1. Define request/response DTOs in core/dto/
- [ ] 2. Create service trait in core/domain/
- [ ] 3. Implement service in infra/
- [ ] 4. Create handler in api/handlers/
- [ ] 5. Add route in api/main.rs
- [ ] 6. Add OpenAPI annotations
- [ ] 7. Write integration tests
- [ ] 8. Update API documentation

**For Database Work**:
- [ ] 1. Create migration file with SQL
- [ ] 2. Update database ERD documentation
- [ ] 3. Update domain models in core/
- [ ] 4. Update repository traits in core/
- [ ] 5. Implement repository in infra/
- [ ] 6. Add indexes for performance
- [ ] 7. Test migration up/down
- [ ] 8. Verify schema with cargo check

**For Testing**:
- [ ] 1. Set up test fixtures and helpers
- [ ] 2. Write positive test cases
- [ ] 3. Write negative test cases
- [ ] 4. Write edge case tests
- [ ] 5. Add integration tests
- [ ] 6. Verify all tests pass
- [ ] 7. Check code coverage
- [ ] 8. Update test documentation

### Example Acceptance Criteria (Reference):

**For Features**:
- [ ] Feature works as specified in requirements
- [ ] All edge cases handled appropriately
- [ ] Error messages are clear and helpful
- [ ] Code follows project conventions
- [ ] No compiler warnings
- [ ] Tests achieve >80% coverage
- [ ] Documentation updated
- [ ] Reviewed by at least one other person

**For Bug Fixes**:
- [ ] Root cause identified and documented
- [ ] Fix resolves the issue completely
- [ ] Regression test added
- [ ] No new issues introduced
- [ ] Related areas tested
- [ ] Fix documented in commit message

**For Refactoring**:
- [ ] Behavior unchanged (verified by tests)
- [ ] Code is more readable/maintainable
- [ ] Performance not degraded
- [ ] All existing tests still pass
- [ ] New tests added if needed
- [ ] Documentation reflects changes
