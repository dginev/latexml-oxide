---
name: next-release
disable-model-invocation: true
description: >
  Cut a new latexml-oxide release (RC or final) through the full validated,
  gated pipeline: bump the version, republish the crate graph, build the
  platform artifacts, and publish to every target (GitHub Release, ghcr.io
  containers, Homebrew tap, crates.io, ar5iv deploy) in the correct order with
  the correct gates. Encodes the hard-won sequencing and traps so a release is
  a checklist, not a re-derivation. Pairs with docs/release/RELEASING.md and
  docs/release/CRATES_IO_PUBLISH.md (the authoritative procedure + trap ledger).
  Invoke for "cut a release", "ship X.Y.Z", "release 0.7.6", "promote the rc",
  "publish to crates.io", "/next-release".
---

# Cutting a latexml-oxide release

`docs/release/RELEASING.md` is the authoritative procedure and `CRATES_IO_PUBLISH.md`
the trap ledger — read them. This skill is the **ordered, gated runbook** distilled
from the 0.7.5 cut, with the exact commands and the failure modes that bite.

## Ground rules

- **The tag string must equal `latexml_oxide/Cargo.toml`'s `version`.** `tools/make_release.sh`
  hard-fails on `GITHUB_REF_NAME != version`. Tags are **bare** (`0.7.5`, no `v` prefix).
- **RC vs final is decided purely by a `-` in the tag.** `release.yml`: a tag containing
  `-` (`0.7.5-rc6`) → **draft prerelease**, no containers/homebrew/crates. A bare tag
  (`0.7.5`) → **public Release**, auto-published, **and fires the `containers` job**.
- **Never publish an `-rc` to crates.io** (`cargo install` ignores prereleases →
  users stay on the old stable). Final only.
- **All repo edits go via branch + PR** (never commit to `main`). Release-prep and the
  promotion are each a PR.
- **`Cargo.lock` is gitignored** here — don't try to commit it.

## The two-gate rule for crates.io

