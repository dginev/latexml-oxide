#!/usr/bin/env python3
"""Decide whether a final `X.Y.Z` release tag can safely skip redundant test
shards and/or reuse the 5-year TeX Live dumps from a passing `X.Y.Z-rcN` tag.

Fail-closed by design: any code/config/dependency diff since the latest RC tag,
any missing/unverified RC run, or any API error falls back cleanly to running
the full release pipeline (`skip_tests=false`, `reuse_dumps_run_id=""`).
"""

from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
import sys
import urllib.error
import urllib.request
from dataclasses import dataclass
from typing import Callable, Optional

DEFAULT_DUMP_YEARS = ("2022", "2023", "2024", "2025", "2026")

# Top-level `[package] version = "..."` line in a Cargo.toml.
PKG_VERSION_RE = re.compile(r'^\s*version\s*=\s*"[^"]+"\s*$')

# Intra-workspace path dependency line in a Cargo.toml, e.g.:
#   latexml_core = {path = "../latexml_core", version = "0.6.0"}
WORKSPACE_DEP_RE = re.compile(
    r'^\s*(latexml(?:_[a-z_]+)?)\s*=\s*\{([^}]*)\}\s*$'
)
VERSION_KV_RE = re.compile(r'\bversion\s*=\s*"[^"]+"')
PATH_KV_RE = re.compile(r'\bpath\s*=\s*"\.\./[^"]+"')


@dataclass
class ReuseDecision:
    skip_tests: bool = False
    reuse_dumps_run_id: str = ""
    source_rc: str = ""
    reason: str = ""


def _run_git(args: list[str], cwd: Optional[str] = None) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        ["git", *args],
        cwd=cwd,
        check=False,
        capture_output=True,
        text=True,
    )


def _is_allowed_doc_path(path: str) -> bool:
    return (
        path in ("CHANGELOG.md", "README.md")
        or path.startswith("docs/")
    )


def _is_cargo_toml(path: str) -> bool:
    return path == "Cargo.toml" or path.endswith("/Cargo.toml")


def _normalize_workspace_dep(line: str) -> Optional[str]:
    """If `line` is an intra-workspace `latexml* = { path = "../...", version = "..." }`
    line, return it with the version string normalized to a placeholder; else None."""
    m = WORKSPACE_DEP_RE.match(line)
    if not m:
        return None
    body = m.group(2)
    if not PATH_KV_RE.search(body) or not VERSION_KV_RE.search(body):
        return None
    return VERSION_KV_RE.sub('version = "<VERSION>"', line, count=1)


def is_version_only_cargo_diff(diff_text: str) -> tuple[bool, str]:
    """Return `(True, "")` iff every hunk in a `git diff --unified=0` of a
    Cargo.toml modifies only package `version = "..."` lines or intra-workspace
    `latexml*` `version = "..."` pins (with zero other edits)."""
    lines = diff_text.splitlines()
    i = 0
    saw_hunk = False
    while i < len(lines):
        line = lines[i]
        if not line.startswith("@@ "):
            i += 1
            continue
        saw_hunk = True
        i += 1
        removed: list[str] = []
        added: list[str] = []
        while i < len(lines) and not lines[i].startswith("@@ "):
            cur = lines[i]
            if cur.startswith("-") and not cur.startswith("---"):
                removed.append(cur[1:])
            elif cur.startswith("+") and not cur.startswith("+++"):
                added.append(cur[1:])
            elif cur.startswith("\\ No newline"):
                pass
            elif cur.startswith(("diff --git ", "index ", "--- ", "+++ ")):
                break
            else:
                return False, f"unexpected diff line: {cur!r}"
            i += 1

        if len(removed) != len(added):
            return False, (
                f"Cargo.toml hunk changed line count (-{len(removed)} / +{len(added)})"
            )
        for old, new in zip(removed, added):
            if PKG_VERSION_RE.match(old) and PKG_VERSION_RE.match(new):
                continue
            old_norm = _normalize_workspace_dep(old)
            new_norm = _normalize_workspace_dep(new)
            if old_norm is not None and old_norm == new_norm:
                continue
            return False, f"non-version Cargo.toml change: -{old!r} / +{new!r}"

    if not saw_hunk:
        # Mode-only or empty diff on Cargo.toml — reject mode changes conservatively.
        if "old mode" in diff_text or "new mode" in diff_text:
            return False, "Cargo.toml file mode changed"
    return True, ""


