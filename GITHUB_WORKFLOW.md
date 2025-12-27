# GitHub Flow Workflow - Comprehensive Guide

## 🚨 **MANDATORY: Always follow this workflow to prevent code loss**

This guide ensures **zero code loss** when multiple developers work on the same repository. Follow every step carefully.

---

## **1. Pre-Work Checklist (MANDATORY)**

### **Before starting ANY work:**

```bash
# 1. Check current status
git status

# 2. Stash any uncommitted changes (if any)
git stash push -m "work-in-progress"

# 3. Ensure you're on main and it's clean
git checkout main
git pull origin main

# 4. Verify no uncommitted changes
git status  # Should show "nothing to commit, working tree clean"

# 5. Start local Postgres and export DB env (required for cargo check/clippy/tests using sqlx macros)
docker compose -f infra/docker_compose/docker-compose.yml up -d postgres
export DATABASE_URL=postgres://user:password@localhost:5432/inventory_db
export TEST_DATABASE_URL=${TEST_DATABASE_URL:-$DATABASE_URL}
```

### **If you have uncommitted changes:**
- **NEVER** switch branches without stashing
- **ALWAYS** commit or stash before any branch operations
- Use descriptive stash messages: `git stash push -m "feat: user login form"`

---

## **2. Standard Development Flow**

### **Phase 1: Setup Feature Branch**

```bash
# 1. Ensure main is up-to-date
git checkout main
git pull origin main

# 2. Create feature branch with descriptive name
git checkout -b feature/123-user-authentication
# OR for bug fixes:
git checkout -b fix/456-login-validation
# OR for hotfixes:
git checkout -b hotfix/789-critical-security-patch

# 3. Verify branch creation
git branch  # Should show * next to your new branch
```

### **Phase 2: Development & Commits**

```bash
# 1. Make changes and stage them
git add .
# OR stage specific files:
git add frontend/src/components/Login.svelte
git add backend/src/auth.rs

# 2. Commit with conventional commit message
git commit -m "feat: implement user authentication with JWT"

# 3. Push to remote (first time use -u)
git push -u origin feature/123-user-authentication
# Subsequent pushes: git push
```

### **Phase 3: Sync with Main (CRITICAL - Do this frequently)**

```bash
# 1. Fetch latest changes
git fetch origin

# 2. Check if main has new commits
git log --oneline main..origin/main

# 3. If main has updates, merge them in
git merge origin/main
# OR use rebase for cleaner history:
git rebase origin/main

# 4. Push merged changes
git push origin feature/123-user-authentication
```

### **Phase 4: Create Pull Request**

1. **Go to GitHub** → Pull Requests → New Pull Request
2. **Select branches:**
   - Base: `main`
   - Compare: `feature/123-user-authentication`
3. **Fill PR details:**
   - **Title:** `feat: implement user authentication with JWT`
   - **Description:** 
     ```
     ## Changes
     - Added JWT authentication middleware
     - Created login/logout endpoints
     - Updated user session management

     ## Testing
     - Unit tests for auth logic
     - Integration tests for endpoints
     - Manual testing of login flow

     ## Related Issues
     Closes #123
     ```
4. **Add reviewers** and **labels**
5. **Create PR**

### **Phase 5: Handle PR Feedback**

```bash
# If reviewer requests changes:

# Option 1: Amend last commit (if small changes)
git add .
git commit --amend --no-edit
git push --force-with-lease origin feature/123-user-authentication

# Option 2: Add new commit (if significant changes)
git add .
git commit -m "fix: address PR feedback on error handling"
git push origin feature/123-user-authentication
```

### **Phase 6: Merge PR**

- **Wait for CI to pass** ✅
- **Get required approvals** ✅
- **Merge using "Squash and merge"** (recommended)
- **Delete branch** (GitHub does this automatically if enabled)

---

## **3. Conflict Resolution (When main changes during your work)**

### **Scenario: Main has new commits while you're working**

```bash
# 1. Stash your current work
git stash push -m "work-before-merge"

# 2. Update your branch with main
git checkout main
git pull origin main
git checkout feature/123-user-authentication
git merge main  # OR: git rebase main

# 3. Restore your work
git stash pop

# 4. Resolve any conflicts if they occur
# Edit conflicted files, then:
git add <resolved-files>
git commit -m "fix: resolve merge conflicts with main"

# 5. Push updated branch
git push origin feature/123-user-authentication
```

### **If conflicts are complex:**

```bash
# Abort merge/rebase and try different approach
git merge --abort
# OR
git rebase --abort

# Then try the other method:
git rebase main  # if merge failed
# OR
git merge main   # if rebase failed
```

---

## **4. Branch Management & Cleanup**

### **After PR is merged:**

```bash
# 1. Update local main
git checkout main
git pull origin main

# 2. Delete local branch safely
git branch -d feature/123-user-authentication
# Use -D if branch wasn't merged (force delete)

# 3. Clean up remote branches (optional)
git fetch --prune
```

