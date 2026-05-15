#!/usr/bin/env python3
"""claude_check_memory.py — lint the Claude Code memory directory for this repo.

Why this lives in `tools/`: the project keeps a per-contributor memory
tree at ~/.claude/projects/<slug>/memory/ (the slug is derived by
Claude Code from the repo's absolute path). The tree has grown large
enough that broken [[link]] refs, MEMORY.md truncation, and orphan
files are easy to introduce. This linter catches all four.

Runs four checks:

  1. Broken [[links]] — references to nonexistent files
  2. Orphan files — memory files not linked from MEMORY.md
  3. MEMORY.md line budget — must be under 200 lines (truncation cap)
  4. Stale CS claims — wisdom files that mention a \\cs that no longer
     appears in the codebase (probably a renamed/removed CS)

Usage:
  python3 tools/claude_check_memory.py                # report
  python3 tools/claude_check_memory.py --strict       # exit non-zero on any issue
  python3 tools/claude_check_memory.py --check links  # only the named check

Designed to be cheap (~1s) so it can run in pre-push or periodically.
Operates only on the running contributor's own memory dir (the slug
is auto-derived from this file's location), so the tool is portable
across machines and contributors.
"""
from __future__ import annotations

import argparse
import re
import subprocess
import sys
from pathlib import Path

REPO_DIR = Path(__file__).resolve().parent.parent
# Claude Code derives the projects subdir from the repo's absolute
# path: leading "/" replaced with "-", remaining "/"s with "-".
PROJECT_SLUG = "-" + str(REPO_DIR).lstrip("/").replace("/", "-")
MEMORY_DIR = Path.home() / ".claude" / "projects" / PROJECT_SLUG / "memory"
MEMORY_INDEX = MEMORY_DIR / "MEMORY.md"
LINE_BUDGET = 200  # MEMORY.md truncates beyond this

LINK_RE = re.compile(r"\[\[([a-zA-Z0-9_]+)\]\]")
# CLAUDE.md's MEMORY.md format is `- [Title](file.md) — hook` (markdown
# link, not wikilink). Recognize both styles so the linter reflects the
# documented index format. Capture the stem (drop trailing `.md`).
MD_LINK_RE = re.compile(r"\(([a-zA-Z0-9_]+)\.md\)")
CS_RE = re.compile(r"\\([a-zA-Z@]+)")  # \foo, \lx@frontmatterhere, etc.


def memory_files() -> set[str]:
  return {p.stem for p in MEMORY_DIR.glob("*.md") if p.stem != "MEMORY"}


def _strip_code(text: str) -> str:
  """Remove fenced code blocks and inline `code` so we don't match
  [[link]] syntax inside literal examples."""
  text = re.sub(r"```[\s\S]*?```", "", text)  # fenced code
  text = re.sub(r"`[^`\n]*`", "", text)        # inline code
  return text


def all_refs() -> dict[str, set[str]]:
  """Map memory-file-stem -> set of [[refs]] it contains.

  Skips refs inside code fences and inline backticks so example
  snippets like `[[NAME]]` don't register as real references.
  """
  out: dict[str, set[str]] = {}
  for p in MEMORY_DIR.glob("*.md"):
    text = _strip_code(p.read_text(errors="replace"))
    out[p.stem] = set(LINK_RE.findall(text))
  return out


def index_refs() -> set[str]:
  if not MEMORY_INDEX.exists():
    return set()
  text = _strip_code(MEMORY_INDEX.read_text())
  return set(LINK_RE.findall(text)) | set(MD_LINK_RE.findall(text))


def check_broken_links(verbose: bool = True) -> int:
  files = memory_files() | {"MEMORY"}
  broken: list[tuple[str, str]] = []
  for src, refs in all_refs().items():
    for r in refs:
      if r not in files:
        broken.append((src, r))
  if verbose:
    print(f"[broken-links] {len(broken)} broken [[link]] refs")
    for src, r in sorted(broken):
      print(f"  {src}.md -> [[{r}]] (no such file)")
  return len(broken)