def diff_is_rc_promotion_only(
    rc_tag: str, head_ref: str = "HEAD", cwd: Optional[str] = None
) -> tuple[bool, str]:
    """Verify `rc_tag..head_ref` touches ONLY Cargo.toml version pins and docs."""
    ns = _run_git(["diff", "--name-status", f"{rc_tag}..{head_ref}"], cwd=cwd)
    if ns.returncode != 0:
        return False, f"git diff --name-status failed: {ns.stderr.strip()}"

    cargo_files: list[str] = []
    for raw in ns.stdout.splitlines():
        if not raw.strip():
            continue
        parts = raw.split("\t")
        status = parts[0]
        path = parts[-1]
        if _is_allowed_doc_path(path):
            if status not in ("A", "M"):
                return False, f"disallowed status {status} on doc file {path}"
            continue
        if _is_cargo_toml(path):
            if status != "M":
                return False, f"disallowed status {status} on {path}"
            cargo_files.append(path)
            continue
        return False, f"non-promotion file changed since {rc_tag}: {status} {path}"

    for path in cargo_files:
        d = _run_git(
            ["diff", "--unified=0", f"{rc_tag}..{head_ref}", "--", path], cwd=cwd
        )
        if d.returncode != 0:
            return False, f"git diff failed on {path}: {d.stderr.strip()}"
        ok, why = is_version_only_cargo_diff(d.stdout)
        if not ok:
            return False, f"{path}: {why}"

    return True, ""


def find_latest_rc_tag(tag: str, cwd: Optional[str] = None) -> Optional[str]:
    """Return the highest-versioned `<tag>-rc*` tag, or None."""
    res = _run_git(["tag", "--list", f"{tag}-rc*", "--sort=-v:refname"], cwd=cwd)
    if res.returncode != 0:
        return None
    for line in res.stdout.splitlines():
        candidate = line.strip()
        if candidate:
            return candidate
    return None


def _default_api_get(url: str, token: str) -> dict:
    headers = {
        "Accept": "application/vnd.github+json",
        "User-Agent": "latexml-oxide-check-rc-reuse",
        "X-GitHub-Api-Version": "2022-11-28",
    }
    if token:
        headers["Authorization"] = f"Bearer {token}"
    req = urllib.request.Request(url, headers=headers)
    with urllib.request.urlopen(req, timeout=20) as resp:
        return json.loads(resp.read().decode("utf-8"))