crates.io is the LAST target and publishes only behind **both** gates on the FINAL tag:
- **Gate A** — the public GitHub Release shows **all platform binaries built/generated**.
- **Gate B** — the **gh-pages docs build is healthy** (`Deploy Docs to GitHub Pages`
  green on the release commit; crates.io's `documentation` link points at gh-pages).

## Sequence

### 0. Pre-flight
- Green `main` (`cargo nextest run --workspace`, N/0).
- **Toolchain check:** the fat-LTO `maxperf` release build OOMs on a nightly whose rustc
  RSS has regressed (#512, closed). `rust-toolchain.toml` floats on `nightly`, and CI +
  release resolve through it. Before cutting, confirm a cold `maxperf` build peaks in the
  single-digit-GB range; if it doesn't, pin the last good dated nightly in that file (its
  header comment gives the procedure) and reopen #512.

### 1. Subcrate version bumps (for the crates.io republish) — the easy-to-miss step
crates.io is **immutable**: you cannot republish an existing version with new bytes. Every
sibling crate that changed since the last publish needs a fresh version + matching dep pins,
or `cargo publish --workspace` publishes only the top crate against **stale** siblings →
downstream build breakage. Check each sibling's diff since the last release tag and bump
honest 0.x semver (minor = API-affecting, patch = internal-only). Update **every**
intra-workspace `path`+`version` pin (`latexml_oxide` + inter-crate). `cargo set-version`
(cargo-edit) automates the cascade if installed; otherwise script it and verify with
`cargo metadata` + `cargo check --workspace`.

### 2. Release-prep PR → RC tag (validation)
Bump `latexml_oxide/Cargo.toml` to `X.Y.Z-rcN`, add the CHANGELOG window (the 0.7.x line
uses `## [X.Y.Z] (theme summary)` with **no date**; `make_release.sh` slices on `^## [X.Y.Z]`),
land the subcrate bumps. PR → green CI (validates the pin resolves) → merge → tag `X.Y.Z-rcN`
→ `release.yml` builds a **draft** across all 5 platforms. This is the real risk gate:
**macOS x86_64 (Intel) fat-LTO is the OOM long pole** — confirm it's green before promoting.
Verify the draft has all 15 assets: `gh release view X.Y.Z-rcN --json assets`.

### 3. Promotion PR → final tag (public)
Tiny commit: `Cargo.toml` `-rcN → X.Y.Z`; refresh the README (`VERSION=` install samples,
the hardcoded `ported tests-N` badge → current `nextest` count). The dynamic `release`
badge auto-flips; don't touch CI/license/arXiv badges. PR → green → merge. Then tag bare:
```bash
git tag -a X.Y.Z -m "…" <merged-sha> && git push origin X.Y.Z
```
`release.yml` re-builds the matrix, **auto-publishes the public Release**, and fires containers.

### 4. Verify Gate A + containers + Gate B
- `gh release view X.Y.Z --json isDraft,assets` → `isDraft=false`, 15 assets.
- **ghcr `:latest` trap (0.7.4 shipped with none):** confirm `:latest` moved:
  ```bash
  docker manifest inspect ghcr.io/dginev/latexml-oxide:latest | sha256sum
  docker manifest inspect ghcr.io/dginev/latexml-oxide:X.Y.Z  | sha256sum   # must match
  ```
  Worker image: `ghcr.io/dginev/latexml-oxide/cortex-worker:X.Y.Z` (amd64). If `:latest`
  didn't move, `docker.yml` has a `workflow_dispatch`.
- Gate B: `Deploy Docs to GitHub Pages` green on the release commit.

### 5. Homebrew tap
```bash
gh repo clone dginev/homebrew-tap && cd homebrew-tap
./update-formula.sh X.Y.Z        # pulls the release's macOS .sha256 sidecars
git -c user.name="Deyan Ginev" -c user.email="deyan.ginev@gmail.com" commit -am "latexml-oxide X.Y.Z"
git push
```
A fresh clone has **no git identity** — set it inline or the commit fails. "Don't forget the
tap": skipping it silently leaves `brew` users on the old version.

### 6. crates.io (LAST, both gates green)
Pre-validate: `cargo publish --dry-run --workspace` (all 8 package/compile/verify bottom-up,
zero errors; the `ignoring test … not included` warnings are the deliberate `exclude=["tests/"]`).
Traps: **B7** the 10 MiB cap is enforced only on real upload, not dry-run — `exclude=["tests/"]`
keeps the top crate ~600 KB; **B8** the 5-new-crate burst limit is N/A for a *re*-publish of
existing crates; **B3/B6** the `latexml_oxide/README.md → ../README.md` symlink must stay.
Repo must be **public**, `~/.cargo/credentials.toml` present. Then, in the **background**
(index waits make it long; a foreground timeout could kill it mid-publish):
```bash
cargo publish --workspace
```
Verify all crates landed against the API (exit 0 alone isn't proof):
```bash
curl -s -H "User-Agent: x" https://crates.io/api/v1/crates/<name> | grep -o '"max_version":"[^"]*"'
```

### 7. ar5iv-editor deploy → latexml.rs
**Manual production cutover** ([[ar5iv-editor-deploy-latexml-rs]] — `release.sh --push`, box
cutover via `ssh root@latexml.rs`). Do **not** automate this unattended.

## Future optimization — avoid the double build
Today an RC and the final each rebuild the whole fat-LTO matrix (~2× the slow part). You
can't relabel RC artifacts — the version string is baked into every binary (`--version`),
asset name, `.deb`, and sha256. The single-build fix: make `release.yml` create the **final**
tag as a **draft** first (today it drafts only on a `-` tag), build **once** → correctly-named
`X.Y.Z` assets, validate the draft, then `gh release edit X.Y.Z --draft=false` (flips
visibility, no rebuild). Re-gate the `containers` job on `release: published` (which
`docker.yml` already supports) so images build at publish, not during the draft build.
