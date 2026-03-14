#!/usr/bin/env python3

from __future__ import annotations

import argparse
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
SUITE_COMMAND = ["cargo", "test", "--quiet", "--", "--test-threads=1"]


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--count", type=int, required=True)
    args = parser.parse_args()

    for _ in range(args.count):
        result = subprocess.run(
            SUITE_COMMAND,
            cwd=ROOT,
            check=False,
        )
        if result.returncode != 0:
            return result.returncode
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
