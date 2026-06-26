<!-- rtk-instructions v2 -->
# RTK (Rust Token Killer) - Token-Optimized Commands

## Golden Rule

**Always prefix commands with `rtk`**. If RTK has a dedicated filter, it uses it. If not, it passes through unchanged. This means RTK is always safe to use.

**Important**: Even in command chains with `&&`, use `rtk`:
```bash
# ❌ Wrong
git add . && git commit -m "msg" && git push

# ✅ Correct
rtk git add . && rtk git commit -m "msg" && rtk git push
```

## RTK Commands by Workflow

### Build & Compile (80-90% savings)
```bash
rtk cargo build         # Cargo build output
rtk cargo check         # Cargo check output
rtk cargo clippy        # Clippy warnings grouped by file (80%)
rtk tsc                 # TypeScript errors grouped by file/code (83%)
rtk lint                # ESLint/Biome violations grouped (84%)
rtk prettier --check    # Files needing format only (70%)
rtk next build          # Next.js build with route metrics (87%)
```

### Test (60-99% savings)
```bash
rtk cargo test          # Cargo test failures only (90%)
rtk go test             # Go test failures only (90%)
rtk jest                # Jest failures only (99.5%)
rtk vitest              # Vitest failures only (99.5%)
rtk playwright test     # Playwright failures only (94%)
rtk pytest              # Python test failures only (90%)
rtk rake test           # Ruby test failures only (90%)
rtk rspec               # RSpec test failures only (60%)
rtk test <cmd>          # Generic test wrapper - failures only
```

### Git (59-80% savings)
```bash
rtk git status          # Compact status
rtk git log             # Compact log (works with all git flags)
rtk git diff            # Compact diff (80%)
rtk git show            # Compact show (80%)
rtk git add             # Ultra-compact confirmations (59%)
rtk git commit          # Ultra-compact confirmations (59%)
rtk git push            # Ultra-compact confirmations
rtk git pull            # Ultra-compact confirmations
rtk git branch          # Compact branch list
rtk git fetch           # Compact fetch
rtk git stash           # Compact stash
rtk git worktree        # Compact worktree
```

Note: Git passthrough works for ALL subcommands, even those not explicitly listed.

### GitHub (26-87% savings)
```bash
rtk gh pr view <num>    # Compact PR view (87%)
rtk gh pr checks        # Compact PR checks (79%)
rtk gh run list         # Compact workflow runs (82%)
rtk gh issue list       # Compact issue list (80%)
rtk gh api              # Compact API responses (26%)
```

### JavaScript/TypeScript Tooling (70-90% savings)
```bash
rtk pnpm list           # Compact dependency tree (70%)
rtk pnpm outdated       # Compact outdated packages (80%)
rtk pnpm install        # Compact install output (90%)
rtk npm run <script>    # Compact npm script output
rtk npx <cmd>           # Compact npx command output
rtk prisma              # Prisma without ASCII art (88%)
```

### Files & Search (60-75% savings)
```bash
rtk ls <path>           # Tree format, compact (65%)
rtk read <file>         # Code reading with filtering (60%)
rtk grep <pattern>      # Search grouped by file (75%). Format flags (-c, -l, -L, -o, -Z) run raw.
rtk find <pattern>      # Find grouped by directory (70%)
```

### Analysis & Debug (70-90% savings)
```bash
rtk err <cmd>           # Filter errors only from any command
rtk log <file>          # Deduplicated logs with counts
rtk json <file>         # JSON structure without values
rtk deps                # Dependency overview
rtk env                 # Environment variables compact
rtk summary <cmd>       # Smart summary of command output
rtk diff                # Ultra-compact diffs
```

### Infrastructure (85% savings)
```bash
rtk docker ps           # Compact container list
rtk docker images       # Compact image list
rtk docker logs <c>     # Deduplicated logs
rtk kubectl get         # Compact resource list
rtk kubectl logs        # Deduplicated pod logs
```

