---
name: latexml-next-release
description: Prepare or execute a latexml-oxide release through its documented validation and publication gates. Use only when explicitly asked to cut, promote, publish, or audit a release, release candidate, crates.io publication, GitHub artifacts, containers, Homebrew formula, or production deployment.
---

# Cut a latexml-oxide release

This is a high-impact workflow. Read `docs/release/RELEASING.md`,
`docs/release/CRATES_IO_PUBLISH.md`, and `docs/release/RELEASE_CRITERIA.md` in full;
they are authoritative over this summary and imported memories. Inspect the
current release workflows and manifests before relying on historical asset counts
or platform lists.

Do not publish, tag, push, alter external registries, or deploy production unless
the user explicitly requested that action. Never expose credentials in logs or
documentation.

## Ordered gates

1. Verify the intended version/tag convention against the current workflow and
   `latexml_oxide/Cargo.toml`. Confirm whether the request is an RC or final.
2. Start from a green release commit. Run the full suite and the documented
   release-profile memory/build check without overlapping other heavy workloads.
3. Determine which publishable workspace crates changed. Because crates.io
   versions are immutable, update all required crate versions and every matching
   path-plus-version dependency consistently. Verify with Cargo metadata/checks.
4. Prepare the release notes and version changes on a focused branch/PR. Keep the
   release-prep and final-promotion history consistent with the current runbook.
5. Use an RC/draft artifact build as the platform risk gate when the current
   workflow supports it. Inspect every expected asset and verify the embedded
   binary version rather than trusting workflow success alone.
6. Promote the final version only after the RC evidence is acceptable. Create and
   push the exact tag only from the intended merged release commit.
7. Verify the public GitHub release, all platform artifacts, checksums, container
   version tags and moving tags, and the documentation deployment.
8. Update the Homebrew tap only from published final artifacts and verified
   checksums.
9. Publish crates.io last, after the release-artifact and documentation gates.
   Run the workspace dry-run first, then verify every published crate/version via
   the registry rather than relying only on command exit status.
10. Treat the ar5iv/latexml.rs production cutover as a separate, explicit manual
    deployment. Do not automate it unattended.

## Handoff and failure handling

At every pause, record the release commit, exact tag/version, completed gates,
workflow URLs, artifact/registry checks, commands actually run, and the first
unmet gate. A failed partial publication is external state: inspect what already
exists before retrying so immutable registries and moving tags are not corrupted
or misreported.

