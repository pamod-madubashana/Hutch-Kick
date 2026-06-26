// pr command plugin for OpenCode
// Handles /pr command for automated PR workflow
import { execSync } from "child_process";

export const PrCommandPlugin = async ({ directory }) => {
  return {
    "tool.execute.before": async (input, output) => {
      if (input.tool !== "bash") return;

      const command = output.args.command || "";
      if (!command.trim().startsWith("/pr")) return;

      // Parse arguments
      const args = command.trim().replace("/pr", "").trim();
      const parts = args.split(/\s+/);
      const branchName = parts[0] || null;
      const prTitle = parts.slice(1).join(" ") || null;

      console.log("[pr] Starting automated PR workflow...");

      // Get current changes for auto-naming
      let statusOutput = "";
      try {
        statusOutput = execSync("rtk git status --short", { encoding: "utf8" });
      } catch (e) {
        console.log("[pr] Error getting status:", e.message);
        return;
      }

      // Auto-generate branch name if not provided
      let finalBranchName = branchName;
      if (!finalBranchName) {
        const files = statusOutput.split("\n").filter(Boolean);
        const hasSrcTs = files.some(f => f.includes("src/") && (f.endsWith(".ts") || f.endsWith(".tsx")));
        const hasRust = files.some(f => f.includes("src-tauri/") && f.endsWith(".rs"));
        const hasDocs = files.some(f => f.endsWith(".md") || f.includes("docs/"));
        const hasConfig = files.some(f => f.includes("package.json") || f.includes("Cargo.toml"));

        if (hasRust) {
          finalBranchName = "fix/connectivity-" + Date.now().toString(36);
        } else if (hasSrcTs) {
          finalBranchName = "feat/ui-" + Date.now().toString(36);
        } else if (hasDocs) {
          finalBranchName = "docs/update-" + Date.now().toString(36);
        } else if (hasConfig) {
          finalBranchName = "chore/deps-" + Date.now().toString(36);
        } else {
          finalBranchName = "changes-" + Date.now().toString(36);
        }
      }

      // Auto-generate PR title if not provided
      let finalPrTitle = prTitle;
      if (!finalPrTitle) {
        if (statusOutput.includes("src-tauri/")) {
          finalPrTitle = "fix: update backend connectivity check";
        } else if (statusOutput.includes("src/")) {
          finalPrTitle = "feat: update UI components";
        } else if (statusOutput.includes(".md")) {
          finalPrTitle = "docs: update documentation";
        } else {
          finalPrTitle = "chore: project updates";
        }
      }

      console.log(`[pr] Branch: ${finalBranchName}`);
      console.log(`[pr] Title: ${finalPrTitle}`);

      // Step 1: Commit changes if any
      if (statusOutput.trim()) {
        console.log("[pr] Committing changes...");
        try {
          execSync("rtk git add .", { stdio: "inherit" });
          execSync(`rtk git commit -m "${finalPrTitle}"`, { stdio: "inherit" });
        } catch (e) {
          console.log("[pr] Commit failed:", e.message);
          return;
        }
      }

      // Step 2: Create and push branch
      console.log("[pr] Creating branch...");
      try {
        execSync("rtk git checkout main", { stdio: "inherit" });
        execSync("rtk git pull hutch-kick main", { stdio: "inherit" });
        execSync(`rtk git checkout -b ${finalBranchName}`, { stdio: "inherit" });
        execSync(`rtk git push -u hutch-kick ${finalBranchName}`, { stdio: "inherit" });
      } catch (e) {
        console.log("[pr] Branch creation failed:", e.message);
        return;
      }

      // Step 3: Create PR
      console.log("[pr] Creating PR...");
      try {
        execSync(`rtk gh pr create --title "${finalPrTitle}" --body "Automated PR from /pr command"`, { stdio: "inherit" });
      } catch (e) {
        console.log("[pr] PR creation failed:", e.message);
        return;
      }

      // Step 4: Merge PR
      console.log("[pr] Merging PR...");
      try {
        execSync("rtk gh pr merge --squash --delete-branch", { stdio: "inherit" });
      } catch (e) {
        console.log("[pr] Merge failed:", e.message);
        return;
      }

      // Step 5: Sync local
      console.log("[pr] Syncing local...");
      try {
        execSync("rtk git checkout main", { stdio: "inherit" });
        execSync("rtk git pull hutch-kick main", { stdio: "inherit" });
        execSync(`rtk git branch -d ${finalBranchName}`, { stdio: "inherit" });
      } catch (e) {
        console.log("[pr] Sync failed:", e.message);
        return;
      }

      console.log("[pr] PR workflow completed successfully!");

      // Replace the command with a success message
      output.args.command = 'echo "[pr] PR workflow completed successfully!"';
    },
  };
};