### **Bulk cleanup of merged branches:**

```bash
# See all merged branches
git branch --merged

# Delete all merged branches except main
git branch --merged | grep -v "main\|master\|develop" | xargs git branch -d

# Clean up remote tracking branches
git fetch --prune
```

---

## **5. Advanced Scenarios**

### **Working on Multiple Features:**

```bash
# Create separate branches for each feature
git checkout -b feature/123-auth
git checkout -b feature/124-profile

# Switch between them safely
git stash push -m "auth-work"
git checkout feature/124-profile
git stash pop
```

### **Hotfix Workflow:**

```bash
# 1. Create hotfix from main
git checkout main
git pull origin main
git checkout -b hotfix/critical-bug

# 2. Fix the bug
git add .
git commit -m "fix: critical security vulnerability in auth"

# 3. Push and create urgent PR
git push -u origin hotfix/critical-bug

# 4. After merge, update main
git checkout main
git pull origin main
```

### **Release Branch Workflow:**

```bash
# 1. Create release branch from main
git checkout main
git pull origin main
git checkout -b release/v1.2.0

# 2. Final testing and version bumps
git add .
git commit -m "chore: bump version to 1.2.0"

# 3. Merge release to main and tag
git checkout main
git merge release/v1.2.0
git tag v1.2.0
git push origin main --tags
```

---

## **6. Emergency Recovery**

### **If you accidentally lose commits:**

```bash
# 1. Check reflog for lost commits
git reflog

# 2. Recover lost commit
git checkout <commit-hash>
git checkout -b recovery-branch

# 3. Cherry-pick important commits
git cherry-pick <important-commit>
```

### **If remote branch is corrupted:**

```bash
# 1. Backup local work
git branch backup-$(date +%Y%m%d-%H%M%S)

# 2. Reset to safe state
git reset --hard origin/main

# 3. Recreate branch
git checkout -b feature/new-branch
```

---

## **7. Best Practices & Rules**

### **❌ NEVER DO:**
- **Force push to main/master** (`git push --force`)
- **Delete branches without checking** (`git branch -D`)
- **Work directly on main** (always use feature branches)
- **Commit without testing** (run tests before commit)
- **Push untested code** (CI should pass)

### **✅ ALWAYS DO:**
- **Pull before starting work** (`git pull origin main`)
- **Create descriptive branch names** (`feature/123-user-auth`)
- **Write clear commit messages** (`feat: add user login`)
- **Sync with main frequently** (at least daily)
- **Test before pushing** (`npm run test`, `cargo test`)
- **Review your own code** before creating PR

### **Branch Naming Convention:**
```
feature/123-user-authentication
fix/456-validation-bug
hotfix/789-security-patch
chore/101-update-dependencies
docs/202-api-documentation
refactor/303-code-cleanup
```

### **Commit Message Format:**
```
type(scope): description

[optional body]

[optional footer]
```

**Types:** `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`

**Examples:**
```
feat(auth): implement JWT token validation
fix(ui): resolve mobile layout overflow
docs(api): update endpoint documentation
test(auth): add integration tests for login
```

---

## **8. Team Collaboration Rules**

### **Communication:**
- **Always communicate** when working on shared components
- **Use GitHub Issues** for feature requests and bug reports
- **Tag reviewers** in PR descriptions
- **Update PR status** regularly

### **Code Review:**
- **Review your own code** first
- **Test PR changes** locally before approving
- **Provide constructive feedback**
- **Approve only tested code**

### **Repository Settings:**
- **Require PR reviews** before merge
- **Require status checks** (CI) to pass
- **Enable branch protection** on main
- **Auto-delete branches** after merge

---

## **9. Quick Reference (Cheat Sheet)**

### **Daily Workflow:**
```bash
git checkout main && git pull          # Start day
git checkout -b feature/xyz            # New work
git add . && git commit -m "feat: xyz" # Code & commit
git push -u origin feature/xyz         # Push
# Create PR on GitHub
```

### **Sync with Team:**
```bash
git fetch origin                       # Get latest
git log main..origin/main              # Check changes
git merge origin/main                  # Sync
git push                               # Update remote
```

### **Cleanup:**
```bash
git checkout main && git pull          # Update main
git branch --merged | xargs git branch -d  # Delete merged
git fetch --prune                      # Clean remotes
```

---

## **10. Troubleshooting**

### **"Your branch is behind origin/main"**
```bash
git fetch origin
git merge origin/main
# OR: git rebase origin/main
```

### **"Merge conflict"**
1. Edit conflicted files
2. `git add <resolved-files>`
3. `git commit -m "fix: resolve merge conflicts"`

### **"Push rejected"**
```bash
git pull --rebase origin <branch-name>
git push
```

### **Lost commits**
```bash
git reflog
git checkout <commit-hash>
git checkout -b recovery-branch
```

---

**REMEMBER: This workflow guarantees ZERO CODE LOSS when followed correctly. Always sync with main frequently and never force push to shared branches!**
