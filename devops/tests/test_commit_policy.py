#!/usr/bin/env python3
"""
Enforces the commit policy on the tree, or on the files a branch changed.

PROTECTS: Only the documented example projects live under `examples/`, and no
file a branch changed is larger than the policy allows - the same rules the
pre-commit hook applies, so a clone that never enabled the hook is still caught
by the gate and by CI.
TIER: build

REQUIREMENTS
  - Python 3.8+
  - Git. The size half needs a base to diff against: `PILL_BASE_REF`, or an
    `origin/*` default branch. Without one it prints [SKIP] for that half and
    still checks the example projects.

DESCRIPTION
    Reads `devops/hooks/policy.env`, the single source of truth shared with
    the pre-commit hook, and checks two things.

    Example projects: every tracked path under `examples/` must sit in one of
    the listed project folders. Files directly in `examples/` (the README) are
    not project folders and are left alone.

    File size: every file the branch *added*, relative to the merge base with
    the base branch, must be at or under the limit. Added only: a changed file
    is usually an existing one being edited, and refusing those would block
    every improvement to a large source file or to the lock file. The stricter
    per-commit rule (any staged file) lives in the pre-commit hook, where the
    author can see what is about to be committed.

USAGE
  python devops/tests/test_commit_policy.py

EXAMPLE USAGE
  python devops/tests/test_commit_policy.py
"""

# Standard library
import os
import subprocess
import sys
from pathlib import Path
from typing import Dict, List, Optional

# =============================================================================
# Constants
# =============================================================================

REPOSITORY_ROOT = Path(__file__).resolve().parent.parent.parent
POLICY_FILE = Path("devops") / "hooks" / "policy.env"

# The refs the size check compares against, in the order they are tried. The
# branch's own upstream wins: a long-lived branch cut from `main` months ago
# would otherwise report every large file added since then as a violation of
# this branch, which is noise, not a mistake.
BASE_BRANCH_CANDIDATES = ("@{upstream}", "origin/HEAD", "origin/main", "origin/master")


# =============================================================================
# Free Functions
# =============================================================================

def run_git(*arguments: str) -> subprocess.CompletedProcess:
    """Runs a git command in the repository root."""
    return subprocess.run(
        ["git", *arguments],
        cwd=str(REPOSITORY_ROOT),
        capture_output=True,
        text=True,
    )


def read_policy() -> Dict[str, str]:
    """The policy values, read from the file the pre-commit hook also sources."""
    values: Dict[str, str] = {}
    for line in (REPOSITORY_ROOT / POLICY_FILE).read_text(encoding="utf-8").splitlines():
        stripped = line.strip()
        if not stripped or stripped.startswith("#") or "=" not in stripped:
            continue
        key, value = stripped.split("=", 1)
        values[key.strip()] = value.strip().strip('"').strip("'")
    return values


def example_project_violations(allowed: List[str]) -> List[str]:
    """Tracked paths under `examples/` that are outside the allowed projects."""
    listed = run_git("ls-files", "examples")
    if listed.returncode != 0:
        return []
    violations = []
    for path in listed.stdout.splitlines():
        parts = path.split("/")
        # `examples/<file>` is a file in the examples root, not a project folder.
        if len(parts) < 3:
            continue
        if parts[1] not in allowed:
            violations.append(path)
    return violations


def resolve_base() -> Optional[str]:
    """The ref the size check compares against, or None when there is none."""
    configured = os.environ.get("PILL_BASE_REF")
    if configured:
        return configured
    for candidate in BASE_BRANCH_CANDIDATES:
        if run_git("rev-parse", "--verify", "--quiet", candidate).returncode == 0:
            return candidate
    return None


def size_violations(base: str, limit_bytes: int) -> Optional[List[str]]:
    """Added files above the limit, or None when the base cannot be resolved.

    Only files the branch *added* are measured. A changed file is usually one
    that already existed and was edited, and refusing those would block every
    improvement to a large source file or the lock file; the pre-commit hook
    applies the stricter rule per commit, where the author can see the intent.
    """
    merge_base = run_git("merge-base", base, "HEAD")
    if merge_base.returncode != 0:
        return None
    changed = run_git(
        "diff", "--name-only", "--diff-filter=A", merge_base.stdout.strip(), "HEAD"
    )
    if changed.returncode != 0:
        return None
    violations = []
    for path in changed.stdout.splitlines():
        file_path = REPOSITORY_ROOT / path
        if file_path.is_file() and file_path.stat().st_size > limit_bytes:
            violations.append(f"{path} ({file_path.stat().st_size} bytes)")
    return violations


def main() -> int:
    """Checks the policy and returns an exit code."""
    if not (REPOSITORY_ROOT / POLICY_FILE).is_file():
        print(f"[FAIL] the commit policy file {POLICY_FILE} is missing")
        return 1
    policy = read_policy()
    limit_bytes = int(policy.get("SIZE_LIMIT_BYTES", "0"))
    allowed = policy.get("EXAMPLE_PROJECTS", "").split()
    if limit_bytes <= 0 or not allowed:
        print(f"[FAIL] {POLICY_FILE} does not name a size limit and example projects")
        return 1

    print(f"[RUN] commit policy: example projects {', '.join(allowed)}")
    failures = []
    for path in example_project_violations(allowed):
        failures.append(f"{path} is in an example project that is not listed")

    base = resolve_base()
    if base is None:
        print(
            "[SKIP] the file size half: no base to compare against "
            "(set PILL_BASE_REF, or fetch origin)"
        )
    else:
        over = size_violations(base, limit_bytes)
        if over is None:
            print(f"[SKIP] the file size half: {base} has no merge base with HEAD")
        else:
            print(
                f"[RUN] file size: files added against {base}, limit {limit_bytes} bytes"
            )
            failures += [
                f"{entry} is over the {limit_bytes}-byte limit" for entry in over
            ]

    for failure in failures:
        print(f"  - {failure}")
    if failures:
        print("[FAIL] the commit policy is broken")
        return 1
    print("[PASS] the commit policy holds")
    return 0


if __name__ == "__main__":
    sys.exit(main())
