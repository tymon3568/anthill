# PR Review Auto-Fix Workflow Rules (v2.1 - Bot-Aware + Multi-Agent Safe Edition)

## Core Principles

- **AUTONOMOUS**: AI tự động chờ, phát hiện, và sửa lỗi từ review bots mà không cần user intervention.
- **POLLING-BASED**: Sử dụng polling mechanism để detect khi bot review hoàn tất.
- **ITERATIVE**: Lặp lại quy trình cho đến khi PR "clean" và ready to merge.
- **SMART WAITING**: Tối ưu thời gian chờ dựa trên patterns của từng bot.
- **TRANSPARENCY**: Log all actions in the PR comments and task files for auditability.
- **CONSENSUS**: If unsure about a fix, request human approval before proceeding.
- **MULTI-AGENT SAFE (SSOT = Task Files)**: Không được sửa code khi chưa “claim task” hợp lệ trong `PROJECT_TRACKING/**/task_*.md`. Task file là single source of truth để tránh nhiều agents đạp nhau.
- **QUALITY GATES FIRST**: Chỉ được chuyển trạng thái “ready to merge” khi đã chạy và pass các quality gates phù hợp (fmt/check/clippy/test hoặc frontend equivalents).

## Bot Detection Configuration

> Note: Bot có thể viết review ở 2 nơi:
> - Review threads (line-specific)
> - PR conversation comments (general)
> Workflow này sẽ fetch cả hai, normalize và dedupe.

```yaml
known_review_bots:
  - name: "coderabbitai[bot]"
    alias: "CodeRabbit"
    avg_review_time: 2-5 min
    completion_signals:
      - "<!-- This is an auto-generated comment by CodeRabbit"
      - "## Summary"
    pending_signals:
      - "<!-- Generating review -->"
      
  - name: "github-actions[bot]"
    alias: "CI/CD"
    avg_review_time: 1-10 min
    completion_signals:
      - "All checks have passed"
      - "build: success"
    pending_signals:
      - "Waiting for status"
      
  - name: "greptile[bot]"
    alias: "Greptile"
    avg_review_time: 1-3 min
    completion_signals:
      - "## Code Review by Greptile"
```

## Workflow States

### New Guard State: TASK_CLAIM_GATE

Trước khi AI sửa code (Phase 5), phải đi qua “Task claim gate” để đảm bảo multi-agent safety:
- Có task file liên quan đến PR/feature hiện tại
- Task đang ở trạng thái `InProgress_By_[AgentName]`
- `Assignee` đúng agent hiện tại
- Dependencies đều `Done`

Nếu không thỏa:
- Update task thành `Blocked_By_Task_Not_Claimed` (hoặc lý do phù hợp)
- Dừng automation để tránh race conditions


```
┌─────────────────────────────────────────────────────────────────────────┐
│                         PR AUTO-FIX STATE MACHINE                       │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  ┌──────────┐    ┌──────────────┐    ┌─────────────┐    ┌───────────┐  │
│  │  CREATE  │───►│ WAIT_REVIEW  │───►│ FETCH_REVIEW│───►│  ANALYZE  │  │
│  │    PR    │    │  (polling)   │    │             │    │           │  │
│  └──────────┘    └──────────────┘    └─────────────┘    └─────┬─────┘  │
│                         ▲                                      │        │
│                         │                              ┌───────┴──────┐ │
│                         │                              ▼              ▼ │
│                  ┌──────┴─────┐    ┌──────────┐    ┌──────┐    ┌─────┐ │
│                  │    PUSH    │◄───│   FIX    │◄───│ISSUES│    │CLEAN│ │
│                  │   FIXES    │    │          │    │FOUND │    │     │ │
│                  └────────────┘    └──────────┘    └──────┘    └──┬──┘ │
│                                                                   │     │
│                                                            ┌──────▼───┐ │
│                                                            │  MERGE   │ │
│                                                            │    PR    │ │
│                                                            └──────────┘ │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## Phase 1: PR Creation & Initial Wait

### Step 1.1: Create PR (if not exists)

```bash
# AI creates PR using gh CLI or MCP tool
gh pr create --title "..." --body "..." --base main
```

- Record PR number and URL
- Log in task file: `* YYYY-MM-DD HH:MM: Created PR #XXX by [Agent]`

