//! japanese-otf's `ajmacros.sty` — a fast, explicit Fatal.
//!
//! ajmacros.sty:246-271 defines recursive kanji scanners (`\@aj半角def`,
//! `\@ajligaturedef`) whose control-sequence NAMES are ISO-2022-JP-encoded
//! kanji. Without pTeX's `\kcatcode` kanji token model (the parked pTeX/upTeX
//! family, DIFFICULT_CASES §D9) the names tokenize as `\@aj` + raw ESC bytes,
//! the delimited tail recursion never meets its `\@nil` sentinel, and
//! `\advance\@tempcnta` spins — an APERIODIC loop (`\number\@tempcnta` changes
//! every turn) that the cycle guard cannot see and only the 4 G token limit
//! stops, after ~250 s (platexsheet-jsclasses, sample-jsclasses, wtref-ja,
//! jpneduenumerate; Perl grinds the same way). A conversion that cannot
//! succeed says so in under a second instead: an out-of-scope bail, not a
//! kanji emulation (the parked-family rule). Reversible when §D9 is taken up.
//! Guard: `perfect_kernel_batch56::japanese_otf_kanji_scanners_bail_fast`.
use crate::prelude::*;

LoadDefinitions!({
  bail()?;
});

fn bail() -> Result<()> {
  Fatal!(
    Stomach,
    Unknown,
    "ajmacros.sty (japanese-otf) needs pTeX's kanji token model, which is not supported; the conversion cannot proceed (DIFFICULT_CASES §D9)"
  )
}
