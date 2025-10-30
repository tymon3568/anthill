# 🎯 Cách Setup Prompt System - Hướng Dẫn Hoàn Chỉnh

## ❓ Câu Hỏi: "AI Agent biết load các file như thế nào?"

### ✅ Trả Lời:

**GitHub Copilot TỰ ĐỘNG load** tất cả `.prompt.md` files từ:
```
~/.config/Code/User/prompts/
```

**KHÔNG cần** user prompt gì cả! Khi bạn bắt đầu chat mới, Copilot đã load sẵn.

## 📦 3 Cách Setup (Chọn 1)

### 🥇 CÁCH 1: Enhanced general.prompt.md (KHUYẾN NGHỊ)

**Tại sao tốt nhất:**
- ✅ Tự động load mỗi session mới
- ✅ Không cần copy/paste gì
- ✅ AI tự biết phải làm gì
- ✅ Bao gồm tất cả instructions cần thiết

**Làm thế nào:**

1. **Thay thế file hiện tại**:
   ```bash
   cp docs/GENERAL_PROMPT_ENHANCED.md \
      ~/.config/Code/User/prompts/general.prompt.md
   ```

2. **Xong!** Bắt đầu chat mới, AI sẽ tự:
   - Load tất cả prompt files
   - Fetch Context7 docs
   - Check TASKS_OVERVIEW.md
   - Sẵn sàng làm việc

3. **Test**: Mở chat mới và gõ "hello", AI sẽ confirm như này:
   ```
   ✅ Context loaded: Anthill + folder-tasks workflow
   📊 Current phase: 03_User_Service (24 tasks)
   🎯 Ready to assist. How can I help?
   ```

### 🥈 CÁCH 2: Session Start Template (Dự phòng)

**Khi nào dùng:**
- Muốn control chính xác khi nào AI load context
- Đang test hoặc debug workflow
- Làm việc với nhiều projects khác nhau

**Setup:**

1. **Lưu template này** vào file text hoặc clipboard manager:
   ```markdown
   I'm working on Anthill project. Please:
   
   1. Confirm loaded: folder-tasks-workflow.prompt.md + anthill-project.prompt.md
   2. Fetch: Context7 /tymon3568/folder-tasks
   3. Check: PROJECT_TRACKING/TASKS_OVERVIEW.md
   4. Show: Current phase, available tasks, valid status values
   ```

2. **Copy/paste** vào đầu mỗi chat session mới

3. **AI sẽ respond** với confirmation và status

### 🥉 CÁCH 3: VS Code Workspace Settings

**Khi nào dùng:**
- Team collaboration (share settings)
- Consistent behavior across team members
- Enterprise setups

**Setup:**

1. Add to `.vscode/settings.json`:
   ```json
   {
     "github.copilot.chat.codeGeneration.instructions": [
       {
         "text": "Follow folder-tasks workflow from ~/.config/Code/User/prompts/folder-tasks-workflow.prompt.md"
       },
       {
         "text": "Use Anthill project standards from ~/.config/Code/User/prompts/anthill-project.prompt.md"
       },
       {
         "text": "Always fetch latest docs: Context7 /tymon3568/folder-tasks"
       }
     ]
   }
   ```

2. Restart VS Code

3. Settings apply to all chat sessions in this workspace

## 🎯 Recommended Workflow

**Chọn CÁCH 1** vì:

### File Structure:
```
~/.config/Code/User/prompts/
├── general.prompt.md                ← Master (use enhanced version)
├── folder-tasks-workflow.prompt.md  ← Workflow rules (auto-loaded)
├── anthill-project.prompt.md        ← Project context (auto-loaded)
└── README.md                         ← Documentation (not auto-loaded)
```

### Quy Trình:

1. **Một lần setup**:
   ```bash
   # Copy enhanced version
   cp docs/GENERAL_PROMPT_ENHANCED.md \
      ~/.config/Code/User/prompts/general.prompt.md
   ```

2. **Mỗi session mới**:
   - Mở VS Code
   - Bắt đầu chat với Copilot
   - AI tự động có full context
   - Gõ "find next task" và bắt đầu làm việc!

3. **Không cần**:
   - ❌ Copy/paste prompt
   - ❌ Nhắc AI load files
   - ❌ Restart VS Code
   - ❌ Bất kỳ manual step nào!

## 🧪 Test Setup Của Bạn

Sau khi setup, test bằng chat mới:

### Test 1: Context Loading
```
User: "Hello"

AI should respond:
✅ Context loaded: Anthill + folder-tasks workflow
📊 Current phase: [reads from TASKS_OVERVIEW.md]
🎯 Ready to assist...
```

### Test 2: Task Commands
```
User: "find next task"

AI should:
1. Search PROJECT_TRACKING/
2. Check dependencies
3. Propose available task
4. Ask if you want to claim it
```