### Step 1.2: Initialize Polling State

```yaml
polling_state:
  pr_number: 123
  pr_url: "https://github.com/owner/repo/pull/123"
  created_at: "2024-01-15T10:00:00Z"
  iteration: 1
  max_iterations: 10  # Prevent infinite loops
  status: "WAIT_REVIEW"
  known_bots_pending: ["coderabbitai[bot]", "github-actions[bot]"]
  last_check: null
  fixes_applied: []
```

---

## Phase 2: Smart Polling for Bot Reviews

### Step 2.1: Polling Strategy

```
POLLING ALGORITHM:
─────────────────────────────────────────────────────
Time since PR created    │  Check interval
─────────────────────────────────────────────────────
0-2 minutes              │  30 seconds (bots starting)
2-5 minutes              │  45 seconds (primary window)
5-10 minutes             │  60 seconds (extended wait)
10+ minutes              │  90 seconds (long reviews)
─────────────────────────────────────────────────────
Max total wait: 15 minutes before alerting user
```

### Step 2.2: Check Bot Review Status

For each polling cycle:

1. **Fetch PR Status** using GitHub MCP:
   ```
   pull_request_read(method="get_review_comments", owner, repo, pullNumber)
   pull_request_read(method="get_status", owner, repo, pullNumber)
   ```

2. **Detect Bot Completion**:
   - Check if known bots have posted their review comments
   - Look for `completion_signals` in comment content
   - Verify CI checks have completed (not pending)

3. **Completion Criteria** (robust, chống “false complete”):

Bot review được coi là “complete” khi thỏa **một trong hai** điều kiện sau, và CI không còn pending:

**A) Required bots complete**
- Tất cả bots trong `bots.required` đã xuất hiện ít nhất 1 review/comment, và
- Không còn `pending_signals` trong nội dung mới nhất của bot (nếu bot có loại signal này), và
- CI/check runs đã kết thúc (success/failure), không “pending/queued/in_progress”

**B) Quiet-period fallback (khi bot comment theo đợt)**
- Đã qua `initial_wait_seconds`, và
- Không có comment/review mới trong `quiet_period_seconds` (mặc định 120s), và
- CI/check runs đã kết thúc (success/failure), không pending

```python
def is_review_complete():
    # Inputs:
    # - required_bots: from config
    # - latest_bot_messages: grouped by bot
    # - last_activity_at: last comment/review/check-update time
    # - ci_state: pending|complete
    #
    # return True if (A or B) and ci_state == complete
    pass
```

**Important**: Nếu `bots.optional` chưa xong nhưng required bots + CI đã complete thì vẫn được chuyển Phase 3/4, nhưng phải ghi log “optional bot pending” vào task + PR comment.

### Step 2.3: Polling Loop Implementation

```
WHILE status == "WAIT_REVIEW" AND iteration <= max_iterations:
    
    1. SLEEP(calculated_interval)
    
    2. FETCH PR comments and status via GitHub MCP
    
    3. CHECK for bot review completion:
       - Parse comments for known bot signatures
       - Check CI/CD status (pending vs complete)
    
    4. IF all bots have reviewed OR timeout reached:
       - SET status = "FETCH_REVIEW"
       - BREAK
    
    5. LOG: "Polling iteration {n}: Waiting for {pending_bots}..."
    
    6. INCREMENT iteration
```

---

## Phase 3: Fetch & Parse Reviews

### Step 3.0: Dedupe Strategy (tránh trùng issue giữa threads vs comments)

- Gộp issues theo key: `(source, file, line_start, line_end, normalized_description_hash)`
- Nếu cùng issue xuất hiện nhiều lần qua iterations: tăng `occurrence_count` để phục vụ “stop conditions” (Phase 6 safeguards)

### Step 3.1: Comprehensive Review Fetch

```
FETCH all review data:
├── PR review comments (line-specific)
├── PR conversation comments (general)
├── CI/CD check results
└── Bot-specific summaries
```

Use GitHub MCP tools:
- `pull_request_read(method="get_review_comments")` - Line comments
- `pull_request_read(method="get_comments")` - General comments  
- `pull_request_read(method="get_status")` - CI status
- `pull_request_read(method="get_reviews")` - Review approvals/requests

### Step 3.2: Parse Bot-Specific Formats

