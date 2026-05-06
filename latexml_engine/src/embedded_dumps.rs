//! Compile-time embedded TeXLive kernel dumps, indexed by TeXLive year.
//!
//! `latexml_engine/build.rs` scans `resources/dumps/` for versioned dumps
//! (`plain.YYYY.dump.txt`, `latex.YYYY.dump.txt`, `texlive.YYYY.version`),
//! stages each one into `$OUT_DIR`, and emits
//! `embedded_dumps_manifest.rs` listing every bundled year. This module
//! `include!`s that manifest, exposing year-aware accessors.
//!
//! Selection at runtime:
//!
//! 1. If [`crate::dump_paths::detect_ambient_texlive_year`] returns a
//!    year that is bundled, use that exact year.
//! 2. Otherwise (no ambient TeXLive, or ambient year not bundled), fall
//!    back to the most-recent bundled year.
//! 3. If nothing is bundled at all, return `None`.
//!
//! Opt out of the embedded fallback altogether with
//! `LATEXML_NO_EMBEDDED_DUMP=1` — useful when iterating locally and you
//! want the binary to surface "no dump available" instead of silently
//! using a stale embedded snapshot.

use once_cell::sync::Lazy;

include!(concat!(env!("OUT_DIR"), "/embedded_dumps_manifest.rs"));

static NO_EMBEDDED: Lazy<bool> =
  Lazy::new(|| std::env::var_os("LATEXML_NO_EMBEDDED_DUMP").is_some());

/// Pick the embedded entry that best matches `prefer` (typically the
/// ambient TL year). Returns `(entry, exact_match)`.
pub(crate) fn select_embedded(prefer: Option<u32>) -> Option<(&'static EmbeddedDumpYear, bool)> {
  if *NO_EMBEDDED {
    return None;
  }
  let entries = non_empty_entries();
  if entries.is_empty() {
    return None;
  }
  if let Some(year) = prefer {
    if let Some(e) = entries.iter().find(|e| e.year == year) {
      return Some((e, true));
    }
  }
  // EMBEDDED_DUMPS is sorted descending by build.rs, so first non-empty is
  // the most-recent year.
  entries.first().copied().map(|e| (e, false))
}

fn non_empty_entries() -> Vec<&'static EmbeddedDumpYear> {
  EMBEDDED_DUMPS
    .iter()
    .filter(|e| !e.plain.is_empty() && !e.latex.is_empty())
    .collect()
}

/// Bundled `plain.YYYY.dump.txt` content for the year that best matches
/// `prefer`. `None` when nothing is bundled or `LATEXML_NO_EMBEDDED_DUMP`
/// is set.
pub fn embedded_plain_dump(prefer: Option<u32>) -> Option<&'static str> {
  let (entry, _) = select_embedded(prefer)?;
  Some(entry.plain)
}

/// Bundled `latex.YYYY.dump.txt` content for the year that best matches
/// `prefer`. See [`embedded_plain_dump`] for the conditions under which
/// this is `None`.
pub fn embedded_latex_dump(prefer: Option<u32>) -> Option<&'static str> {
  let (entry, _) = select_embedded(prefer)?;
  Some(entry.latex)
}

/// First line of the bundled `texlive.YYYY.version` stamp for the chosen
/// year (used by the staleness check). `None` if no embedded dump applies.
pub fn embedded_texlive_version_first_line(prefer: Option<u32>) -> Option<&'static str> {
  let (entry, _) = select_embedded(prefer)?;
  entry.stamp.lines().next()
}

/// Year-tag for the chosen embedded dump (used in log messages).
/// Returns `None` if no embedded dump applies.
pub fn embedded_year(prefer: Option<u32>) -> Option<u32> {
  let (entry, _) = select_embedded(prefer)?;
  Some(entry.year)
}

/// Whether at least one embedded plain+latex pair is bundled and the
/// opt-out env var isn't set.
pub fn plain_embedded_available() -> bool {
  embedded_plain_dump(None).is_some()
}

pub fn latex_embedded_available() -> bool {
  embedded_latex_dump(None).is_some()
}