### Network (65-70% savings)
```bash
rtk curl <url>          # Compact HTTP responses (70%)
rtk wget <url>          # Compact download output (65%)
```

### Meta Commands
```bash
rtk gain                # View token savings statistics
rtk gain --history      # View command history with savings
rtk discover            # Analyze Claude Code sessions for missed RTK usage
rtk proxy <cmd>         # Run command without filtering (for debugging)
rtk init                # Add RTK instructions to CLAUDE.md
rtk init --global       # Add RTK to ~/.claude/CLAUDE.md
```

## Token Savings Overview

| Category | Commands | Typical Savings |
|----------|----------|-----------------|
| Tests | vitest, playwright, cargo test | 90-99% |
| Build | next, tsc, lint, prettier | 70-87% |
| Git | status, log, diff, add, commit | 59-80% |
| GitHub | gh pr, gh run, gh issue | 26-87% |
| Package Managers | pnpm, npm, npx | 70-90% |
| Files | ls, read, grep, find | 60-75% |
| Infrastructure | docker, kubectl | 85% |
| Network | curl, wget | 65-70% |

Overall average: **60-90% token reduction** on common development operations.
<!-- /rtk-instructions -->

---

# Graphify Rules

## What is Graphify
Graphify converts project content (code, docs, images) into a persistent knowledge graph with clustered communities, HTML visualization, and audit reports.

## Pre-PR / Pre-Push Knowledge Graph Update

**CRITICAL**: Before every PR or push to main, update the graphify knowledge map:

```bash
# Run graphify to refresh the knowledge graph
/graphify
```

This ensures the knowledge graph reflects the latest project state for:
- Architecture documentation
- Code relationship mapping
- Community detection for module organization
- Audit trail of project evolution

## When to Update the Knowledge Graph
- Before creating a pull request
- Before pushing to main/master
- After significant code changes
- After adding/removing major features
- After restructuring the codebase

## Graphify Output
Output is stored in `kick-graphify/` and includes:
- `knowledge-graph.html` - Interactive visualization
- `knowledge-graph.json` - Machine-readable graph data
- `audit-report.md` - Change summary and analysis

**Note**: Graphify CLI commands default to `graphify-out/`. For this project, always pass `--graph kick-graphify/graph.json`:
```bash
graphify path "A" "B" --graph kick-graphify/graph.json
graphify explain "X" --graph kick-graphify/graph.json
```

---

# Git Workflow Rules

## Commit After Edit
**CRITICAL**: After editing any file, you MUST commit that file immediately when the change is complete.

```bash
# After editing a file
rtk git add <edited-file>
rtk git commit -m "descriptive message about the change"
```

Do NOT batch unrelated changes into a single commit. Each logical change should be its own commit.

## Folder-Wise Pull Requests
**CRITICAL**: When creating PRs, organize them by folder/feature. Do NOT combine unrelated changes into a single PR.

Why:
- PR titles should accurately describe all changes in the PR
- Reviewers can focus on specific areas
- Easier to revert specific changes if needed
- Cleaner git history

Example:
```
# ❌ Wrong - mixing unrelated changes
PR: "Update dashboard and fix typo in README"

# ✅ Correct - separate PRs by folder/feature
PR 1: "feat(dashboard): add weekly heatmap component"
PR 2: "docs: fix typo in README"
```

When to split PRs:
- Changes in different folders (`src/components/` vs `src-tauri/`)
- Different feature areas (UI vs backend vs docs)
- Mix of bug fixes and new features
- Configuration changes mixed with code changes

---

# Custom Skills

## /pr - Automated PR Workflow
```bash
/pr                           # Auto-generate branch name and PR title
/pr <branch-name>             # Use custom branch name
/pr <branch-name> <title>     # Use custom branch name and PR title
```

Performs: commit → branch → push → create PR → merge to main → delete branch → sync local
