---
name: pr
description: "Create PR, merge to main, clean up branch, and sync local code. Trigger: /pr"
trigger: /pr
---

# /pr

Automated PR workflow: create branch, push, create PR, merge to main, clean up, and sync.

## Usage

```
/pr                           # Auto-generate branch name and PR title from changes
/pr <branch-name>             # Use custom branch name
/pr <branch-name> <title>     # Use custom branch name and PR title
```

## What it does

1. **Stage and commit** all uncommitted changes
2. **Create branch** from main (auto-named or custom)
3. **Push branch** to remote
4. **Create PR** with descriptive title and body
5. **Merge PR** to main (squash merge)
6. **Delete remote branch**
7. **Pull main** to sync local code
8. **Clean up** local branch

## Workflow Steps

### Step 1 - Check for changes

```bash
rtk git status
rtk git diff --stat
rtk git log --oneline -3
```

If no changes to commit, skip to Step 2 with existing commits.

### Step 2 - Commit changes

```bash
rtk git add .
rtk git commit -m "<descriptive message>"
```

Generate a meaningful commit message based on the changes.

### Step 3 - Create and push branch

```bash
rtk git checkout main
rtk git pull hutch-kick main
rtk git checkout -b <branch-name>
rtk git push -u hutch-kick <branch-name>
```

Branch naming conventions:
- `feat/<name>` - new features
- `fix/<name>` - bug fixes
- `docs/<name>` - documentation
- `chore/<name>` - maintenance

### Step 4 - Create PR

```bash
rtk gh pr create --title "<title>" --body "<body>"
```

PR body should include:
- Summary of changes
- What was changed and why
- Any breaking changes

### Step 5 - Merge PR

```bash
rtk gh pr merge --squash --delete-branch
```

### Step 6 - Sync local

```bash
rtk git checkout main
rtk git pull hutch-kick main
```

### Step 7 - Clean up

```bash
rtk git branch -d <branch-name>
```

## Auto-naming Logic

If no branch name provided, analyze changes:

| Files changed | Branch prefix | Example |
|---------------|---------------|---------|
| `src/**/*.tsx`, `src/**/*.ts` | `feat/` or `fix/` | `feat/ui-update` |
| `src-tauri/**/*.rs` | `feat/` or `fix/` | `fix/connectivity-check` |
| `*.md`, `docs/` | `docs/` | `docs/update-readme` |
| `package.json`, `Cargo.toml` | `chore/` | `chore/deps-update` |

## Error Handling

- If push fails: check remote permissions
- If PR creation fails: branch may already exist, try different name
- If merge fails: check for conflicts, resolve manually
- If pull fails: stash local changes, pull, then pop stash

## Notes

- Uses `hutch-kick` as remote name (not `origin`)
- Uses squash merge for clean history
- Always syncs local after merge
- Deletes remote branch after merge
