#!/usr/bin/env python3

from __future__ import annotations

import html
from datetime import datetime
from pathlib import Path

import matplotlib.pyplot as plt
import pandas as pd


ROOT = Path(__file__).resolve().parent.parent
RESEARCH_DIR = ROOT / "research"
RESULTS_TSV = RESEARCH_DIR / "results.tsv"
PROGRESS_PNG = RESEARCH_DIR / "progress.png"
INDEX_HTML = RESEARCH_DIR / "index.html"


def empty_report() -> None:
    INDEX_HTML.write_text(
        """<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta http-equiv="refresh" content="5">
  <title>hypernote-mdx autoresearch</title>
  <style>
    body { font-family: Georgia, serif; margin: 2rem; line-height: 1.5; }
    code { background: #f3f0e8; padding: 0.1rem 0.3rem; }
  </style>
</head>
<body>
  <h1>hypernote-mdx autoresearch</h1>
  <p>No results yet. Run <code>just research-baseline</code>.</p>
</body>
</html>
""",
        encoding="utf-8",
    )


def main() -> int:
    if not RESULTS_TSV.exists():
        empty_report()
        return 0

    df = pd.read_csv(RESULTS_TSV, sep="\t")
    if df.empty:
        empty_report()
        return 0

    df["suite_seconds"] = pd.to_numeric(df["suite_seconds"], errors="coerce")
    df["suite_stddev_seconds"] = pd.to_numeric(df["suite_stddev_seconds"], errors="coerce")
    if "bench_runs" in df.columns:
        df["bench_runs"] = pd.to_numeric(df["bench_runs"], errors="coerce")
    else:
        df["bench_runs"] = pd.NA
    df["build_seconds"] = pd.to_numeric(df["build_seconds"], errors="coerce")
    df["status"] = df["status"].str.strip().str.lower()

    valid = df[df["status"] != "crash"].copy().reset_index(drop=True)
    if valid.empty:
        empty_report()
        return 0

    baseline = float(valid.loc[0, "suite_seconds"])
    kept = valid[valid["status"] == "keep"].copy()
    best = float(kept["suite_seconds"].min()) if not kept.empty else baseline

    fig, ax = plt.subplots(figsize=(16, 8))

    discarded = valid[valid["status"] == "discard"]
    ax.scatter(
        discarded.index,
        discarded["suite_seconds"],
        c="#c2c2c2",
        s=18,
        alpha=0.55,
        zorder=2,
        label="Discarded",
    )

    kept_idx = valid.index[valid["status"] == "keep"]
    kept_scores = valid.loc[valid["status"] == "keep", "suite_seconds"]
    ax.scatter(
        kept_idx,
        kept_scores,
        c="#1c7c54",
        s=56,
        zorder=4,
        label="Kept",
        edgecolors="black",
        linewidths=0.5,
    )

    if not kept_scores.empty:
        ax.step(
            kept_idx,
            kept_scores.cummin(),
            where="post",
            color="#0b5d3b",
            linewidth=2,
            alpha=0.85,
            zorder=3,
            label="Running best",
        )

    for idx in kept_idx:
        label = str(valid.loc[idx, "description"]).strip()
        if len(label) > 45:
            label = label[:42] + "..."
        ax.annotate(
            label,
            (idx, valid.loc[idx, "suite_seconds"]),
            textcoords="offset points",
            xytext=(6, 6),
            fontsize=8,
            color="#0b5d3b",
            rotation=28,
            ha="left",
            va="bottom",
        )

    n_total = len(df)
    n_kept = len(df[df["status"] == "keep"])
    ax.set_xlabel("Experiment #", fontsize=12)
    ax.set_ylabel("Suite Runtime (seconds, lower is better)", fontsize=12)
    ax.set_title(
        f"hypernote-mdx Autoresearch: {n_total} Experiments, {n_kept} Kept",
        fontsize=14,
    )
    ax.grid(True, alpha=0.2)
    ax.legend(loc="upper right", fontsize=9)

    margin = max(0.02, (baseline - best) * 0.2 if baseline > best else baseline * 0.1)
    ax.set_ylim(max(0.0, best - margin), baseline + margin)

    plt.tight_layout()
    plt.savefig(PROGRESS_PNG, dpi=150, bbox_inches="tight")
    plt.close(fig)

    best_row = kept.loc[kept["suite_seconds"].idxmin()] if not kept.empty else valid.iloc[0]
    improvement = baseline - float(best_row["suite_seconds"])
    improvement_pct = (improvement / baseline * 100.0) if baseline else 0.0

    recent_rows = []
    for _, row in df.tail(20).iterrows():
        bench_runs = row["bench_runs"]
        bench_runs_text = (
            str(int(bench_runs)) if pd.notna(bench_runs) else "n/a"
        )
        recent_rows.append(
            "<tr>"
            f"<td><code>{html.escape(str(row['commit']))}</code></td>"
            f"<td>{float(row['suite_seconds']):.6f}</td>"
            f"<td>{float(row['suite_stddev_seconds']):.6f}</td>"
            f"<td>{bench_runs_text}</td>"
            f"<td>{float(row['build_seconds']):.6f}</td>"
            f"<td>{html.escape(str(row['status']))}</td>"
            f"<td>{html.escape(str(row['description']))}</td>"
            "</tr>"
        )

    html_doc = f"""<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta http-equiv="refresh" content="5">
  <title>hypernote-mdx autoresearch</title>
  <style>
    :root {{
      color-scheme: light;
      --bg: #f5f1e8;
      --fg: #1b1a17;
      --muted: #70695a;
      --card: #fffaf0;
      --accent: #1c7c54;
      --line: #d9d0bf;
    }}
    body {{
      margin: 0;
      font-family: Georgia, serif;
      background: radial-gradient(circle at top, #fffef8 0%, var(--bg) 65%);
      color: var(--fg);
    }}
    main {{
      max-width: 1200px;
      margin: 0 auto;
      padding: 2rem;
    }}
    h1, h2 {{
      margin-bottom: 0.4rem;
    }}
    p {{
      color: var(--muted);
    }}
    .grid {{
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
      gap: 1rem;
      margin: 1.5rem 0;
    }}
    .card {{
      background: var(--card);
      border: 1px solid var(--line);
      border-radius: 16px;
      padding: 1rem 1.2rem;
      box-shadow: 0 12px 30px rgba(27, 26, 23, 0.08);
    }}
    .metric {{
      font-size: 1.8rem;
      color: var(--accent);
    }}
    img {{
      width: 100%;
      border-radius: 18px;
      border: 1px solid var(--line);
      background: white;
      box-shadow: 0 18px 34px rgba(27, 26, 23, 0.08);
    }}
    table {{
      width: 100%;
      border-collapse: collapse;
      background: var(--card);
      border-radius: 16px;
      overflow: hidden;
    }}
    th, td {{
      padding: 0.7rem 0.8rem;
      border-bottom: 1px solid var(--line);
      text-align: left;
      font-size: 0.95rem;
    }}
    th {{
      background: #efe6d4;
    }}
    code {{
      background: #efe6d4;
      padding: 0.1rem 0.3rem;
      border-radius: 6px;
    }}
  </style>
</head>
<body>
  <main>
    <h1>hypernote-mdx autoresearch</h1>
    <p>Auto-refreshes every 5 seconds. Serve this directory with <code>just research-serve</code>.</p>
    <p>Official score: <code>hyperfine</code> on <code>cargo test --quiet</code> with compile excluded via <code>--prepare 'cargo test --no-run --quiet'</code>. The harness auto-increases timed runs so each benchmark spends several measured seconds, not a tiny probe.</p>
    <div class="grid">
      <div class="card">
        <div>Baseline</div>
        <div class="metric">{baseline:.6f}s</div>
      </div>
      <div class="card">
        <div>Best kept</div>
        <div class="metric">{float(best_row["suite_seconds"]):.6f}s</div>
      </div>
      <div class="card">
        <div>Total improvement</div>
        <div class="metric">{improvement:.6f}s</div>
        <div>{improvement_pct:.2f}%</div>
      </div>
      <div class="card">
        <div>Experiments</div>
        <div class="metric">{n_total}</div>
        <div>{n_kept} kept</div>
      </div>
    </div>
    <img src="progress.png?ts={int(datetime.now().timestamp())}" alt="Autoresearch progress chart">
    <h2>Recent results</h2>
    <table>
      <thead>
        <tr>
          <th>Commit</th>
          <th>Suite mean</th>
          <th>Stddev</th>
          <th>Runs</th>
          <th>Build</th>
          <th>Status</th>
          <th>Description</th>
        </tr>
      </thead>
      <tbody>
        {''.join(recent_rows)}
      </tbody>
    </table>
    <p>Last updated: {datetime.now().strftime("%Y-%m-%d %H:%M:%S")}</p>
  </main>
</body>
</html>
"""
    INDEX_HTML.write_text(html_doc, encoding="utf-8")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
