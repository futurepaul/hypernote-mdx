#!/usr/bin/env python3

from __future__ import annotations

import argparse
import csv
import json
import math
import subprocess
import sys
import time
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
RESEARCH_DIR = ROOT / "research"
LOGS_DIR = RESEARCH_DIR / "logs"
TMP_DIR = RESEARCH_DIR / "tmp"
RESULTS_TSV = RESEARCH_DIR / "results.tsv"
PLOT_SCRIPT = RESEARCH_DIR / "plot_progress.py"
TSV_HEADER = [
    "commit",
    "suite_seconds",
    "suite_stddev_seconds",
    "bench_runs",
    "build_seconds",
    "status",
    "description",
]


def run(
    cmd: list[str],
    *,
    timeout: int,
    stdout_path: Path | None = None,
    stderr_path: Path | None = None,
) -> subprocess.CompletedProcess[str]:
    stdout_handle = open(stdout_path, "w", encoding="utf-8") if stdout_path else subprocess.PIPE
    stderr_handle = open(stderr_path, "w", encoding="utf-8") if stderr_path else subprocess.PIPE
    try:
        return subprocess.run(
            cmd,
            cwd=ROOT,
            text=True,
            check=False,
            timeout=timeout,
            stdout=stdout_handle,
            stderr=stderr_handle,
        )
    finally:
        if stdout_path:
            stdout_handle.close()
        if stderr_path:
            stderr_handle.close()


