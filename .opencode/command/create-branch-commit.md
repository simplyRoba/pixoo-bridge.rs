---
description: Create a feature branch from main and commit changes with a conventional message without pushing.
---
<CreateBranch>
  $ARGUMENTS
</CreateBranch>

**Guardrails**
- Always sync with the latest `main` before creating a branch and keep commands minimal.
- Favor the most straightforward branching workflow; do not introduce extra work beyond staging and committing the requested files.
- Use a conventional commit message that reflects the change purpose (e.g., `build: ...`, `fix: ...`).
- Stop without pushing—branch and commit remain local unless an explicit push request follows.

**Steps**
1. Run `git switch main` and attempt `git pull` to ensure the base branch is current—if pull fails (e.g., due to auth), proceed anyway after noting the state.
2. Create the feature branch using a descriptive name aligned to the change (for example `feat/target-install`).
3. Stage the updated files (e.g., `git add <files>`).
4. Commit with the agreed conventional commit message, such as `git commit -m "build: install amd64 target before build"`.
5. Confirm `git status` shows a clean working tree on the new branch and do not push.

**Reference**
- Use `git status` and `git log -1` to verify the branch contains the intended changes.
