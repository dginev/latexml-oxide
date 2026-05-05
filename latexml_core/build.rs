//! Bakes the current git SHA into `LATEXML_GIT_SHA` so the
//! telemetry module can include it in every per-job record.
//! See `docs/TELEMETRY.md` §4 Step 5.

use std::process::Command;

fn main() {
  // Re-run when the git ref moves (commit, checkout, etc.)
  println!("cargo:rerun-if-changed=../.git/HEAD");
  println!("cargo:rerun-if-changed=../.git/refs/heads");
  // Also re-run if the env override changes
  println!("cargo:rerun-if-env-changed=LATEXML_GIT_SHA_OVERRIDE");

  let sha = std::env::var("LATEXML_GIT_SHA_OVERRIDE")
    .ok()
    .or_else(|| {
      Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|o| {
          if o.status.success() {
            String::from_utf8(o.stdout)
              .ok()
              .map(|s| s.trim().to_string())
          } else {
            None
          }
        })
    })
    .unwrap_or_default();

  println!("cargo:rustc-env=LATEXML_GIT_SHA={sha}");
}