**CodeRabbit Format:**
```markdown
## Summary
[Overview of changes]

## Walkthrough  
[File-by-file analysis]

## Issues Found
- Critical: [description] (file:line)
- Warning: [description]
- Suggestion: [description]
```

**Greptile Format:**
```markdown
## Code Review by Greptile
### Issues
- [severity] file.rs:L10-15: description
```

### Step 3.3: Normalize Issues

Convert all bot formats to unified structure (có mapping severity thống nhất + tracking lặp):

#### Severity Normalization

Mỗi bot có cách đặt severity khác nhau. Normalize về 4 mức và dùng chúng xuyên suốt workflow:

- `Critical`: security, data integrity, migration correctness, auth, tenant isolation, panics/crashes
- `Warning`: logic/perf issues, correctness, missing tests cho critical path
- `Style`: lint/format/docs consistency
- `Nitpick`: cosmetic, naming, optional suggestions

Mapping gợi ý:
- `critical|high|blocker` → `Critical`
- `warning|medium` → `Warning`
- `low|style|lint|docs` → `Style`
- `nitpick|suggestion|minor` → `Nitpick`

Nếu bot không gán severity rõ: mặc định `Warning` (conservative).

```yaml
issues:
  - id: "issue_001"
    source: "CodeRabbit"
    severity: "Critical"  # Critical|Warning|Style|Nitpick
    file: "src/main.rs"
    line_start: 42
    line_end: 45
    description: "Missing error handling"
    suggested_fix: "Add Result<> wrapper"
    auto_fixable: true
    status: "pending"  # pending|fixed|skipped|manual|blocked
    occurrence_count: 1
    first_seen_iteration: 1
    last_seen_iteration: 1
```

---

## Phase 4: Analyze & Decide

### Step 4.0: Risk Guardrails (hard stop vs safe auto-fix)

Một số loại thay đổi cần bảo thủ để tránh “auto-fix gây breaking change”:

**Hard Stop (manual approval required)**
- Thay đổi authn/authz flow
- Thay đổi semantics business logic không có suggested fix rõ ràng
- Thay đổi migrations ảnh hưởng dữ liệu/constraints mà không có hướng dẫn cụ thể
- Thay đổi multi-tenant isolation (tenant_id filters, composite FK rules)
- Security issues cần quyết định policy

**Proceed with Caution**
- Perf fixes: chỉ auto-fix nếu có chứng cứ rõ (N+1) và thay đổi nhỏ, có test/benchmark tối thiểu
- Refactors lớn: không auto-fix, chuyển `manual`

Nếu issue thuộc “Hard Stop”:
- Mark `manual`
- Log rõ lý do + đề xuất hướng xử lý
- Dừng nếu severity >= `Critical` và chưa có người confirm


### Step 4.1: Fixability Assessment

```
FOR each issue:
    
    1. CHECK if already fixed in current code:
       - Read file content
       - Compare with issue description
       - Run local diagnostics (cargo check, clippy)
    
    2. CLASSIFY fixability:
       - AUTO_FIX: Clear, mechanical fixes (syntax, formatting, simple logic)
       - NEEDS_CONTEXT: Requires understanding broader architecture
       - MANUAL_ONLY: Business logic, security decisions, breaking changes
       - SKIP: Informational, resolved, or false positive
    
    3. PRIORITIZE by severity and effort:
       Priority = (Severity_Weight * 10) + (Ease_of_Fix * 5) - (Risk * 3)
```

### Step 4.2: Decision Matrix

| Issue Type     | Auto-Fix? | Confidence  | Action         |
|----------------|-----------|-------------|----------------|
| Syntax error   | Yes       | High        | Fix & commit   |
| Missing import | Yes       | High        | Fix & commit   |
| Unused code    | Yes       | Medium      | Fix & commit   |
| Logic error    | Maybe     | Low-Medium  | Analyze first  |
| Security issue | No        | N/A         | Flag for human |
| Architecture   | No        | N/A         | Flag for human |
| Style/format   | Yes       | High        | Fix & commit   |

### Step 4.3: Update Task File

Add issues to task file with proper tracking:

