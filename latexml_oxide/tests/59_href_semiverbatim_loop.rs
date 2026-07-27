//! `\href` inside a **Semiverbatim** argument must not infinite-loop.
//!
//! The other half of the defect [`58_href_edef_loop`](../58_href_edef_loop.rs)
//! guards. LaTeXML expands `\href{u}{t}` to
//! `\lx@hyper@url@\href{}{}{u}{t}` — the re-emitted `\href` exists only to fill
//! the constructor's reversion slot `#1`. PR "href protected" stopped the
//! `\edef`/`\xdef` re-expansion by marking `\href` `protected`, but ONE seam
//! legitimately expands protected macros: `Parameter::digest`'s semiverbatim
//! pre-expansion (Perl `Core/Parameter.pm` L123-132, "If semiverbatim, Expand
//! (before digest), so tokens can be neutralized") reads with
//! `fully_expand = true` (Perl `Core/Gullet.pm` L408-409). That pass linearizes
//! tokens one at a time and never reaches `\lx@hyper@url@`'s parameter list, so
//! it expanded the re-emitted `\href` as an ordinary macro — forever.
//!
//! Reached from a `.bib`: `\bib@field@default@doi` reads `Semiverbatim`, and
//! INSPIRE exports DOIs as `doi = {\href{https://doi.org/…}{…}}`. Witnesses
//! 2605.00181, 2605.19650, 2606.06645 — all three took
//! `Fatal:Timeout:Recursion` ("a window of 6 token(s) repeated 100+ times")
//! during the recursive bibliography session and lost the whole bibliography;
//! the fatal aborted the document. Perl `latexmlc` **hangs** on the same input
//! (rc=124 after 300 s on the 7-line reproducer), so this is a shared upstream
//! bug — see `docs/parity/KNOWN_PERL_ERRORS.md`.
//!
//! Fix: the reversion slot carries the command NAME as an OTHER-catcode token
//! instead of the live control sequence, exactly as the sibling `\url` path
//! (`\lx@hyper@url`) has always done. Inert to every expansion regime, and
//! stringifies/reverts identically — so the self-reference is structurally
//! impossible rather than dependent on a flag one seam is entitled to ignore.
mod cluster;
use cluster::convert_and_post_clean;

/// The runaway manifested as a `Fatal:` with no bibliography at all;
/// `convert_and_post_clean` asserts zero POST-stage `Error:` markers, which is
/// where the recursive `.bib` session reports (a core-only guard was blind to
/// this — see the helper's doc).
#[test]
fn href_in_semiverbatim_bib_field_does_not_loop() {
  let x = convert_and_post_clean("tests/cluster_regressions/bib_href_in_identifier_field.tex");

  // Both entries must survive. Before the fix the session died on the first
  // one and `MakeBibliography` fell back to no bibliography whatsoever.
  assert!(
    x.contains("<bibitem") || x.contains("bibitem"),
    "no bibliography at all — the \\href expansion loop likely re-triggered:\n{x}"
  );
  for needle in ["A Paper With A Wrapped DOI", "A Flux Concentrator"] {
    assert!(
      x.contains(needle),
      "{needle:?} missing from the bibliography:\n{x}"
    );
  }
  // The DOI field still produces its identifier element — the fix must not
  // have silenced the runaway by dropping the field. (What the identifier
  // *reads* is a separate, pre-existing matter: a link macro inside a
  // Semiverbatim field stringifies with its command name, and `\url` in the
  // same position has always done the same. Not asserted here.)
  // `MakeBibliography` rewrites `ltx:bib-identifier[@scheme='doi']` into the
  // entry's external link, so the surviving marker in POST output is the
  // `dx.doi.org` href it builds.
  assert!(
    x.contains("dx.doi.org"),
    "the wrapped DOI field produced no doi link:\n{x}"
  );
}
