#!/usr/bin/env python3
"""Offline unit tests for tools/check_rc_reuse.py."""

from __future__ import annotations

import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import check_rc_reuse as rc  # noqa: E402


def _git(cwd: Path, *args: str) -> str:
    res = subprocess.run(
        ["git", *args],
        cwd=str(cwd),
        check=True,
        capture_output=True,
        text=True,
    )
    return res.stdout.strip()


def _make_repo(root: Path) -> None:
    _git(root, "init", "-q")
    _git(root, "config", "user.email", "test@example.com")
    _git(root, "config", "user.name", "Test")
    (root / "latexml_core").mkdir()
    (root / "latexml_oxide").mkdir()
    (root / "docs").mkdir()
    (root / "latexml_core" / "Cargo.toml").write_text(
        '[package]\nname = "latexml_core"\nversion = "0.7.0-rc1"\n',
        encoding="utf-8",
    )
    (root / "latexml_oxide" / "Cargo.toml").write_text(
        '[package]\nname = "latexml"\nversion = "0.8.0-rc1"\n\n'
        "[dependencies]\n"
        'serde = "1.0.229"\n'
        'latexml_core = {path = "../latexml_core", version = "0.7.0-rc1"}\n',
        encoding="utf-8",
    )
    (root / "latexml_core" / "lib.rs").write_text("pub fn f() {}\n", encoding="utf-8")
    (root / "CHANGELOG.md").write_text("# Changelog\n\n## [0.8.0]\n", encoding="utf-8")
    _git(root, "add", ".")
    _git(root, "commit", "-q", "-m", "rc1")
    _git(root, "tag", "0.8.0-rc1")


def _fake_api(
    rc_sha: str,
    rc_tag: str = "0.8.0-rc1",
    conclusion: str = "success",
    run_id: int = 12345,
    expired_years: tuple[str, ...] = (),
):
    def get(url: str, _token: str) -> dict:
        if "/actions/workflows/release.yml/runs" in url:
            return {
                "workflow_runs": [
                    {
                        "id": run_id,
                        "head_sha": rc_sha,
                        "head_branch": rc_tag,
                        "event": "push",
                        "conclusion": conclusion,
                    }
                ]
            }
        if f"/actions/runs/{run_id}/artifacts" in url:
            return {
                "artifacts": [
                    {
                        "name": f"dumps-{y}",
                        "expired": y in expired_years,
                        "size_in_bytes": 1024,
                    }
                    for y in rc.DEFAULT_DUMP_YEARS
                ]
            }
        raise AssertionError(f"unexpected URL: {url}")

    return get