```markdown
### PR Review Issues (Iteration #1)
**Source**: CodeRabbit, GitHub Actions
**Fetched**: 2024-01-15 10:05:00

- [x] Fix missing DEFERRABLE constraint (Critical, CodeRabbit) - Fixed in commit abc123
- [ ] Optimize N+1 query (Warning, Greptile) - In Progress
- [ ] Add unit test for edge case (Suggestion, CodeRabbit) - Pending
- [~] Consider renaming variable (Nitpick, CodeRabbit) - Skipped (cosmetic only)
```

---

## Phase 5: Apply Fixes & Push

### Step 5.0: Task Claim Gate (MANDATORY - Multi-Agent Safe)

Trước khi apply bất kỳ fix nào:

1. Xác định task file liên quan trong `PROJECT_TRACKING/**/task_*.md`
2. Kiểm tra:
   - `Status: InProgress_By_[AgentName]`
   - `Assignee: [AgentName]`
   - All `Dependencies:` đều `Done`
3. Nếu không đạt:
   - Update task: `Status: Blocked_By_Task_Not_Claimed` (hoặc lý do phù hợp)
   - Ghi log: agent, PR, thời điểm, điều kiện fail
   - STOP (không sửa code)


### Step 5.1: Apply Fixes Sequentially

```
FOR each fixable_issue (sorted by priority):
    
    1. READ target file(s)
    
    2. APPLY fix using Edit/Write tools
    
    3. VALIDATE locally (MANDATORY QUALITY GATES):

       Rust (backend/services):
       - cargo fmt
       - cargo check --workspace
       - cargo clippy --workspace -- -D warnings
       - cargo test --workspace

       Frontend (nếu có):
       - bun run typecheck (hoặc equivalent)
       - bun run lint
       - bun run test

       Notes:
       - Nếu task chỉ chạm 1 crate/service, có thể chạy subset, nhưng phải ghi rõ trong task log + PR comment.
       - Không chuyển “NeedsReview/merge” nếu bất kỳ gate nào fail.
    
    4. IF validation fails:
       - REVERT change
       - MARK issue as "manual_required"
       - CONTINUE to next issue
    
    5. IF validation passes:
       - STAGE changes
       - MARK issue as "fixed"
       - LOG fix details
```

### Step 5.2: Commit & Push

```bash
# Single commit for all fixes in this iteration
git add -A
git commit -m "fix(pr-review): resolve bot review issues (iteration #N)

Fixed:
- [issue_001]: Missing DEFERRABLE constraint
- [issue_002]: Unused import removed

Skipped (manual review needed):
- [issue_003]: Security consideration

TaskID: XX.YY.ZZ"

git push origin <branch>
```

### Step 5.3: Update PR

Comment on PR to document fixes (audit-friendly):

```markdown
## Auto-Fix Report (Iteration #1)

**Commit:** <sha-or-link>
**Quality Gates:** fmt ✅ / check ✅ / clippy ✅ / test ✅  (hoặc list failures)

### Fixed
- [issue_001] (Critical, CodeRabbit): Missing DEFERRABLE on UNIQUE constraints (file:line)
- [issue_002] (Style, Greptile): Removed unused import (file:line)

### Skipped
- [issue_010] (Nitpick, CodeRabbit): Variable rename suggestion (cosmetic)

### Needs Manual Review / Approval
- [issue_003] (Critical, CodeRabbit): Security consideration for input validation
  - Reason: Hard Stop category (security policy decision)
  - Suggested next step: reviewer confirm desired validation approach

### Notes
- Optional bots pending: <list> (if any)
- Remaining known risks: <summary>

---
*Automated auto-fix. Next: re-poll after quiet period.*
```

---

## Phase 6: Re-iterate Until Clean

### Step 6.0: Iteration Stop Conditions (diminishing returns)

Ngoài `max_iterations`, dừng sớm và escalate khi:

- Số issue không giảm qua 2 iterations liên tiếp
- Cùng một issue (same normalized key) xuất hiện ≥ 3 lần (`same_issue_appears_3_times`)
- Fix mới làm CI fail 2 lần liên tiếp
- Tổng thời gian vượt `max_total_runtime`
- Xuất hiện `Critical` thuộc nhóm Hard Stop

Khi dừng:
- Cập nhật task: `Blocked_By_Escalation_[Reason]`
- Comment PR: tóm tắt issue còn lại + log gates + đề xuất bước tiếp theo


### Step 6.1: Loop Back to Polling