def check_orphans(verbose: bool = True) -> int:
  refs = index_refs()
  orphans = sorted(f for f in memory_files() if f not in refs)
  if verbose:
    print(f"[orphans] {len(orphans)} files not linked from MEMORY.md")
    for f in orphans:
      print(f"  {f}.md")
  return len(orphans)


def check_index_budget(verbose: bool = True) -> int:
  if not MEMORY_INDEX.exists():
    if verbose:
      print("[budget] MEMORY.md missing")
    return 1
  lines = MEMORY_INDEX.read_text().splitlines()
  n = len(lines)
  over = max(0, n - LINE_BUDGET)
  if verbose:
    status = "OK" if over == 0 else f"OVER BUDGET by {over} lines"
    print(f"[budget] MEMORY.md = {n} lines (cap {LINE_BUDGET}) — {status}")
  return over


# Pedagogical / example CSes that wouldn't appear in source verbatim.
EXAMPLE_CSES = {
  "foo", "bar", "baz", "foo@bar", "some", "X", "Y", "Z", "A", "B", "C",
  "cs", "csname", "stuff", "thing", "x", "y", "z",
}


def check_stale_cs_claims(verbose: bool = True, sample: int = 30,
                          threshold: float = 0.7) -> int:
  """Flag wisdom files where MANY (>threshold) referenced CSes are not
  found in source. A single missing CS is usually a pedagogical
  example; many missing suggests the file is genuinely stale.
  """
  if not (REPO_DIR / "latexml_core").exists():
    if verbose:
      print("[stale-cs] skipped (not in repo)")
    return 0
  rg_check = subprocess.run(["which", "rg"], capture_output=True)
  if rg_check.returncode != 0:
    if verbose:
      print("[stale-cs] skipped (ripgrep not available)")
    return 0
  search_dirs = [str(REPO_DIR / d) for d in
                 ("latexml_core", "latexml_engine", "latexml_package",
                  "latexml_contrib", "latexml_oxide")]
  stale = []
  for p in sorted(MEMORY_DIR.glob("wisdom_*.md")):
    text = p.read_text(errors="replace")
    cses = set(CS_RE.findall(text))
    cses = {c for c in cses if len(c) > 1 and c not in EXAMPLE_CSES}
    cses = list(cses)[:sample]
    if len(cses) < 4:  # too few signal to assess
      continue
    misses = []
    for cs in cses:
      r = subprocess.run(
        ["rg", "-q", "-F", f"\\{cs}", *search_dirs],
        capture_output=True,
      )
      if r.returncode != 0:
        misses.append(cs)
    miss_ratio = len(misses) / len(cses)
    if miss_ratio >= threshold:
      stale.append((p.stem, miss_ratio, misses[:3]))
  if verbose:
    print(f"[stale-cs] {len(stale)} wisdom files where >{int(threshold*100)}% of CSes are missing")
    for src, ratio, miss in stale:
      print(f"  {src}.md ({int(ratio*100)}% miss) -> \\{', \\'.join(miss)}")
  return len(stale)


CHECKS = {
  "links": check_broken_links,
  "orphans": check_orphans,
  "budget": check_index_budget,
  "stale-cs": check_stale_cs_claims,
}


def main(argv: list[str]) -> int:
  ap = argparse.ArgumentParser(description=__doc__.split("\n", 1)[0])
  ap.add_argument("--strict", action="store_true",
                  help="exit non-zero if any issue found")
  ap.add_argument("--check", choices=list(CHECKS) + ["all"], default="all")
  ap.add_argument("-q", "--quiet", action="store_true")
  args = ap.parse_args(argv)

  if not MEMORY_DIR.exists():
    print(f"ERROR: memory dir not found at {MEMORY_DIR}", file=sys.stderr)
    return 2

  selected = list(CHECKS) if args.check == "all" else [args.check]
  issues = 0
  for name in selected:
    issues += CHECKS[name](verbose=not args.quiet)
  print(f"\nTotal issues: {issues}")
  if args.strict and issues:
    return 1
  return 0


if __name__ == "__main__":
  sys.exit(main(sys.argv[1:]))