def evaluate_rc_reuse(
    tag: str,
    repo: str,
    token: str = "",
    head_ref: str = "HEAD",
    cwd: Optional[str] = None,
    dump_years: tuple[str, ...] = DEFAULT_DUMP_YEARS,
    api_get: Callable[[str, str], dict] = _default_api_get,
) -> ReuseDecision:
    if "-" in tag:
        return ReuseDecision(reason=f"{tag} is a pre-release tag; run full release pipeline")

    rc_tag = find_latest_rc_tag(tag, cwd=cwd)
    if not rc_tag:
        return ReuseDecision(reason=f"no matching {tag}-rc* tag found")

    sha_res = _run_git(["rev-parse", "--verify", f"{rc_tag}^{{commit}}"], cwd=cwd)
    if sha_res.returncode != 0:
        return ReuseDecision(reason=f"could not resolve commit for {rc_tag}")
    rc_sha = sha_res.stdout.strip()

    ok, why = diff_is_rc_promotion_only(rc_tag, head_ref=head_ref, cwd=cwd)
    if not ok:
        return ReuseDecision(
            source_rc=rc_tag,
            reason=f"diff since {rc_tag} is not version-only ({why}); run full release pipeline",
        )

    runs_url = (
        f"https://api.github.com/repos/{repo}/actions/workflows/release.yml/runs"
        f"?head_sha={rc_sha}&status=completed&per_page=20"
    )
    try:
        runs_data = api_get(runs_url, token)
    except (urllib.error.URLError, TimeoutError, ValueError, OSError) as exc:
        return ReuseDecision(
            source_rc=rc_tag,
            reason=f"GitHub Actions API query failed ({exc}); run full release pipeline",
        )

    green_run_id: Optional[int] = None
    for run in runs_data.get("workflow_runs", []):
        if (
            run.get("head_sha") == rc_sha
            and run.get("conclusion") == "success"
            and run.get("event") == "push"
            and run.get("head_branch") == rc_tag
        ):
            green_run_id = run.get("id")
            break

    if green_run_id is None:
        return ReuseDecision(
            source_rc=rc_tag,
            reason=f"no passing release.yml tag-push run found for {rc_tag} ({rc_sha[:10]})",
        )

    # All required tests (`ci` full + `windows-ci` require_guards + `dumps`) passed
    # on `rc_tag` and the diff since `rc_tag` is strictly Cargo.toml version pins / docs.
    artifacts_url = (
        f"https://api.github.com/repos/{repo}/actions/runs/{green_run_id}/artifacts"
        "?per_page=50"
    )
    reuse_dumps_run_id = ""
    dump_note = "dumps will be regenerated"
    try:
        art_data = api_get(artifacts_url, token)
        live_artifacts = {
            a.get("name"): a
            for a in art_data.get("artifacts", [])
            if not a.get("expired", True) and (a.get("size_in_bytes") or 0) > 0
        }
        expected = [f"dumps-{y}" for y in dump_years]
        missing = [name for name in expected if name not in live_artifacts]
        if not missing:
            reuse_dumps_run_id = str(green_run_id)
            dump_note = f"reusing {len(expected)} TL dump artifacts from run {green_run_id}"
        else:
            dump_note = f"dump artifacts unavailable/expired ({', '.join(missing)}); regenerating"
    except (urllib.error.URLError, TimeoutError, ValueError, OSError) as exc:
        dump_note = f"artifact query failed ({exc}); regenerating dumps"

    return ReuseDecision(
        skip_tests=True,
        reuse_dumps_run_id=reuse_dumps_run_id,
        source_rc=rc_tag,
        reason=(
            f"verified pure version/doc promotion from green {rc_tag} "
            f"(run {green_run_id}, {rc_sha[:10]}): skipping redundant ci/windows-ci; {dump_note}"
        ),
    )


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--tag", required=True, help="Release tag name (e.g. 0.8.0)")
    parser.add_argument("--repo", required=True, help="GitHub repository (owner/name)")
    parser.add_argument("--head", default="HEAD", help="Git ref for current release commit")
    parser.add_argument(
        "--dump-years",
        default=",".join(DEFAULT_DUMP_YEARS),
        help="Comma-separated TL dump years required for dump reuse",
    )
    parser.add_argument("--github-output", default="", help="Path to $GITHUB_OUTPUT")
    args = parser.parse_args(argv)

    token = os.environ.get("GITHUB_TOKEN", "")
    years = tuple(y.strip() for y in args.dump_years.split(",") if y.strip())
    decision = evaluate_rc_reuse(
        tag=args.tag,
        repo=args.repo,
        token=token,
        head_ref=args.head,
        dump_years=years,
    )

    print(
        f"rc-reuse-check: skip_tests={str(decision.skip_tests).lower()} "
        f"reuse_dumps_run_id={decision.reuse_dumps_run_id or '<none>'} "
        f"source_rc={decision.source_rc or '<none>'} — {decision.reason}"
    )

    if args.github_output:
        with open(args.github_output, "a", encoding="utf-8") as fh:
            fh.write(f"skip_tests={'true' if decision.skip_tests else 'false'}\n")
            fh.write(f"reuse_dumps_run_id={decision.reuse_dumps_run_id}\n")
            fh.write(f"source_rc={decision.source_rc}\n")

    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
