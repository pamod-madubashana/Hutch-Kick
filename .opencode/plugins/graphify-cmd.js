// graphify command plugin for OpenCode
// Handles /graphify command to run the graphify pipeline
import { existsSync } from "fs";
import { join } from "path";
import { execSync } from "child_process";

export const GraphifyCommandPlugin = async ({ directory }) => {
  return {
    "tool.execute.before": async (input, output) => {
      if (input.tool !== "bash") return;

      const command = output.args.command || "";
      if (!command.trim().startsWith("/graphify")) return;

      // Parse arguments
      const args = command.trim().replace("/graphify", "").trim();
      const path = args || ".";

      console.log(`[graphify] Running graphify on: ${path}`);

      try {
        // Check if graphify is installed
        execSync("python3 -c \"import graphify\"", { stdio: "ignore" });
      } catch {
        console.log("[graphify] Installing graphify...");
        try {
          execSync("pip install graphifyy -q", { stdio: "inherit" });
        } catch {
          execSync("pip3 install graphifyy -q", { stdio: "inherit" });
        }
      }

      // Run graphify pipeline
      const graphifyScript = `
import json
from pathlib import Path
from graphify.detect import detect
from graphify.extract import collect_files, extract
from graphify.build import build_from_json
from graphify.cluster import cluster, score_all
from graphify.analyze import god_nodes, surprising_connections
from graphify.report import generate
from graphify.export import to_json, to_html
import networkx as nx

# Step 1: Detect files
print("[graphify] Detecting files...")
result = detect(Path('${path}'))
print(f"[graphify] Found {result.get('total_files', 0)} files")

# Step 2: Extract from code files
code_files = []
for f in result.get('files', {}).get('code', []):
    p = Path(f)
    if p.is_dir():
        code_files.extend(collect_files(p))
    else:
        code_files.append(p)

if code_files:
    print(f"[graphify] Extracting from {len(code_files)} code files...")
    ast_result = extract(code_files)
    print(f"[graphify] AST: {len(ast_result['nodes'])} nodes, {len(ast_result['edges'])} edges")
else:
    ast_result = {'nodes': [], 'edges': [], 'hyperedges': [], 'input_tokens': 0, 'output_tokens': 0}

# Step 3: Build and cluster
print("[graphify] Building graph...")
extraction = ast_result
G = build_from_json(extraction)

if G.number_of_nodes() == 0:
    print("[graphify] ERROR: Graph is empty")
else:
    communities = cluster(G)
    cohesion = score_all(G, communities)
    gods = god_nodes(G)
    surprises = surprising_connections(G, communities)
    labels = {cid: 'Community ' + str(cid) for cid in communities}

    # Step 4: Generate outputs
    output_dir = Path('graphify-out')
    output_dir.mkdir(exist_ok=True)

    report = generate(G, communities, cohesion, labels, gods, surprises, result, 
                     {'input': extraction.get('input_tokens', 0), 'output': extraction.get('output_tokens', 0)}, 
                     '${path}')
    (output_dir / 'GRAPH_REPORT.md').write_text(report)
    to_json(G, communities, output_dir / 'graph.json')

    if G.number_of_nodes() <= 5000:
        to_html(G, communities, output_dir / 'graph.html', community_labels=labels)

    print(f"[graphify] Done! Graph: {G.number_of_nodes()} nodes, {G.number_of_edges()} edges, {len(communities)} communities")
    print("[graphify] Output: graphify-out/")
`;

      // Replace the bash command with our graphify script
      output.args.command = `python3 -c "${graphifyScript.replace(/"/g, '\\"').replace(/\n/g, '\\n')}"`;
    },
  };
};