def capture(cmd: list[str]) -> str:
    result = subprocess.run(
        cmd,
        cwd=ROOT,
        text=True,
        check=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    return result.stdout.strip()


def ensure_clean_git() -> None:
    status = capture(["git", "status", "--porcelain"])
    if status:
        raise SystemExit("working tree must be clean before running an experiment")


def ensure_dirs() -> None:
    LOGS_DIR.mkdir(parents=True, exist_ok=True)
    TMP_DIR.mkdir(parents=True, exist_ok=True)


def ensure_results_tsv() -> None:
    if RESULTS_TSV.exists():
        return
    with RESULTS_TSV.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.writer(handle, delimiter="\t")
        writer.writerow(TSV_HEADER)


def results_row_count() -> int:
    if not RESULTS_TSV.exists():
        return 0
    with RESULTS_TSV.open("r", encoding="utf-8", newline="") as handle:
        reader = csv.DictReader(handle, delimiter="\t")
        return sum(1 for _ in reader)


def latest_keep() -> dict[str, str] | None:
    if not RESULTS_TSV.exists():
        return None
    with RESULTS_TSV.open("r", encoding="utf-8", newline="") as handle:
        rows = list(csv.DictReader(handle, delimiter="\t"))
    kept = [row for row in rows if row["status"] == "keep"]
    return kept[-1] if kept else None


def measure_build(commit: str, timeout: int) -> tuple[float, bool]:
    stdout_path = LOGS_DIR / f"{commit}.build.stdout.log"
    stderr_path = LOGS_DIR / f"{commit}.build.stderr.log"
    started = time.perf_counter()
    try:
        result = run(
            ["cargo", "test", "--no-run", "--quiet"],
            timeout=timeout,
            stdout_path=stdout_path,
            stderr_path=stderr_path,
        )
    except subprocess.TimeoutExpired:
        return time.perf_counter() - started, False
    return time.perf_counter() - started, result.returncode == 0


def probe_suite(commit: str, timeout: int) -> tuple[float, bool]:
    stdout_path = LOGS_DIR / f"{commit}.probe.stdout.log"
    stderr_path = LOGS_DIR / f"{commit}.probe.stderr.log"
    started = time.perf_counter()
    try:
        result = run(
            ["cargo", "test", "--quiet"],
            timeout=timeout,
            stdout_path=stdout_path,
            stderr_path=stderr_path,
        )
    except subprocess.TimeoutExpired:
        return 0.0, False
    return time.perf_counter() - started, result.returncode == 0


def resolve_bench_runs(
    requested_runs: int,
    min_benchmark_seconds: float,
    probe_seconds: float,
) -> int:
    if probe_seconds <= 0.0:
        return requested_runs
    budget_runs = math.ceil(min_benchmark_seconds / probe_seconds)
    return max(requested_runs, budget_runs)


def measure_suite(commit: str, runs: int, warmup: int, timeout: int) -> tuple[float, float, bool]:
    json_path = TMP_DIR / f"{commit}.hyperfine.json"
    stdout_path = LOGS_DIR / f"{commit}.hyperfine.stdout.log"
    stderr_path = LOGS_DIR / f"{commit}.hyperfine.stderr.log"
    cmd = [
        "hyperfine",
        "--warmup",
        str(warmup),
        "--runs",
        str(runs),
        "--export-json",
        str(json_path),
        "--prepare",
        "cargo test --no-run --quiet",
        "cargo test --quiet",
    ]
    try:
        result = run(
            cmd,
            timeout=timeout,
            stdout_path=stdout_path,
            stderr_path=stderr_path,
        )
    except subprocess.TimeoutExpired:
        return 0.0, 0.0, False
    if result.returncode != 0 or not json_path.exists():
        return 0.0, 0.0, False
    payload = json.loads(json_path.read_text(encoding="utf-8"))
    benchmark = payload["results"][0]
    return float(benchmark["mean"]), float(benchmark["stddev"]), True


def decide_status(
    requested: str,
    suite_seconds: float,
    threshold_ms: float,
    threshold_pct: float,
) -> str:
    if requested != "auto":
        return requested
    previous = latest_keep()
    if previous is None:
        return "keep"
    previous_seconds = float(previous["suite_seconds"])
    delta = previous_seconds - suite_seconds
    threshold = max(threshold_ms / 1000.0, previous_seconds * (threshold_pct / 100.0))
    return "keep" if delta > threshold else "discard"


def append_row(row: dict[str, str | float]) -> None:
    ensure_results_tsv()
    with RESULTS_TSV.open("a", encoding="utf-8", newline="") as handle:
        writer = csv.writer(handle, delimiter="\t")
        writer.writerow(
            [
                row["commit"],
                f"{row['suite_seconds']:.6f}",
                f"{row['suite_stddev_seconds']:.6f}",
                row["bench_runs"],
                f"{row['build_seconds']:.6f}",
                row["status"],
                row["description"],
            ]
        )


def refresh_report() -> None:
    subprocess.run(
        [sys.executable, str(PLOT_SCRIPT)],
        cwd=ROOT,
        check=True,
    )


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--description", required=True)
    parser.add_argument(
        "--status",
        choices=["auto", "keep", "discard", "crash"],
        default="auto",
    )
    parser.add_argument("--runs", type=int, default=10)
    parser.add_argument("--warmup", type=int, default=3)
    parser.add_argument("--min-benchmark-seconds", type=float, default=6.0)
    parser.add_argument("--threshold-ms", type=float, default=10.0)
    parser.add_argument("--threshold-pct", type=float, default=3.0)
    parser.add_argument("--build-timeout", type=int, default=180)
    parser.add_argument("--probe-timeout", type=int, default=180)
    parser.add_argument("--bench-timeout", type=int, default=300)
    args = parser.parse_args()

    ensure_clean_git()
    ensure_dirs()
    ensure_results_tsv()
    if args.description == "baseline" and results_row_count() > 0:
        raise SystemExit(
            "baseline already exists; run `just research-reset` before starting a new campaign"
        )

    commit = capture(["git", "rev-parse", "--short", "HEAD"])
    build_seconds, build_ok = measure_build(commit, args.build_timeout)

    if build_ok:
        probe_seconds, probe_ok = probe_suite(commit, args.probe_timeout)
    else:
        probe_seconds, probe_ok = 0.0, False

    bench_runs = resolve_bench_runs(
        args.runs, args.min_benchmark_seconds, probe_seconds
    )

    if build_ok and probe_ok:
        suite_seconds, suite_stddev_seconds, bench_ok = measure_suite(
            commit, bench_runs, args.warmup, args.bench_timeout
        )
    else:
        suite_seconds, suite_stddev_seconds, bench_ok = 0.0, 0.0, False

    if not build_ok or not probe_ok or not bench_ok:
        status = "crash"
    else:
        status = decide_status(
            args.status,
            suite_seconds,
            args.threshold_ms,
            args.threshold_pct,
        )

    row = {
        "commit": commit,
        "suite_seconds": suite_seconds,
        "suite_stddev_seconds": suite_stddev_seconds,
        "bench_runs": bench_runs,
        "build_seconds": build_seconds,
        "status": status,
        "description": args.description,
    }
    append_row(row)
    refresh_report()

    print("---")
    print(f"commit:               {commit}")
    print(f"probe_seconds:        {probe_seconds:.6f}")
    print(f"bench_runs:           {bench_runs}")
    print(f"suite_mean_seconds:   {suite_seconds:.6f}")
    print(f"suite_stddev_seconds: {suite_stddev_seconds:.6f}")
    print(f"build_seconds:        {build_seconds:.6f}")
    print(f"status:               {status}")
    print(f"description:          {args.description}")
    if status in {"discard", "crash"}:
        print("suggested_reset:      git reset --hard HEAD~1")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
