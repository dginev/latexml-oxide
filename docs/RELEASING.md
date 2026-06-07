# Releasing

The release process for latexml-oxide is automated end-to-end: bump the
version, write the changelog, push a tag. GitHub Actions builds the
artifacts on `ubuntu-22.04` (glibc 2.35, broadest binary compatibility)
and publishes them to a new GitHub Release.

Currently supported platform: **`x86_64-unknown-linux-gnu`**. macOS,
Windows, aarch64, and musl are explicitly out of scope for now.

## What ships in a release

Four files attached to each `X.Y.Z` GitHub Release:

| Asset | Purpose |
|---|---|
| `latexml-oxide-X.Y.Z-x86_64-unknown-linux-gnu.tar.gz` | Portable archive: stripped `latexml_oxide` binary, `README.md`, `CHANGELOG.md`, `LICENSE`. |
| `latexml-oxide-X.Y.Z-x86_64-unknown-linux-gnu.tar.gz.sha256` | SHA-256 sidecar (ripgrep-style). |
| `latexml-oxide_X.Y.Z-1_amd64.deb` | Debian package with declared runtime `Depends:` on `libxml2`, `libxslt1.1`, `libkpathsea6`, `texlive-latex-{base,extra}`, `texlive-science`. |
| `latexml-oxide_X.Y.Z-1_amd64.deb.sha256` | SHA-256 sidecar. |

The shipped `latexml_oxide` binary is fully self-contained — XSLT
stylesheets, CSS, JavaScript, and the RelaxNG schema tree are
embedded at build time (`include_str!` / `include_bytes!`). Format
dumps for a **5-year moving TeX Live window** (currently 2022–2026)
are also embedded. They are NOT in git: `release.yml` first calls
`release-dumps.yml`, which generates each year's
`{plain,latex}.YYYY.dump.txt` + `texlive.YYYY.version` inside a pinned
TL-year container (`ghcr.io/tkw1536/texlive-docker:YYYY` — the image
family behind Perl LaTeXML's CI) under a strict zero-error `--init`
gate, then the release job downloads the full window into
`resources/dumps/` and verifies completeness before building. The
release workflow builds with `--profile maxperf` (`release.yml`), so a
single optimized, portable artifact is what users download.

**Design requirement — portability.** A conversion must not *read* any of
latexml_oxide's *own* resources from disk during its main operation: the
binary carries them all and serves them from memory (XSLT/CSS/JS/schema via
the `embed:///` libxml2 input callback, format dumps via `include_str!`).
*Writing* files into the destination directory is expected and fine. This is
verified end-to-end: XSLT resolves with zero `.xsl` disk reads (`strace`),
and renaming the dev-tree `resources/dumps/` away still converts successfully
from the embedded dumps. The host **TeX Live ecosystem is out of scope** —
reading `.sty`/`.cls`/`.tfm` from the user's texmf tree via `kpathsea` is
allowed and expected (see the runtime-dependency note below). See the
"Self-contained, portable binary" principle in
[`OXIDIZED_DESIGN.md`](OXIDIZED_DESIGN.md).

System libraries (`libxml2`, `libxslt1.1`, `libkpathsea6`) remain
dynamically linked — they're either installed by the `.deb`'s
`Depends:` or by the tarball user's own apt invocation. TeX Live
(`kpsewhich`, `pdflatex`) is required at runtime and not bundled.

## Release procedure

1. **Bump the version** in `latexml_oxide/Cargo.toml`:

    ```toml
    [package]
    version = "X.Y.Z"   # was: X.Y.(Z-1)
    ```

   Other workspace crates use their own version cadence and do not need
   bumping in lockstep.

2. **Add a CHANGELOG entry** at the top of `CHANGELOG.md`:

    ```markdown
    ## [X.Y.Z] YYYY-MM-DD

    - …summary of user-visible changes…
    ```

   `tools/make_release.sh` slices this section into the GitHub release
   body, falling back to a plain link to the full changelog if the
   matching header is missing.

3. **Sanity-check the release build locally** (catches build-script,
   `cargo-deb` metadata, and embedded-resource regressions before they
   reach CI):

    ```bash
    cargo install cargo-deb     # one-time per developer
    bash tools/make_release.sh
    ```

   On success the script prints the four artifact paths and their
   SHA-256 hashes. Spot-check the `.deb`:

    ```bash
    dpkg-deb -I target/release-artifacts/latexml-oxide_X.Y.Z-1_amd64.deb
    dpkg-deb -c target/release-artifacts/latexml-oxide_X.Y.Z-1_amd64.deb
    ```

4. **Commit, tag, push.** Tag format is bare `X.Y.Z` — no `v` prefix —
   matching the existing tag history.

    ```bash
    git commit -am "release: X.Y.Z"
    git tag X.Y.Z
    git push origin master
    git push origin X.Y.Z
    ```

5. **Watch the workflow.** `Release` on the Actions tab. Typical
   duration: 10–15 min cold, ~5 min warm. On success the new release
   appears at <https://github.com/dginev/latexml-oxide/releases/tag/X.Y.Z>.

## Failure recovery

* **Workflow fails before publishing.** Fix the underlying issue,
  delete the tag (`git tag -d X.Y.Z && git push origin :refs/tags/X.Y.Z`),
  push the fix, re-tag.

* **Workflow publishes a broken release.** Don't reuse the tag. Bump
  to `X.Y.(Z+1)` and start over — the deleted-then-replaced tag flow
  is confusing for downstream installers that may have already cached
  the broken assets.

* **`make_release.sh` rejects `GITHUB_REF_NAME`-vs-Cargo.toml mismatch.**
  This means you tagged before bumping. Bump `latexml_oxide/Cargo.toml`,
  amend the commit (or add a follow-up), and re-tag.

## Extending the workflow

* **New target triple** (e.g. `aarch64-unknown-linux-gnu`): add a
  matrix entry in `.github/workflows/release.yml` with the relevant
  cross-compile setup, and teach `tools/make_release.sh` to accept a
  target triple env var.

* **Skip the .deb**: comment out the `cargo deb …` block in
  `tools/make_release.sh` and drop the `.deb` lines from the
  `files:` list in `.github/workflows/release.yml`.

* **Embed additional resources**: add `include_str!` / `include_bytes!`
  hooks at the call site, then refresh the smoke test
  (`latexml_oxide/tests/001_single_binary_smoke.rs`) to assert the
  new asset appears.