```
AFTER push:
    
    1. RESET polling state:
       - iteration += 1
       - status = "WAIT_REVIEW"
       - last_check = now()
    
    2. WAIT for bots to re-review (shorter wait - bots are faster on updates)
       - Initial wait: 60 seconds
       - Polling interval: 30 seconds
    
    3. FETCH new reviews
    
    4. IF new issues found:
       - REPEAT Phase 4-5
    
    5. IF no new issues AND all checks pass:
       - PROCEED to merge
```

### Step 6.2: Iteration Limits

```yaml
safeguards:
  max_iterations: 10
  max_fixes_per_iteration: 20
  max_total_runtime: 30 minutes
  quiet_period_seconds: 120
  no_progress_iterations: 2
  escalation_triggers:
    - same_issue_appears_3_times
    - ci_fails_after_fix_twice
    - security_issue_detected
    - critical_hard_stop_detected
    - no_progress_2_iterations
```

---

## Phase 7: Merge & Cleanup

### Step 7.1: Pre-Merge Checklist

- [ ] All bot reviews addressed (fixed or acknowledged)
- [ ] All CI checks passing
- [ ] No unresolved critical/warning issues
- [ ] At least one approval (if required by repo settings)
- [ ] Branch is up-to-date with base

### Step 7.2: Merge PR

```bash
# Using GitHub MCP
merge_pull_request(
    owner="...",
    repo="...",
    pullNumber=123,
    merge_method="squash",  # or "merge", "rebase" based on repo config
    commit_title="feat: [description] (#123)"
)
```

### Step 7.3: Post-Merge Actions

1. **Update Task File**:
   ```markdown
   ## Status: Done
   
   ### Completion Log
   * 2024-01-15 10:30: PR #123 merged successfully
   * Total iterations: 3
   * Issues fixed: 7
   * Issues skipped: 2
   * Total time: 12 minutes
   ```

2. **Move to Next Task**:
   - Check task dependencies
   - Update parent task progress
   - Begin next task in queue

---

## Polling Implementation Details

### Bash-based Polling (using background task)

```bash
# Start background polling
while true; do
    # Check PR status
    STATUS=$(gh pr view 123 --json reviews,statusCheckRollup)
    
    # Parse for completion
    if [[ "$STATUS" contains completion signals ]]; then
        echo "Reviews complete"
        break
    fi
    
    sleep 45
done
```

### AI Agent Polling Pattern

```
FUNCTION poll_for_reviews(pr_number, timeout_minutes=15):
    start_time = now()
    check_count = 0
    
    WHILE (now() - start_time) < timeout_minutes:
        check_count += 1
        
        # Fetch current state
        reviews = github_mcp.pull_request_read(method="get_review_comments", ...)
        status = github_mcp.pull_request_read(method="get_status", ...)
        
        # Check if bots have finished
        bot_reviews = filter_bot_reviews(reviews)
        ci_complete = all_checks_complete(status)
        
        IF bot_reviews.count >= expected_bots AND ci_complete:
            RETURN {status: "complete", reviews: bot_reviews}
        
        # Adaptive sleep
        sleep_time = calculate_interval(check_count, start_time)
        LOG(f"Waiting for reviews... (check #{check_count}, sleeping {sleep_time}s)")
        
        SLEEP(sleep_time)
    
    RETURN {status: "timeout", partial_reviews: bot_reviews}
```

---

## Error Handling & Edge Cases

### Timeout Handling

```
IF polling_timeout_reached:
    1. LOG warning in task file
    2. NOTIFY user: "Bot reviews taking longer than expected. Options:"
       - Continue waiting
       - Proceed with available reviews
       - Cancel and investigate
    3. AWAIT user decision (or auto-proceed if configured)
```

### CI Failure Handling

```
IF ci_checks_failed:
    1. FETCH failure logs
    2. ANALYZE if related to recent changes
    3. IF fixable:
       - Apply fix
       - Push
       - Re-enter polling loop
    4. IF not fixable:
       - ESCALATE to user
       - PAUSE automation
```

### Conflict Handling

```
IF merge_conflict_detected:
    1. FETCH conflicting files
    2. ATTEMPT auto-resolution (if simple)
    3. IF complex conflict:
       - NOTIFY user
       - PROVIDE conflict summary
       - AWAIT manual resolution
```

---

## Configuration Options