### Test 3: Status Knowledge
```
User: "What are the valid status values?"

AI should list exactly 5:
- Todo
- InProgress_By_[Agent]
- Blocked_By_[Reason]
- NeedsReview
- Done
```

### Test 4: Context7 Integration
```
User: "Show me the latest folder-tasks workflow"

AI should:
1. Use mcp_upstash_conte tools
2. Fetch from /tymon3568/folder-tasks
3. Show relevant docs
```

## 🐛 Troubleshooting

### Problem: AI không biết về folder-tasks

**Solution:**
```bash
# Verify files exist
ls -la ~/.config/Code/User/prompts/

# Should see:
# - general.prompt.md
# - folder-tasks-workflow.prompt.md
# - anthill-project.prompt.md

# If missing, copy from project:
cp docs/GENERAL_PROMPT_ENHANCED.md ~/.config/Code/User/prompts/general.prompt.md
```

### Problem: AI không tự động check TASKS_OVERVIEW.md

**Solution:**
Enhanced `general.prompt.md` includes this instruction. Make sure you're using the enhanced version.

### Problem: AI dùng sai status values

**Solution:**
The enhanced prompt has STRICT rules section. AI should never use custom values. If it does, remind:
```
"Please use ONLY these status values:
Todo, InProgress_By_Claude, Blocked_By_[Reason], NeedsReview, Done"
```

## 📝 Maintenance

### Updating Prompts

Khi cần update instructions:

1. **Edit file locally**:
   ```bash
   code ~/.config/Code/User/prompts/general.prompt.md
   ```

2. **Changes apply immediately** - no need to restart VS Code

3. **Commit to project** (optional):
   ```bash
   cp ~/.config/Code/User/prompts/general.prompt.md \
      docs/GENERAL_PROMPT_ENHANCED.md
   
   git add docs/GENERAL_PROMPT_ENHANCED.md
   git commit -m "docs: update AI agent instructions"
   ```

### Syncing Across Machines

If working on multiple computers:

```bash
# On machine 1 (after setup)
cd ~
tar -czf copilot-prompts.tar.gz .config/Code/User/prompts/

# Copy to machine 2, then:
cd ~
tar -xzf copilot-prompts.tar.gz
```

Or use dotfiles repo to sync automatically.

## 🎓 Training New Team Members

Share this checklist:

```markdown
□ Install VS Code + GitHub Copilot
□ Clone anthill repo
□ Copy enhanced prompt files:
  cp docs/GENERAL_PROMPT_ENHANCED.md \
     ~/.config/Code/User/prompts/general.prompt.md
□ Test with "hello" in chat - should see confirmation
□ Try "find next task" - should work automatically
□ Read PROJECT_TRACKING/README.md for quick reference
□ Done! Start working on tasks
```

## 💡 Pro Tips

### Tip 1: Test Context Loading
At start of session, ask AI:
```
"What project are we working on and what workflow system do we use?"
```

Should get correct answer about Anthill + folder-tasks.

### Tip 2: Quick Status Check
Ask AI anytime:
```
"Show current task status"
```

AI should check task file and show progress.

### Tip 3: Helper Script
Use alongside AI:
```bash
# You: Find tasks
./scripts/task-helper.sh find 03_User_Service

# AI: Claim and work on task
"claim task_03.03.05"
```

### Tip 4: Validate AI Work
After AI updates task file, verify:
```bash
# Check status value is valid
grep "**Status:**" PROJECT_TRACKING/.../task_XX.YY.ZZ_*.md

# Should be one of: Todo, InProgress_By_Claude, Blocked_By_*, NeedsReview, Done
```

## ✅ Final Checklist

Before starting work, ensure:

- [ ] Enhanced general.prompt.md is in ~/.config/Code/User/prompts/
- [ ] folder-tasks-workflow.prompt.md is in place
- [ ] anthill-project.prompt.md is in place
- [ ] Tested: AI responds with context confirmation
- [ ] Tested: AI can find tasks
- [ ] Tested: AI knows valid status values
- [ ] Helper script works: ./scripts/task-helper.sh phases

If all ✅, you're ready! AI will work smoothly with folder-tasks system.

## 🚀 Quick Start Command

Copy/paste này vào terminal để setup everything:

```bash
cd /home/arch/Project/test/anthill

# Copy enhanced prompt (METHOD 1 - RECOMMENDED)
cp docs/GENERAL_PROMPT_ENHANCED.md \
   ~/.config/Code/User/prompts/general.prompt.md

# Test helper script
./scripts/task-helper.sh phases

# Start new Copilot chat and say "hello"
# AI should confirm context loaded!

echo "✅ Setup complete! Start a new Copilot chat to test."
```

**That's it!** 🎉