class CheckRcReuseTests(unittest.TestCase):
    def test_prerelease_tag_never_skips(self) -> None:
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            _make_repo(root)
            d = rc.evaluate_rc_reuse(
                "0.8.0-rc1", "o/r", cwd=str(root), api_get=lambda *_: {}
            )
            self.assertFalse(d.skip_tests)
            self.assertEqual(d.reuse_dumps_run_id, "")

    def test_missing_rc_tag_never_skips(self) -> None:
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            _make_repo(root)
            d = rc.evaluate_rc_reuse(
                "0.9.0", "o/r", cwd=str(root), api_get=lambda *_: {}
            )
            self.assertFalse(d.skip_tests)
            self.assertEqual(d.reuse_dumps_run_id, "")

    def test_pure_version_and_doc_promotion_reuses_tests_and_dumps(self) -> None:
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            _make_repo(root)
            rc_sha = _git(root, "rev-parse", "0.8.0-rc1^{commit}")
            (root / "latexml_core" / "Cargo.toml").write_text(
                '[package]\nname = "latexml_core"\nversion = "0.7.0"\n',
                encoding="utf-8",
            )
            (root / "latexml_oxide" / "Cargo.toml").write_text(
                '[package]\nname = "latexml"\nversion = "0.8.0"\n\n'
                "[dependencies]\n"
                'serde = "1.0.229"\n'
                'latexml_core = {path = "../latexml_core", version = "0.7.0"}\n',
                encoding="utf-8",
            )
            (root / "CHANGELOG.md").write_text(
                "# Changelog\n\n## [0.8.0]\nFinal notes.\n", encoding="utf-8"
            )
            _git(root, "add", ".")
            _git(root, "commit", "-q", "-m", "release 0.8.0")

            d = rc.evaluate_rc_reuse(
                "0.8.0", "o/r", cwd=str(root), api_get=_fake_api(rc_sha)
            )
            self.assertTrue(d.skip_tests, d.reason)
            self.assertEqual(d.reuse_dumps_run_id, "12345")
            self.assertEqual(d.source_rc, "0.8.0-rc1")

    def test_expired_dump_artifact_skips_tests_but_regenerates_dumps(self) -> None:
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            _make_repo(root)
            rc_sha = _git(root, "rev-parse", "0.8.0-rc1^{commit}")
            (root / "latexml_oxide" / "Cargo.toml").write_text(
                '[package]\nname = "latexml"\nversion = "0.8.0"\n\n'
                "[dependencies]\n"
                'serde = "1.0.229"\n'
                'latexml_core = {path = "../latexml_core", version = "0.7.0-rc1"}\n',
                encoding="utf-8",
            )
            _git(root, "add", ".")
            _git(root, "commit", "-q", "-m", "release 0.8.0")

            d = rc.evaluate_rc_reuse(
                "0.8.0",
                "o/r",
                cwd=str(root),
                api_get=_fake_api(rc_sha, expired_years=("2022",)),
            )
            self.assertTrue(d.skip_tests, d.reason)
            self.assertEqual(d.reuse_dumps_run_id, "")

    def test_failed_rc_run_never_skips(self) -> None:
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            _make_repo(root)
            rc_sha = _git(root, "rev-parse", "0.8.0-rc1^{commit}")
            (root / "latexml_oxide" / "Cargo.toml").write_text(
                '[package]\nname = "latexml"\nversion = "0.8.0"\n\n'
                "[dependencies]\n"
                'serde = "1.0.229"\n'
                'latexml_core = {path = "../latexml_core", version = "0.7.0-rc1"}\n',
                encoding="utf-8",
            )
            _git(root, "add", ".")
            _git(root, "commit", "-q", "-m", "release 0.8.0")

            d = rc.evaluate_rc_reuse(
                "0.8.0",
                "o/r",
                cwd=str(root),
                api_get=_fake_api(rc_sha, conclusion="failure"),
            )
            self.assertFalse(d.skip_tests)
            self.assertEqual(d.reuse_dumps_run_id, "")

    def test_rust_source_change_forces_full_pipeline(self) -> None:
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            _make_repo(root)
            rc_sha = _git(root, "rev-parse", "0.8.0-rc1^{commit}")
            (root / "latexml_core" / "lib.rs").write_text(
                "pub fn f() { /* fix */ }\n", encoding="utf-8"
            )
            _git(root, "add", ".")
            _git(root, "commit", "-q", "-m", "fix")

            d = rc.evaluate_rc_reuse(
                "0.8.0", "o/r", cwd=str(root), api_get=_fake_api(rc_sha)
            )
            self.assertFalse(d.skip_tests)
            self.assertEqual(d.reuse_dumps_run_id, "")

    def test_external_dep_bump_forces_full_pipeline(self) -> None:
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            _make_repo(root)
            rc_sha = _git(root, "rev-parse", "0.8.0-rc1^{commit}")
            (root / "latexml_oxide" / "Cargo.toml").write_text(
                '[package]\nname = "latexml"\nversion = "0.8.0"\n\n'
                "[dependencies]\n"
                'serde = "1.0.230"\n'
                'latexml_core = {path = "../latexml_core", version = "0.7.0-rc1"}\n',
                encoding="utf-8",
            )
            _git(root, "add", ".")
            _git(root, "commit", "-q", "-m", "dep bump")

            d = rc.evaluate_rc_reuse(
                "0.8.0", "o/r", cwd=str(root), api_get=_fake_api(rc_sha)
            )
            self.assertFalse(d.skip_tests)
            self.assertEqual(d.reuse_dumps_run_id, "")

    def test_workspace_dep_feature_change_forces_full_pipeline(self) -> None:
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            _make_repo(root)
            rc_sha = _git(root, "rev-parse", "0.8.0-rc1^{commit}")
            (root / "latexml_oxide" / "Cargo.toml").write_text(
                '[package]\nname = "latexml"\nversion = "0.8.0"\n\n'
                "[dependencies]\n"
                'serde = "1.0.229"\n'
                'latexml_core = {path = "../latexml_core", version = "0.7.0", default-features = false}\n',
                encoding="utf-8",
            )
            _git(root, "add", ".")
            _git(root, "commit", "-q", "-m", "feature tweak")

            d = rc.evaluate_rc_reuse(
                "0.8.0", "o/r", cwd=str(root), api_get=_fake_api(rc_sha)
            )
            self.assertFalse(d.skip_tests)
            self.assertEqual(d.reuse_dumps_run_id, "")

    def test_latest_rc_is_selected_over_older_green_rc(self) -> None:
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            _make_repo(root)
            rc1_sha = _git(root, "rev-parse", "0.8.0-rc1^{commit}")
            (root / "latexml_core" / "lib.rs").write_text(
                "pub fn f() { let _ = 1; }\n", encoding="utf-8"
            )
            _git(root, "add", ".")
            _git(root, "commit", "-q", "-m", "rc2")
            _git(root, "tag", "0.8.0-rc2")
            (root / "latexml_oxide" / "Cargo.toml").write_text(
                '[package]\nname = "latexml"\nversion = "0.8.0"\n\n'
                "[dependencies]\n"
                'serde = "1.0.229"\n'
                'latexml_core = {path = "../latexml_core", version = "0.7.0-rc1"}\n',
                encoding="utf-8",
            )
            _git(root, "add", ".")
            _git(root, "commit", "-q", "-m", "final")

            # API only has green run for rc1, NOT rc2.
            d = rc.evaluate_rc_reuse(
                "0.8.0", "o/r", cwd=str(root), api_get=_fake_api(rc1_sha, rc_tag="0.8.0-rc1")
            )
            self.assertEqual(d.source_rc, "0.8.0-rc2")
            self.assertFalse(d.skip_tests)


if __name__ == "__main__":
    unittest.main()