```yaml
# .claude/pr-autofix-config.yaml
pr_autofix:
  enabled: true
  
  polling:
    initial_wait_seconds: 120
    max_wait_minutes: 15
    check_interval_seconds: 45
    
  bots:
    required: ["coderabbitai[bot]"]
    optional: ["greptile[bot]"]
    
  auto_fix:
    enabled: true
    max_iterations: 10
    severity_threshold: "warning"  # Only fix warning+ by default
    skip_nitpicks: true
    
  merge:
    auto_merge: true
    require_ci_pass: true
    merge_method: "squash"
    
  notifications:
    on_fix: true
    on_skip: true
    on_error: true
    on_merge: true
```

---

## Quick Reference Commands

```bash
# Manual trigger
claude "Fix PR review issues for PR #123 and merge when clean"

# Check status
claude "What's the current status of PR #123 auto-fix?"

# Force re-check
claude "Re-fetch reviews for PR #123 and continue fixing"

# Skip and merge
claude "Skip remaining nitpicks on PR #123 and merge"
```

---

## Summary Flow Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                    COMPLETE AUTO-FIX FLOW                       │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  User: "Create PR and fix all issues"                          │
│           │                                                     │
│           ▼                                                     │
│  ┌─────────────────┐                                           │
│  │  1. Create PR   │                                           │
│  └────────┬────────┘                                           │
│           │                                                     │
│           ▼                                                     │
│  ┌─────────────────┐    ┌─────────────────┐                    │
│  │  2. Poll for    │───►│  3. Fetch &     │                    │
│  │     reviews     │    │     parse       │                    │
│  │  (2-5 min wait) │    │     reviews     │                    │
│  └─────────────────┘    └────────┬────────┘                    │
│                                  │                              │
│                                  ▼                              │
│                         ┌─────────────────┐                    │
│                         │  4. Analyze &   │                    │
│                         │     categorize  │                    │
│                         └────────┬────────┘                    │
│                                  │                              │
│                    ┌─────────────┴─────────────┐               │
│                    ▼                           ▼               │
│           ┌──────────────┐            ┌──────────────┐         │
│           │  No issues   │            │ Issues found │         │
│           │  (clean!)    │            │              │         │
│           └──────┬───────┘            └──────┬───────┘         │
│                  │                           │                  │
│                  │                           ▼                  │
│                  │                   ┌──────────────┐          │
│                  │                   │  5. Apply    │          │
│                  │                   │     fixes    │          │
│                  │                   └──────┬───────┘          │
│                  │                          │                   │
│                  │                          ▼                   │
│                  │                   ┌──────────────┐          │
│                  │                   │  6. Push &   │──────┐   │
│                  │                   │     comment  │      │   │
│                  │                   └──────────────┘      │   │
│                  │                                         │   │
│                  │                          ┌──────────────┘   │
│                  │                          │                   │
│                  │                          ▼                   │
│                  │                   ┌──────────────┐          │
│                  │                   │  Loop back   │          │
│                  │                   │  to step 2   │          │
│                  │                   └──────────────┘          │
│                  │                                              │
│                  ▼                                              │
│           ┌──────────────┐                                     │
│           │  7. MERGE!   │                                     │
│           └──────┬───────┘                                     │
│                  │                                              │
│                  ▼                                              │
│           ┌──────────────┐                                     │
│           │  8. Next     │                                     │
│           │     task     │                                     │
│           └──────────────┘                                     │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## Version History

| Version | Date       | Changes                                                    |
|---------|------------|------------------------------------------------------------|
| 1.0     | 2024-01-01 | Initial manual workflow                                    |
| 2.0     | 2024-01-15 | Added bot-aware polling, iterative fix loop, auto-merge   |

---

## Feature Comparison

| Feature                  | v1.0 (Manual) | v2.0 (Bot-Aware) |
|--------------------------|---------------|------------------|
| Wait for bot review      | Manual        | Auto polling     |
| Detect bot completion    | No            | Signal-based     |
| Iterative fixing         | One-shot      | Loop until clean |
| Auto merge               | No            | Yes              |
| Adaptive wait time       | No            | Smart intervals  |
| Bot format parsing       | Basic         | Multi-bot        |
| State machine            | No            | Full workflow    |
| Error recovery           | Basic         | Comprehensive    |
