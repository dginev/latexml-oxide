//! Cluster regressions in bibliography handling — `.bib`/`.bbl` reading, the
//! recursive BibTeX session, citation styles and rendered `References`.
//!
//! Split out of `06_cluster_regressions`; shares its helpers via
//! [`mod cluster`](cluster).

mod cluster;
use cluster::{
  convert_and_post, convert_and_post_clean, convert_to_xml, convert_to_xml_ar5iv,
  convert_to_xml_contrib, convert_to_xml_contrib_clean,
};

/// bbl/bib precedence matrix for `\lx@ifusebbl` (latex_constructs.rs) — the
/// decision seam behind `\bibliography`. The clauses are arbitrary tokens, so
/// marker text pins WHICH phase was chosen without running the full BibTeX
/// pipeline. Covers the cb8b648784 fallback (bbl-first config + no .bbl on
/// disk → use the real .bib) and Perl's first-phase-only rule.
#[test]
fn cluster_bbl_bib_precedence() {
  // Default config ['bib','bbl']: refs.bib AND <jobname>.bbl both exist —
  // the bib phase is first and all bibs exist → BIB wins.
  let x = convert_to_xml("tests/cluster_regressions/bblbib/both.tex");
  assert!(
    x.contains("BIBCHOSEN") && !x.contains("BBLCHOSEN"),
    "default config with both files should choose bib, got:\n{x}"
  );
  // Default config, requested norefs.bib is MISSING but <jobname>.bbl exists
  // → falls to the bbl clause (Perl: "Couldn't find all bib files").
  let x = convert_to_xml("tests/cluster_regressions/bblbib/bblwins.tex");
  assert!(
    x.contains("BBLCHOSEN") && !x.contains("BIBCHOSEN"),
    "default config with missing .bib should choose bbl, got:\n{x}"
  );
  // nobibtex config ['bbl'] with <jobname>.bbl on disk → BBL wins,
  // even though refs.bib also exists.
  let x = convert_to_xml("tests/cluster_regressions/bblbib/bblfirst.tex");
  assert!(
    x.contains("BBLCHOSEN") && !x.contains("BIBCHOSEN"),
    "nobibtex config with .bbl present should choose bbl, got:\n{x}"
  );
  // nobibtex config ['bbl'] and NO <jobname>.bbl: Perl's first-phase-only
  // rule — no 'bib' phase configured, so NEITHER clause fires (empty +
  // Info:expected:bbl), not a spurious empty bibliography.
  let x = convert_to_xml("tests/cluster_regressions/bblbib/bblnone.tex");
  assert!(
    !x.contains("BBLCHOSEN") && !x.contains("BIBCHOSEN"),
    "nobibtex config without .bbl should choose neither, got:\n{x}"
  );
  // ar5iv profile (bibconfig=bbl,bib) but NO <jobname>.bbl: falls through to
  // the configured bib phase because refs.bib exists (cb8b648784; witness
  // 2605.16562 — refs.bib and no .bbl under the ar5iv fleet profile).
  let x = convert_to_xml_ar5iv("tests/cluster_regressions/bblbib/bblfallback.tex");
  assert!(
    x.contains("BIBCHOSEN") && !x.contains("BBLCHOSEN"),
    "ar5iv bbl-first config without .bbl should fall back to bib, got:\n{x}"
  );
}
/// A biber `.bbl` with more than one `\datalist` (biblatex's apa style asks for
/// two sorting schemes, so the same references are emitted twice) used to hang
/// the engine: each `\enddatalist` expands to a bare
/// `\thebibliography…\endthebibliography`, neither of which opens a group, so
/// the second one re-entered `setupPseudoBibitem` while the first arming was
/// live and captured `\save@bibitem` ← `\restoring@bibitem` — a self-referential
/// `\let` that expands forever (`Fatal:Timeout:TokenLimit`, 1e9 tokens).
/// The blank line after `\printbibliography` covers the second half of the fix:
/// `\endthebibliography` now disarms the redirection, so that `\par` no longer
/// expands to `\par@in@bibliography` and deposits a stray empty bibitem outside
/// the biblist. Witness: arXiv 2605.17646 (Perl converts it — its biblatex
/// binding never defines `\printbibliography`, so upstream never reaches this —
/// but Perl hangs identically on the bare-CS form; KNOWN_PERL_ERRORS #57).
#[test]
fn cluster_biblatex_two_datalists() {
  let x =
    convert_to_xml_contrib_clean("tests/cluster_regressions/biblatex_two_datalists/twolists.tex");
  // One bibliography per \datalist, each holding its own biblist.
  assert_eq!(
    x.matches("<bibliography").count(),
    2,
    "expected one <bibliography> per \\datalist:\n{x}"
  );
  assert_eq!(
    x.matches("<biblist>").count(),
    2,
    "expected one <biblist> per \\datalist:\n{x}"
  );
  // 2 entries × 2 datalists, and NOT a 5th stray from the trailing blank line.
  assert_eq!(
    x.matches("<bibitem").count(),
    4,
    "expected exactly 4 bibitems (2 entries x 2 datalists, no stray):\n{x}"
  );
}
/// arXiv/html_feedback#6797 — an author-year bibliography built from a `.bib`
/// used the FULL author list as the entry's refnum LABEL (5104 characters on the
/// witness, arXiv 2607.21432); and because the author-year branch also skipped
/// the first block, the authors appeared ONLY there.
///
/// pdflatex is the ground truth for the shape: `aa.bst` over the witness emits
/// `\bibitem[{Abitbol {et~al.}(2025)Abitbol, …all surnames…}]` — natbib's SHORT
/// form is the citation label, the long list is only natbib's optional
/// full-author form and is never printed — and shows the authors in the entry
/// BODY. Perl is byte-identical to the old Rust (`do_names` truncates only for a
/// literal BibTeX `and others`), so this is a deliberate divergence,
/// OXIDIZED_DESIGN #71.
#[test]
fn cluster_bib_long_author_list_refnum() {
  let x = convert_and_post("tests/cluster_regressions/bib_long_author_list.tex");
  // The refnum tag of the <bibitem> that carries a given entry's title. NOTE
  // the refnum is pushed LAST within <tags>, so it follows the title — scope to
  // the enclosing bibitem rather than searching backwards from the title.
  let refnum = |title: &str| -> String {
    let i = x
      .find(title)
      .unwrap_or_else(|| panic!("no entry titled {title} in:\n{x}"));
    let start = x[..i].rfind("<bibitem").unwrap_or(0);
    let end = x[start..]
      .find("</bibitem>")
      .map(|e| start + e)
      .unwrap_or(x.len());
    let s = start
      + x[start..end]
        .find(r#"role="refnum""#)
        .unwrap_or_else(|| panic!("no refnum in the bibitem for {title}:\n{x}"));
    let open = x[s..].find('>').expect("tag open") + s + 1;
    let close = x[open..].find("</tag>").expect("tag close") + open;
    let (mut out, mut depth) = (String::new(), 0);
    for c in x[open..close].chars() {
      match c {
        '<' => depth += 1,
        '>' => depth -= 1,
        _ if depth == 0 => out.push(c),
        _ => {},
      }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
  };
  // >2 authors: the label is natbib's short form, NOT the full list.
  assert_eq!(
    refnum("science goals and forecasts"),
    "Abitbol et al. (2025)",
    "long-author label:\n{x}"
  );
  // 2 and 1 author are unaffected by the short form.
  assert_eq!(
    refnum("On examples"),
    "Jones and Brown (2019)",
    "2-author:\n{x}"
  );
  assert_eq!(
    refnum("A single-author paper"),
    "Berg (2018)",
    "1-author:\n{x}"
  );
  // A literal BibTeX `and others` already produced "et al." — still does.
  //
  // The needle is the SENTENCE-CASED title ("BibTeX" -> "bibtex"): `.bib` field
  // values now go through the engine's `\bib@@title`, which re-cases per
  // `BibTeX_title_case` exactly as Perl does (BibTeX.pool.ltxml L281-333). The
  // deleted string route left the author's capitalization alone, so this needle
  // used to read "BibTeX". Verified against same-host `latexmlc`: all four
  // rendered titles are byte-identical between the engines.
  assert_eq!(
    refnum("An explicit bibtex others entry"),
    "Smith et al. (2020)",
    "explicit others:\n{x}"
  );
  // The full author list must NOT be lost: with a short label the first block
  // (the authors) is no longer redundant and must appear in the entry body.
  assert!(
    x.contains("Abril-Cabezas") && x.contains("Agrawal"),
    "the full author list must survive in the entry body:\n{x}"
  );
  // ...but the RENDERED first block must not re-print the year the label
  // already carries. Perl dropped the whole block to avoid that redundancy; we
  // keep it and drop only its year field, matching the shipped biblatex
  // author-year rendering (`[Smith (2020)] John Smith “A study…”`).
  //
  // Scoped to the first <bibblock> on purpose: `2025` also legitimately occurs
  // as the `<tags>` metadata year (CrossRef reads it, and the numeric style
  // emits it too), in `key="Collab2025"`, and as this entry's journal VOLUME.
  let first_block = {
    let i = x.find("science goals and forecasts").expect("entry");
    let start = x[..i].rfind("<bibitem").expect("bibitem start");
    let b = start + x[start..].find("<bibblock").expect("first bibblock");
    let e = b + x[b..].find("</bibblock>").expect("bibblock end");
    &x[b..e]
  };
  assert!(
    first_block.contains(r#"class="ltx_bib_author""#),
    "the first block should carry the authors:\n{first_block}"
  );
  assert!(
    !first_block.contains(r#"class="ltx_bib_year""#),
    "the first block must not re-emit the year the label already carries:\n{first_block}"
  );
}
/// biblatex author-year support (ar5iv-bindings PRs #20/#21 + repair
/// 0911aec): style=apa documents with a biber .bbl get "Surname, Year"
/// labels, one schema-valid role-tagged <ltx:tags> per bibitem, and the
/// three citation families; style=numeric documents keep sequential
/// labels, core [ ] brackets, and plain-\cite fallbacks (multicite keys
/// comma-joined).
#[test]
fn cluster_biblatex_authoryear() {
  let x = convert_to_xml_contrib("tests/cluster_regressions/biblatex_ay/ay.tex");
  // Structured tags with author/year roles (single-author, 2-author "&",
  // 3+-author "et al." short form vs full list, prefix-name surname).
  assert!(
    x.contains(r#"<tag role="year">2020</tag>"#),
    "year tag missing:\n{x}"
  );
  assert!(
    x.contains(r#"<tag role="authors">Smith</tag>"#),
    "authors tag missing:\n{x}"
  );
  assert!(
    x.contains(r#"<tag role="refnum">Smith (2020)</tag>"#),
    "refnum tag missing:\n{x}"
  );
  assert!(
    x.contains(r#"<tag role="authors">Jones &amp; Brown</tag>"#),
    "2-author tag missing:\n{x}"
  );
  assert!(
    x.contains(r#"<tag role="authors">Adams et al.</tag>"#),
    "et-al short form missing:\n{x}"
  );
  assert!(
    x.contains(r#"<tag role="fullauthors">Adams, Baker &amp; Clark</tag>"#),
    "fullauthors missing:\n{x}"
  );
  assert!(
    x.contains(r#"<tag role="authors">Berg</tag>"#),
    "prefix-name surname missing:\n{x}"
  );
  // Citation families: parenthetical vs textual vs bare, with show= specs.
  assert!(
    x.contains("citemacro_citep"),
    "parenthetical cite class missing:\n{x}"
  );
  assert!(
    x.contains("citemacro_citet"),
    "textual cite class missing:\n{x}"
  );
  assert!(
    x.contains(r#"show="Authors Phrase1YearPhrase2""#),
    "textual show spec missing:\n{x}"
  );
  assert!(
    x.contains(r#"show="FullAuthorsPhrase1Year""#),
    "starred full-author show missing:\n{x}"
  );
  // Multicite: two bibrefs inside one cite, "; "-joined.
  assert!(
    x.contains(r#"bibrefs="smith2020""#) && x.contains(r#"bibrefs="jones2019""#),
    "multicite per-group bibrefs missing:\n{x}"
  );
  // arxiv-readability#10 / ar5iv-bindings#4: \parencite[see][]{key} — a
  // present-but-EMPTY second optional must NOT demote the prenote to a
  // postnote ("(see Smith, 2020)", never "(Smith, 2020, see)").
  assert!(
    x.matches("(see ").count() >= 2,
    "issue-4 prenote missing:\n{x}"
  );
  assert!(
    !x.contains(", see)"),
    "issue-4 prenote demoted to postnote:\n{x}"
  );

  let x = convert_to_xml_contrib("tests/cluster_regressions/biblatex_ay/num.tex");
  // Numeric style: sequential labels, NO author-year relabeling, and the
  // fallback \cite path (keys preserved; multicite keys comma-joined).
  assert!(
    x.contains(r#"bibrefs="smith2020""#),
    "numeric fallback lost keys:\n{x}"
  );
  assert!(
    x.contains(r#"bibrefs="smith2020,jones2019""#),
    "numeric multicite keys not comma-joined:\n{x}"
  );
  assert!(
    !x.contains("Smith, 2020"),
    "numeric doc must not get author-year labels:\n{x}"
  );
  assert!(
    !x.contains(r#"role="fullauthors""#),
    "numeric doc must not get author-year tags:\n{x}"
  );
}
/// apacite spells its citation pre-note in ANGLE brackets:
/// `\cite<pre-note>[post-note]{key-list}` (apacite.sty L259-311 dispatch
/// `\@ifnextchar< {\@cite} {\@cite<>}`, L313-327 `\def\@cite<#1>`). Without that
/// form the kernel/natbib `\cite` takes the single token `<` as its whole key
/// list: the citation renders as a dangling `[<]`, the REAL keys are never cited
/// (so they are silently absent from the References) and `see>` leaks into the
/// body text. Witness 2605.10951 (`\cite<see>{Gangopadhyay02,Ferris25}`,
/// agujournal2019), 2606.16518, 2606.19048, 2606.21531, 2606.24563.
///
/// Guards BOTH halves of the fix: the pre-note form resolves its keys, AND the
/// pre-note-ABSENT case does not swallow a later `>`. The latter is why this
/// uses the real `OptionalAngled` parameter type rather than
/// `OptionalMatch:< OptionalUntil:>` — `Until` never checks for the OPENING
/// delimiter, so with no `<` it scanned to the next `>` anywhere downstream and
/// `\citeA{Gangopadhyay02} and $a > b$` reported the key as `b`.
#[test]
fn apacite_angled_prenote_cites_keys_and_does_not_swallow_gt() {
  let x = convert_to_xml_contrib("tests/cluster_regressions/cite_angled_prenote/ap.tex");
  // `\cite<see>{Gangopadhyay02,Ferris25}` cites BOTH real keys, not `<`.
  assert!(
    x.contains("Gangopadhyay02,Ferris25"),
    "\\cite<see>{{...}} lost its keys (apacite angle-bracket pre-note):\n{x}"
  );
  assert!(
    !x.contains(r#"bibrefs="&lt;""#) && !x.contains(r#"bibrefs="<""#),
    "`<` was parsed as the citation key list:\n{x}"
  );
  // Pre-note absent + a later `>`: the cite keeps its key and the math survives.
  assert!(
    !x.contains(r#"bibrefs="b""#),
    "an absent angle pre-note swallowed the cite and the following `$a > b$`:\n{x}"
  );
}
/// Real sn-jnl.cls loads natbib for EVERY reference style (L1649/1652/1662/…:
/// `\usepackage[numbers,sort&compress]{natbib}` / `\usepackage[authoryear]{natbib}`),
/// but our binding `LoadClass!("OmniBus")`es — which short-circuits the
/// unbound-class dependency scan — and OmniBus only `def_autoload`s natbib off
/// `\citet`/`\citep`/`\citeyear`/…, deliberately NOT off `\cite` (the kernel
/// already defines it). So a paper citing solely via natbib's TWO-optional
/// `\cite[pre][post]{keys}` never triggered the autoload and the kernel's
/// single-optional `\cite[] Semiverbatim` read `[` as the whole key list — the
/// real keys were dropped (silently absent from the References) and `]{keys}`
/// leaked as body text. Witness 2605.23484 (sn-mathphys-num), 2606.10002
/// (sn-basic), 2606.10215, 2606.11534.
#[test]
fn sn_jnl_natbib_two_optional_cite_keeps_its_keys() {
  let x = convert_to_xml_contrib("tests/cluster_regressions/sn_jnl_cite/sn.tex");
  assert!(
    x.contains("Melrose1980"),
    "sn-jnl `\\cite[e.g.][]{{Melrose1980}}` lost its key (natbib not loaded):\n{x}"
  );
  assert!(
    !x.contains(r#"bibrefs="[""#) && !x.contains(r#"bibrefs="&#91;""#),
    "`[` was parsed as the citation key list — natbib's two-optional \
     \\cite[pre][post]{{keys}} did not parse:\n{x}"
  );
  assert!(
    x.contains("Zhang2021"),
    "`\\cite[see][chap.~2]{{Zhang2021}}` lost its key:\n{x}"
  );
}
/// amsrefs writes the bibliography INTO the document —
/// `\begin{bibdiv}\begin{biblist}\bib{key}{article}{...}` — instead of into an
/// external `.bib`. The engine digests that correctly into
/// `ltx:biblist`/`ltx:bibentry` (see the `amsrefs_basic` structure test), but
/// upstream `MakeBibliography::getBibEntries` collects entries ONLY from
/// `getBibliographies()`, which resolves `//ltx:bibliography/@files` — an
/// amsrefs bibliography has no `@files`, so nothing is collected, and `process`
/// then executes its unconditional `removeNodes(//ltx:bibentry)`, deleting every
/// entry it never converted. The whole bibliography vanishes with ZERO errors:
/// empty References plus every `\cite` dangling.
///
/// PARITY with installed AND vendored Perl 0.8.8 (rev 51fea96a) — fixed here
/// rather than reproduced (OXIDIZED_DESIGN #57, KNOWN_PERL_ERRORS #49).
/// Witness 2605.01646 (AIPFa.tex; Perl: 0 bibitems / 81 dangling citations,
/// Rust now 23 / 0), 2605.00783, 2605.03852.
///
/// NOTE the structure test `amsrefs_basic` asserts only on the ENGINE's XML and
/// so never exercised MakeBibliography — which is exactly how this stayed
/// silent. This test runs the full pipeline.
#[test]
fn amsrefs_inline_bibliography_is_not_dropped() {
  let x = convert_and_post("tests/cluster_regressions/amsrefs_inline_bibliography.tex");
  // The inline entries became real bibitems (post ran and collected them).
  assert!(
    x.contains("<bibitem"),
    "amsrefs inline bibliography was dropped whole — no bibitem survived:\n{x}"
  );
  // Both entries, with their content, are present. NB amsrefs sentence-cases
  // titles ("On Examples" -> "On examples"), as `amsrefs_basic.xml` records.
  for needle in ["Beilinson", "Height pairing", "On examples", "Smith"] {
    assert!(
      x.contains(needle),
      "amsrefs entry content `{needle}` missing from the References:\n{x}"
    );
  }
  // No leftover uncollected bibentry (they were converted, not deleted).
  assert!(
    !x.contains("<bibentry"),
    "an ltx:bibentry survived unconverted:\n{x}"
  );
  // A `\bib` field value is TeX, not literal text: Perl `BibTeX.pool.ltxml`
  // `\bibentry@create` (L134-166) hands the assembled entry to a fresh Mouth, so
  // `\ndash`/`\MR{…}` tokenize as control sequences. Building a pre-tokenized
  // catcode-12 stream instead leaked them verbatim (`661\ndash693` and the OT1
  // rendering `Review “MR–849427˝`) and left `pages` empty. Witness 2508.17585.
  assert!(
    x.contains("661–693"),
    "`pages={{661\\ndash 693}}` did not render as an en-dashed range — the field \
     value never reached the handlers as live TeX:\n{x}"
  );
  // MakeBibliography must clone bib-review's CHILDREN (Perl `do_links`
  // L655-667 `cloneNodes($node->childNodes)`), not collapse them to text, or
  // the `\MR` MathSciNet link is dropped on the floor.
  assert!(
    x.contains("mathscinet-getitem?mr=849427"),
    "the `review={{\\MR{{849427}}}}` MathSciNet link vanished — bib-review's \
     children were flattened to plain text:\n{x}"
  );
}
/// Loading `bibunits` — even without ever opening a `bibunit` environment —
/// made EVERY citation dangle. `\cite` runs bibunits' `\lx@bibunits@resetglobal`,
/// stamping `CITE_UNIT=bu0`, so the bibref asks for `BIBLABEL:bu0:<key>`; the
/// document's one `\bibliography` registers its bibitems under the default
/// `bibliography` list, and CrossRef searched the unit list ONLY. Witness
/// 2303.06077 (revtex4-2 + bibunits): 93 bibitems rendered, 93 keys dangling,
/// 0 links. Deleting the single `\usepackage{bibunits}` line resolves the cite,
/// which is the whole defect in one bisect.
#[test]
fn bibunits_cite_resolves_against_the_main_bibliography() {
  let x = convert_and_post("tests/cluster_regressions/bibunits_cite.tex");
  // The entry reaches the References either way — the defect is the LINK.
  assert!(
    x.contains("<bibitem"),
    "bibunits: the bibliography itself is missing:\n{x}"
  );
  assert!(
    !x.contains("ltx_missing_citation"),
    "bibunits: \\cite{{Smith2020}} dangles — CrossRef only searched the `bu0` \
     unit list and never fell back to the main `bibliography` list:\n{x}"
  );
}
/// Witness 2605.00490: a JabRef `.bib` self-declaring `% Encoding: Cp1252`.
/// MakeBibliography read it with `read_to_string`, which hard-errors on the
/// first non-UTF-8 byte, so the whole bibliography was dropped and the paper
/// rendered an empty References section with NO `Error:` — a silent, total
/// loss. Real `bibtex` 0.99d is 8-bit clean and Perl passes raw bytes through
/// (`Mouth.pm` L75-80).
///
/// This exercises the POST path (`convert_bib_file_to_xml`), which is where
/// the production failure actually happened; `pre_bibtex`'s own
/// `non_utf8_bib_file_is_read_not_rejected` covers the engine-side reader.
#[test]
fn non_utf8_bib_file_still_yields_a_bibliography() {
  let x = convert_and_post("tests/cluster_regressions/cp1252_bib.tex");
  assert!(
    x.contains("<bibitem"),
    "cp1252 .bib: the whole bibliography was dropped on a non-UTF-8 byte:\n{x}"
  );
  // The Latin-1 fallback is lossless byte -> char, so the accent survives to
  // the rendered entry rather than collapsing to U+FFFD. Only the SURNAME is
  // asserted: the fixture's `author = {Café, André}` is BibTeX's `Last, First`
  // form, so the style abbreviates the given name to `A.` ("A. Café").
  assert!(
    x.contains("Café"),
    "cp1252 .bib: the accented surname did not survive the decode:\n{x}"
  );
}
/// Witness arXiv 2607.00045 (sn-jnl): 44 of its 78 rendered entries carry
/// `note = {\url{...}}`, and every one of them rendered as the dead literal text
/// `\urlhttps://…` instead of a link.
///
/// Two independent flatten-to-text steps had to be fixed, and EITHER of them
/// alone keeps the bug alive:
///
/// 1. `convert_bib_file_to_xml` stringified the digested field
///    (`interpret_tex_text`), and a Whatsit stringifies to its REVERSION — so
///    `\url{…}` came back as its own TeX source, which `strip_braces` then
///    mashed into `\urlhttps://…`. `\href{u}{text}` was worse: the reversion
///    drops the second argument, so the link TEXT was lost outright.
/// 2. `apply_formatter` then took `get_content()` of the field node, discarding
///    any element children. Perl's formatters are `do_any`-shaped and return
///    `$doc->cloneNodes(@nodes)` (`MakeBibliography.pm` L525-531, L550-552), so
///    the markup reaches the bibitem.
///
/// Same-host Perl renders all three of these correctly, so this was
/// GENUINE-RUST-ONLY, not a parity gap.
/// The `.bbl` standard fallbacks: a `.bib` field may use `\url`/`\doi` in a
/// document that loads NOTHING defining them. BibTeX would copy the field into
/// a `.bbl` whose preamble provides them; we digest the raw field one step
/// earlier, so `make_bibliography` supplies the same `\providecommand` block.
///
/// Red before that block existed: `Error:undefined:\url` raised from the field
/// digest (`at Anonymous String`). Witness 2605.01149 — no hyperref, no
/// url.sty, no .bbl, one `howpublished = {\url{...}}`; 1 error -> 0 with the
/// URL still recovered. On the 2026-07-26 sandbox rerun the missing block cost
/// 90 `no_problem -> error` papers in sandbox-arxiv-2605.
///
/// The sibling `bib_field_markup_survives_into_the_bibliography` covers the
/// other side: with hyperref loaded, hyperref's `\url` must still win
/// (`\providecommand` defers), so the link keeps its `href`.
#[test]
fn bib_field_bbl_fallbacks_render_without_a_url_package() {
  let x = convert_and_post("tests/cluster_regressions/bib_field_no_url_package.tex");
  assert!(
    x.contains("<bibitem"),
    "bbl fallbacks: no bibliography at all:\n{x}"
  );
  // The recovered content must be THERE ...
  assert!(
    x.contains("https://example.org/nopkg"),
    "the howpublished URL must survive into the entry:\n{x}"
  );
  assert!(
    x.contains("10.1000/xyz123"),
    "the note's DOI must survive into the entry:\n{x}"
  );
  // `%` is a comment in TeX but literal data in a `.bib`: a percent-encoded URL
  // must survive whole, not be truncated at the first `%` (which also took the
  // closing brace, surfacing as `expected:}`). See `tokenize_bib_field`.
  assert!(
    x.contains("B130936%20Law%20of%20War.pdf"),
    "percent-encoded URL was truncated at the first %:\n{x}"
  );
  // ... and no TeX source may leak, exactly as in the hyperref sibling.
  for leak in ["\\url", "\\doi"] {
    assert!(
      !x.contains(leak),
      "raw {leak} leaked into the rendered bibliography:\n{x}"
    );
  }
}
#[test]
fn bib_field_markup_survives_into_the_bibliography() {
  let x = convert_and_post("tests/cluster_regressions/bib_field_markup.tex");
  assert!(
    x.contains("<bibitem"),
    "bib markup: no bibliography at all:\n{x}"
  );
  // The whole point: no TeX source may leak into the rendered entries.
  for leak in ["\\urlhttps", "\\url{", "\\href", "\\emph"] {
    assert!(
      !x.contains(leak),
      "bib markup: {leak:?} leaked as literal text into the bibliography:\n{x}"
    );
  }
  // `\url` becomes a real link carrying the URL as its own text.
  assert!(
    x.contains("href=\"https://example.org/a\""),
    "bib markup: \\url in a note did not become a link:\n{x}"
  );
  // `\href`'s SECOND argument is the link text — the reversion path lost it.
  assert!(
    x.contains("href=\"https://example.org/b\"") && x.contains("the link text"),
    "bib markup: \\href lost its href or its link text:\n{x}"
  );
  // Markup inside a title survives as markup, not as flattened text.
  assert!(
    x.contains("<emph") && x.contains("emphasis"),
    "bib markup: the emphasized title lost its markup:\n{x}"
  );
  // The fragment is spliced as SERIALIZED XML, so an unescaped `&` in a text
  // node would make the generated bibliography unparseable and drop every
  // entry — the other three entries above are the canary for that.
  assert!(
    x.contains("an ampersand"),
    "bib markup: `\\&` in a marked-up title broke the field:\n{x}"
  );
  // A wholly plain field survives intact — the markup handling cannot silently
  // restructure the 99% case.
  //
  // `title` is asserted SENTENCE-CASED and `publisher` verbatim, which is the
  // engine's `\bib@@title` re-casing (BibTeX.pool.ltxml L281-333) applying to
  // one and not the other. The deleted string route re-cased nothing, so this
  // needle used to read "Wholly Plain Title". Verified against same-host
  // `latexmlc`: all eight rendered titles in this fixture are byte-identical
  // between the engines.
  assert!(
    x.contains("Wholly plain title") && x.contains("Plain Publisher"),
    "bib markup: a plain field was damaged:\n{x}"
  );
  // Block-level content closes the `ltx:text` wrapper and continues as a
  // sibling. Serializing only the wrapper's children dropped everything past
  // that point — silently, with zero errors. The content must survive (the
  // fallback renders it as flattened text, which is the pre-existing
  // behaviour); losing it is the regression being guarded.
  for needle in ["INSIDELIST", "afterblock", "AFTERPAR"] {
    assert!(
      x.contains(needle),
      "bib markup: {needle:?} was silently dropped — block content escaped the \
       wrapper and the fragment was spliced anyway:\n{x}"
    );
  }
  // Fields that reached NO emit branch at all, so their content never appeared
  // in the References. Perl emits every one of them and the format specs
  // already query the matching `ltx:bib-*` elements — only the emitter was
  // missing. `howpublished` is the important one: it is how a @misc carries its
  // URL, and that URL was simply gone.
  for needle in [
    "BIGINSTITUTE",
    "TECHMEMO",
    "LECTURENOTES",
    "SECONDED",
    "BERLINPLACE",
    "SOMEUNIVERSITY",
  ] {
    assert!(
      x.contains(needle),
      "bib fields: {needle:?} never reached the bibliography — its field is \
       parsed but emitted by no branch:\n{x}"
    );
  }
  assert!(
    x.contains("href=\"https://example.org/howpub\""),
    "bib fields: a @misc lost the URL its `howpublished` carries:\n{x}"
  );
}
/// Witness 2605.00184 (`warm-ref.bib`, a Mendeley export): a bare `%` in an
/// `abstract` field comments out the rest of the line INCLUDING the field's
/// closing brace, so the entry's group never closes and every following entry is
/// swallowed. 52 entries, 0 emitted, 102 errors — the whole bibliography, not
/// just the entry that carried the `%`.
///
/// Real BibTeX never lets this happen: a `.bst` declares a closed ENTRY field
/// list and the standard styles omit `abstract`/`keywords`, so bibtex(1) drops
/// them from the `.bbl` (verified against bibtex 0.99d). Same-host Perl is worse
/// than Rust here — 101 errors plus a `too_many_errors` Fatal and no output —
/// so reading these three fields Verbatim is a shared-bug fix. The keys stay
/// supported and still emit `ltx:bib-extract[@role]` exactly as Perl does.
/// OXIDIZED_DESIGN #73.
#[test]
fn bib_abstract_percent_does_not_sink_the_entry() {
  let x = convert_and_post_clean("tests/cluster_regressions/bib_abstract_percent.tex");
  assert!(
    x.contains("<bibitem"),
    "abstract percent: no bibliography at all:\n{x}"
  );
  // Containment is the whole point: the entry AFTER the runaway must survive.
  // Before the fix every one of these was gone together.
  for needle in ["Gilakjani", "Okonkwo", "Lindqvist", "Chen"] {
    assert!(
      x.contains(needle),
      "abstract percent: {needle:?} was swallowed by a runaway field:\n{x}"
    );
  }
  assert!(
    x.contains("The entry after the runaway"),
    "abstract percent: the trailing entry's title was lost:\n{x}"
  );
  // Nothing renders `ltx:bib-extract`, so the prose must not surface as visible
  // bibliography text either — reading it Verbatim keeps it out of harm's way,
  // it does not promote it into the entry.
  assert!(
    !x.contains("ablation study"),
    "abstract percent: abstract prose leaked into the rendered entry:\n{x}"
  );
}
/// `\&amp;` in a `.bib` field is a doubly escaped ampersand, and renders as one.
///
/// A reference manager rendered the field to HTML (`&` -> `&amp;`) and a second
/// pass TeX-escaped that entity's own ampersand, so the file carries four
/// characters the author never wrote. TeX has no way to know: `\&` is the glyph,
/// `amp;` is ordinary text, and the entry reads "Computer Engineering, &amp;
/// Applied Computing". pdflatex prints exactly the same, so this is neither a
/// Perl gap nor a pdflatex gap — it is corrupt input, and reading `.bib`
/// directly is what puts us in a position to undo it. OXIDIZED_DESIGN #74.
///
/// All three spellings are real; found by scanning 6000 arXiv/2605 sources.
/// `titleentity` is the one that needs its own decode: `\bib@@title` re-reads
/// the raw field for case conversion instead of using the argument it was
/// handed, so it bypasses the decode done while assembling the entry.
///
/// RED before the fix: `&amp;amp;` for the first three entries.
#[test]
fn bib_escaped_amp_entity_decodes_to_one_ampersand() {
  let x = convert_and_post_clean("tests/cluster_regressions/bib_escaped_amp_entity.tex");
  // The XML escape of a literal `&` is `&amp;`, so the doubled entity shows up
  // as `&amp;amp;`. Assert on that serialized form directly — unescaping first
  // would make the two cases indistinguishable.
  assert!(
    !x.contains("&amp;amp;"),
    "escaped amp entity: a doubled `&amp;` survived into the bibliography:\n{x}"
  );
  // Not merely absent — decoded. Each entry keeps ONE ampersand, with the text
  // on both sides intact (a decode that ate the neighbouring word would also
  // satisfy the assertion above).
  for needle in [
    "Computer Engineering, &amp; Applied Computing",
    "the A&amp;AS all-sky survey",
    "shake &amp; bake generation",
    // Control: an ordinary `\&` was already right and must stay right.
    "Crime &amp; Delinquency",
  ] {
    assert!(
      x.contains(needle),
      "escaped amp entity: expected {needle:?} in the bibliography:\n{x}"
    );
  }
  // Near miss: "amp;" that is NOT preceded by an ampersand is ordinary text.
  assert!(
    x.contains("amp; token and amplitude modulation"),
    "escaped amp entity: a literal `amp;` unrelated to an ampersand was eaten:\n{x}"
  );
}
/// A bare `&` in a `.bib` field is the literal character, not an alignment tab.
///
/// `publisher = {Taylor & Francis}` is real: seven arXiv/2605 papers ship it
/// (per-witness provenance in the fixture header). BibTeX's lexer has no
/// alignment, so the `&` it hands back is a character in a publisher's or a
/// journal's name — but TeX reads catcode 4, raises `Error:unexpected:&`, and
/// DROPS the character, so the entry printed "Taylor Francis".
///
/// Deliberately ahead of every other engine, which is why the before-numbers
/// matter: same-host `latexmlc` raised the same one-error-per-`&` (2605.03054
/// 1/1, 2605.06249 3/3, 2605.00462 1/1, 2605.06624 1/1, 2605.08753 1/1,
/// 2605.10409 1/1), and bibtex 0.99d + pdflatex agree — under `plain` and
/// `abbrvnat` the bare `&` reaches the `.bbl`, pdflatex stops with "Misplaced
/// alignment tab character &" and prints "Taylor Francis". Reading `.bib`
/// directly is what lets us decide otherwise. OXIDIZED_DESIGN #74.
///
/// Neutralized at the per-entry Mouth beside the `%` of #74 — regime A, because
/// `&` derails alignment in ANY field and these fields have no Perl
/// Semiverbatim precedent to follow.
///
/// RED before the fix: 5 post-stage errors, and "Taylor Francis".
#[test]
fn bib_bare_ampersand_is_literal_data() {
  let x = convert_and_post_clean("tests/cluster_regressions/bib_bare_ampersand.tex");
  // The ampersand is KEPT, with the text on both sides of it — a neutralization
  // that swallowed the rest of the field would still be error-free.
  for needle in [
    "Taylor &amp; Francis",
    "Information Processing &amp; Management",
    "Knowledge Discovery &amp; Data Mining",
  ] {
    assert!(
      x.contains(needle),
      "bare ampersand: expected the literal {needle:?} in the bibliography:\n{x}"
    );
  }
  // Containment: every entry still renders, including the one after the run of
  // bad fields. The sibling `%` bug (#73) took the whole bibliography down.
  for needle in ["Author", "Builder", "Coder", "Crespi", "Draper", "Ericsson"] {
    assert!(
      x.contains(needle),
      "bare ampersand: {needle:?} was lost from the bibliography:\n{x}"
    );
  }
  assert!(
    x.contains("The entry after the stray ampersands"),
    "bare ampersand: the trailing entry's title was lost:\n{x}"
  );
}
/// Making `&` ordinary must not flatten the live TeX beside it.
///
/// The boundary the mouth-level neutralization has to hold: it downgrades ONE
/// catcode, so everything else in the same field keeps working. `\emph` still
/// marks up, `$x_1+x_2$` still parses as math with `_` a subscript INSIDE math
/// (the neutralization is not a blanket verbatim), and the space-form accents
/// PR #399 recovered still resolve — all in the entry that also carries a bare
/// `&`, so a regression cannot hide behind a separate clean fixture.
#[test]
fn bib_bare_ampersand_leaves_live_markup_alone() {
  let x = convert_and_post_clean("tests/cluster_regressions/bib_bare_ampersand.tex");
  // Lower-cased by `\bib@@title`'s `capitalize1` recasing, exactly as Perl
  // recases it — the point here is the `&`, which survives.
  assert!(
    x.contains("ampere &amp; ohm"),
    "markup boundary: the ampersand in the boundary entry was lost:\n{x}"
  );
  assert!(
    x.contains(">emphasized</emph>"),
    "markup boundary: \\emph stopped producing markup:\n{x}"
  );
  // `_` keeps its subscript meaning INSIDE math — the neutralization downgrades
  // one catcode in the `.bib` text, it is not a blanket verbatim. The parsed
  // form is a subscript application, not the literal characters.
  assert!(
    x.contains(r#"role="SUBSCRIPTOP""#),
    "markup boundary: inline math stopped parsing (`_` must still subscript \
     inside math):\n{x}"
  );
  for needle in ["\u{160}pakov", "Gon\u{e7}alves"] {
    assert!(
      x.contains(needle),
      "markup boundary: space-form accent {needle:?} regressed (PR #399):\n{x}"
    );
  }
}
/// A `.bib` field's content is DATA, not TeX: a bare `_`, `&`, `#` or `%` is
/// the literal character. `AT&T` renders "AT&T"; `AT1G01010_v2` renders
/// "AT1G01010_v2". Witnesses: 2605.06926 (8 `unexpected:_`, four `eprint` PDF
/// URLs), 2605.01936, 2605.04604, 2605.08986, 2605.11300, and the `&` half in
/// 2605.01936 / 2605.06249 (`publisher = {Taylor & Francis}`).
///
/// Authorized surpass-Perl AND surpass-pdflatex (OXIDIZED_DESIGN #74): real
/// BibTeX has a DATA regime and a TeX regime, and we collapse them because we
/// read `.bib` directly with no `.bst` in the loop. That `bibtex(1)` +
/// `pdflatex` also break on these characters is a property of that toolchain,
/// not a semantic we are obliged to reproduce.
///
/// ONE fixture for all of it, because the whole risk is that fixing one case
/// breaks another. This asserts, together: the four specials bare render
/// literally; already-escaped `\&`/`\%`/`\_`/`\#` render IDENTICALLY and are not
/// double-escaped; `$x_1+x_2$` still parses as math with a real `<msub>`;
/// `\emph` still produces markup; a space-form accent still reverts (PR #399);
/// and `%20` inside `\url{…}` is untouched, since url.sty reads that argument
/// verbatim. The `\\&` hazard is pinned separately, by the
/// `escape_bib_data_specials` unit tests in `latexml_engine/src/bibtex.rs` —
/// see the fixture header for why it cannot live end-to-end.
#[test]
fn bib_field_specials_are_data_not_tex() {
  let x = convert_and_post_clean("tests/cluster_regressions/bib_field_specials.tex");
  assert!(
    x.contains("<bibitem"),
    "bib specials: no bibliography at all:\n{x}"
  );
  // All ten entries arrive — `Gilakjani` is the containment canary.
  for needle in [
    "Okonkwo",
    "Lindqvist",
    "Chen",
    "Park",
    "Diallo",
    "Bergstr",
    "Raghavan",
    "Osman",
    "Gilakjani",
  ] {
    assert!(
      x.contains(needle),
      "bib specials: {needle:?} was swallowed:\n{x}"
    );
  }
  // The rendered `<text class="ltx_bib_title">` of the entry with a given key.
  let title_of = |key: &str| -> String {
    let i = x
      .find(&format!("key=\"{key}\""))
      .unwrap_or_else(|| panic!("bib specials: no entry keyed {key:?} in:\n{x}"));
    let block = &x[i..];
    let s = block
      .find("class=\"ltx_bib_title\">")
      .expect("a rendered bib-title")
      + "class=\"ltx_bib_title\">".len();
    let e = block[s..].find("</text>").expect("bib-title close") + s;
    block[s..e].to_string()
  };
  // 1. The four specials, bare, are literal text. `&` is XML-escaped on the way
  //    out, which is the correct rendering of the character. `recase_title`
  //    lowercases (capitalize1, Perl parity), hence `at1g01010`.
  let bare = "AT&amp;T dataset at1g01010_v2 at 95% with #3 replicates";
  assert_eq!(
    title_of("barespecials"),
    bare,
    "bib specials: bare `& _ % #` did not render literally:\n{x}"
  );
  // 2. Idempotency: the entry that already wrote `\&`/`\_`/`\%`/`\#` renders
  //    the SAME string. Had the escaper double-escaped, `\&` would have become
  //    `\\&` — a line break followed by an ampersand.
  assert_eq!(
    title_of("preescaped"),
    title_of("barespecials"),
    "bib specials: the pre-escaped title must render identically to the bare \
     one:\n{x}"
  );
  assert!(
    !x.contains(r"\&") && !x.contains(r"\_") && !x.contains(r"\%"),
    "bib specials: an escaping backslash leaked into the output:\n{x}"
  );
  // 3. Math survives WITH its subscript, and 4. markup survives.
  //    `convert_and_post_clean` stops at the post-processed `ltx` XML with
  //    `pmml: false`, so the subscript shows as XMath's SUBSCRIPTOP rather than
  //    an `<m:msub>`.
  assert!(
    x.contains(r#"role="SUBSCRIPTOP""#),
    "bib specials: `$x_1+x_2$` in a title lost its subscript:\n{x}"
  );
  assert!(
    x.contains("<emph font=\"italic\">Drosophila</emph>"),
    "bib specials: `\\emph` in a title stopped producing markup:\n{x}"
  );
  // 5. Space-form accents still revert (PR #399).
  assert!(
    x.contains("Špakov") && x.contains("Gonçalves"),
    "bib specials: a space-form accent stopped reverting:\n{x}"
  );
  // 6. A percent-encoded URL inside `\url{…}` is a nested DATA region.
  assert!(
    x.contains("http://example.org/a%20b"),
    "bib specials: `%20` inside `\\url{{…}}` was escaped into the href:\n{x}"
  );
  // And a Verbatim/Semiverbatim-read FIELD is passed through raw, so neither
  // the `url` href nor the `doi` id picks up a backslash.
  assert!(
    x.contains("https://example.org/a%20b?x=1&amp;y=2_3"),
    "bib specials: the `url` field was escaped:\n{x}"
  );
  assert!(
    x.contains("10.1007/3-540-44886-1%5F25"),
    "bib specials: the `doi` id lost its percent-encoding:\n{x}"
  );
  // 7. The original cluster: both spellings of the AIP `eprint` URL land as a
  //    plain `_`.
  for needle in ["18931574/1876_1_online.pdf", "15540793/184110_1_online.pdf"] {
    assert!(
      x.contains(needle),
      "bib specials: {needle:?} did not reach the rendered links:\n{x}"
    );
  }
  // 8. The `&` half of the cluster.
  assert!(
    x.contains("Taylor &amp; Francis"),
    "bib specials: a bare `&` in `publisher` did not render:\n{x}"
  );
  assert!(
    x.contains("IEEE Communications Surveys &amp; Tutorials"),
    "bib specials: a bare `&` in `journal` did not render:\n{x}"
  );
}
/// A space-form accent in an author name must survive reversion.
///
/// `{\v S}pakov` / `Gon{\c c}alves` / `\" Ozturk` are ordinary BibTeX — the
/// accent command separated from its argument by a space rather than braced.
/// The space is what TERMINATES the control word, and the tokenizer consumes
/// it, so by token-time it is gone as data. Reverting tokens back to a string
/// therefore has to re-emit it: `UnTeX` does, plain concatenation does not.
///
/// `\bib@@names` reverted with `to_string()` where Perl uses
/// `UnTeX($names, 1)` (`BibTeX.pool.ltxml` L277) — and `Display for Tokens`
/// says outright that it is "NOT for creating valid TeX (use revert or UnTeX
/// for that!)". So `\v` and `S` were welded into `\vS`, a control sequence that
/// exists in no LaTeX: `Error:undefined:\vS`, rendered as literal `\vSpakov`.
///
/// RED before the fix, on exactly this content: 2 errors
/// (`undefined:\vS`, `undefined:\cc`) and `O. \vSpakov and A. Gon\ccalves`.
/// Same-host `latexmlc`: 0 errors, `O. Špakov and A. Gonçalves` —
/// GENUINE-RUST-ONLY. Found by the 2026-07-26 sweep of sandbox-arxiv-2605/2606,
/// where it cost ~+2800 error documents per corpus; the guard suite was green
/// throughout, because every bibliography fixture was ASCII.
#[test]
fn bib_name_space_form_accent_survives_reversion() {
  let x = convert_and_post("tests/cluster_regressions/bib_accent_space_form.tex");
  assert!(
    x.contains("<bibitem"),
    "accent fixture: no bibliography at all:\n{x}"
  );
  // The accents must have COMPOSED, not merely survived as source.
  for (name, ch) in [
    ("Špakov", 'Š'),    // {\v S} — space form, braced group
    ("Gonçalves", 'ç'), // {\c c} — space form, braced group
    ("Özturk", 'Ö'),    // \" O   — space form, no braces at all
    ("Švec", 'Š'),      // \v{S}  — braced argument, never broken
    ("Grégoire", 'é'),  // {\'e}  — braced argument, never broken
  ] {
    assert!(
      x.contains(name),
      "accent fixture: {name:?} (composing {ch:?}) is missing — the accent \
       command did not apply:\n{x}"
    );
  }
  // And no welded control sequence may leak. These are the exact shapes the
  // bug produced; each is a CS that exists in no LaTeX.
  for welded in ["\\vS", "\\cc", "\\\"O", "\\vSpakov", "\\ccalves"] {
    assert!(
      !x.contains(welded),
      "accent fixture: {welded:?} leaked — a control word was welded to its \
       argument, so reversion dropped the terminating space:\n{x}"
    );
  }
}
/// A blank line or a `\\` inside a `.bib` FIELD VALUE must not start a new
/// bibliography item.
///
/// `{bibtex@bibliography}` inherited Perl's `setupPseudoBibitem`
/// (`latex_constructs.pool.ltxml` L4028-4047), which `\let`s BOTH `\par` and
/// `\\` to `\par@in@bibliography` — a heuristic that emits a fresh
/// `\save@bibitem{}` whenever it fires. It rescues HAND-WRITTEN
/// `thebibliography` lists whose author used blank lines instead of
/// `\bibitem`; a `.bib`-derived bibliography is machine-generated
/// (`Pre::BibTeX::toTeX` writes one `\ProcessBibTeXEntry{key}` per line) and
/// has no missing `\bibitem` to rescue, so it can only misfire. It did:
/// `<ltx:bibitem>` was opened inside `<ltx:surname>` / `<ltx:bib-title>` /
/// `<ltx:bib-note>`, which the model rejects, and it never closed — every
/// later entry then nested inside the dangling element.
///
/// Note the diagnostic trap: the element in the error message is NOT unclosed
/// at fault. `<ltx:surname>` is opened and closed exactly as its constructor
/// says; it is merely the insertion context when the spurious item is opened.
///
/// Same-host Perl 0.8.8 over this very `.bib` emits the same 6
/// `malformed:ltx:bibitem` errors (it names `<ltx:givenname>` where Rust names
/// `<ltx:surname>` — a name-split nuance, same cluster), so pdflatex is the
/// ground truth: bibtex 0.99d compresses white-space runs in a field value to a
/// single space (the blank line never reaches TeX) and copies `\\` through,
/// where `thebibliography` renders it as a line break inside the item.
/// OXIDIZED_DESIGN #75.
///
/// RED before the fix on exactly this fixture: 6 post-stage
/// `malformed:ltx:bibitem` errors — 2 in `<ltx:surname>`, 2 in
/// `<ltx:bib-note>`, 1 in `<ltx:bib-title>`, 1 cascading into the injected
/// item's own `<ltx:bibblock>`. Witnesses 2605.03313 7 -> 0,
/// 2605.03693 7 -> 1 (residual is an unrelated text-mode `^`),
/// 2605.11080 1 -> 0; final rendered HTML byte-identical on all three.
#[test]
fn bib_field_blank_line_does_not_inject_a_bibitem() {
  let x = convert_and_post_clean("tests/cluster_regressions/bib_field_blank_line.tex");
  assert!(
    x.contains("<bibitem"),
    "blank line: no bibliography at all:\n{x}"
  );
  // Every entry must arrive, including the canary AFTER the injected item.
  for needle in ["Tao", "Santos", "Girolami", "Fixture"] {
    assert!(
      x.contains(needle),
      "blank line: {needle:?} was lost to an injected bibliography item:\n{x}"
    );
  }
  assert!(
    x.contains("The entry after the injected item"),
    "blank line: the containment canary's title was lost:\n{x}"
  );
  // The blank line inside `title` is whitespace, so the title must read as ONE
  // phrase; before the fix everything past it moved into an injected item.
  assert!(
    x.contains("top-k range reporting"),
    "blank line: the title was split at the blank line:\n{x}"
  );
  // The blank lines inside `author` must not damage the name split either:
  // `MacDonald` is the SECOND name, the one that followed a blank line.
  assert!(
    x.contains("MacDonald"),
    "blank line: a name following a blank line was lost:\n{x}"
  );
  // Both `\\`-separated URLs of the 2605.11080 note must survive in one entry.
  for url in ["pages.jh.edu/eberti2/ringdown", "grit/files/ringdown"] {
    assert!(
      x.contains(url),
      "blank line: {url:?} was lost — `\\\\` started a new item:\n{x}"
    );
  }
  // Resilience half: an EMPTY name part is the other way an `<ltx:surname>`
  // could be left open, since `insert_element(tag, [], attrs)` opens and leaves
  // open on empty content (Perl `insertElement($tag, undef, ...)`). It cannot
  // happen through this path — `\bib@@names` skips an empty part outright — and
  // these five entries pin that. Each one's title must arrive, which it cannot
  // if a preceding entry left an element open.
  for title in [
    "Wholly empty author",
    "Surname then nothing",
    "Nothing then given",
    "A trailing and",
    "Two ands in a row",
  ] {
    assert!(
      x.contains(title),
      "blank line: {title:?} lost — an empty name part left an element open:\n{x}"
    );
  }
  // The parts that DO exist are still split correctly around the empty ones.
  // `, John` is deliberately absent from this list: a givenname-only author
  // renders as nothing at all, which is a MakeBibliography formatting nuance
  // shared with Perl (bibtex's `{f.}` would print "J."), not this cluster. Its
  // ENTRY still arrives — asserted by title above — which is what containment
  // means here.
  for name in ["Smith", "Brown", "Green"] {
    assert!(
      x.contains(name),
      "blank line: {name:?} lost — an empty sibling part ate a real one:\n{x}"
    );
  }
}

/// A `%` inside a `.bib` field value is data, not a comment.
///
/// BibTeX's lexer has no comment syntax inside an entry — `Pre::BibTeX` only
/// skips `%` in the junk BETWEEN entries — so the value it stores keeps its
/// `%`. LaTeXML then re-injects that value as TeX source, where catcode 14
/// makes it comment out the rest of its line, closing brace included: the
/// entry's group never closes and every later entry nests inside the unclosed
/// element (`<ltx:bibentry> isn't allowed in <ltx:bibentry>`).
///
/// Two paths, one per entry in the fixture: `doi` goes through the per-entry
/// Mouth `\ProcessBibTeXEntry` opens, `title` does NOT — `\bib@@title` re-reads
/// the raw field and tokenizes it itself. Fixing only the mouth left the
/// `title` half broken (it went 28 → 37 errors on the witness, surfacing an
/// `Extra \endcsname` once the value was no longer truncated), so both call
/// sites read `%` as OTHER. OXIDIZED_DESIGN #74.
///
/// RED before the fix: witnesses 2605.01196 (`doi={%doi:10.1017/jfm.2016.420}`)
/// and 2605.02131 (percent-encoded URL in a `title`'s `\href`) at **28 errors
/// each**; same-host `latexmlc` 29 / 31 on the same inputs, so this is a shared
/// bug and a surpass-Perl fix, not a Rust-only defect. Both are 0 after.
/// `convert_and_post_clean`, not `convert_and_post`: the bibliography is built
/// in POST, and a text-presence assertion still finds text INSIDE an unclosed
/// element — the plain helper passes on a structurally broken document.
#[test]
fn bib_field_percent_is_an_ordinary_character() {
  let x = convert_and_post_clean("tests/cluster_regressions/bib_field_percent.tex");
  // Containment: four entries, four bibitems. Before the fix the runaway
  // swallowed its followers into its own unclosed <ltx:bibentry>.
  assert_eq!(
    x.matches("<bibitem").count(),
    4,
    "percent field: expected four bibitems, one per entry:\n{x}"
  );
  assert!(
    x.contains("The entry after the runaway"),
    "percent field: the canary entry after the runaway is missing:\n{x}"
  );
  // The `%` is DATA: it must survive into the output, not be dropped along
  // with everything after it on its line.
  // (the doi becomes an href, so its own `%`/`:` are URL-escaped there —
  // `%25doi%3A` IS the surviving `%doi:`.)
  assert!(
    x.contains("dx.doi.org/%25doi%3A10.1017/jfm.2016.420"),
    "percent field: the doi value was truncated at its percent sign:\n{x}"
  );
  assert!(
    x.contains("https://cdn.example.org/20240723%20IPWG%20Item%2004b%20DRAFT.pdf"),
    "percent field: the percent-encoded URL lost its escapes:\n{x}"
  );
  assert!(
    x.contains("A linked title behind a percent-encoded URL"),
    "percent field: the linked title did not render:\n{x}"
  );
  // Neutralizing `%` must not flatten the field: markup, math and accents in
  // the SAME field still have to work (the boundary PR #399 established).
  assert!(
    x.contains("Yield rose <emph") && x.contains(">sharply</emph>"),
    "percent field: \\emph in a percent-bearing title lost its markup:\n{x}"
  );
  assert!(
    x.contains("tex=\"x_{1}+x_{2}\"") && x.contains("<XMApp>"),
    "percent field: inline math in a percent-bearing title did not parse:\n{x}"
  );
  assert!(
    x.contains("Špakov"),
    "percent field: the space-form accent in the same entry stopped \
     composing:\n{x}"
  );
  // No entry may nest inside another, and no element may be left open.
  assert!(
    !x.contains("<bibentry"),
    "percent field: a raw <bibentry> survived post-processing — \
     MakeBibliography could not read the entry:\n{x}"
  );
}

/// A space-form accent in a `MRREVIEWER` / `ZBLREVIEWER` field must survive the
/// Tokens->string round trip.
///
/// `current_entry_field` returns `Tokens`; `to_string()` drops the space that
/// terminates a control word, so `Fran\c cois` welded to `\ccois` — an undefined
/// macro where a reviewer's name belongs. Perl cannot hit this: its
/// `currentBibEntryField` returns a plain string (`Pre/BibTeX/Entry.pm` L38), so
/// `Tokenize($mrreviewer)` has no round trip. GENUINE-RUST-ONLY; the fix is
/// `untex()`, the same one PR #399 applied to `\bib@@names`.
///
/// Witness 2605.11579: 5 errors -> 1, and the survivor (`undefined:\Dbar`, a
/// MathSciNet glyph macro LaTeXML genuinely lacks) is now an honest diagnostic
/// rather than a fabricated one.
#[test]
fn bib_mr_reviewer_accent_survives_reversion() {
  let x = convert_and_post_clean("tests/cluster_regressions/bib_mr_reviewer_accent.tex");
  // The welds this guards against, one per accent shape in the witness.
  for weld in ["ccois", "cprimefand", "\\ic", "\\io"] {
    assert!(
      !x.contains(weld),
      "MR reviewer: accent welded to the following text ({weld:?}):\n{x}"
    );
  }
  // ... and the names must actually be there, in both the MR and Zbl paths.
  assert!(
    x.matches("Digne").count() >= 2,
    "MR reviewer: the cedilla name is missing from one of the MR/Zbl paths:\n{x}"
  );
  for needle in ["fand", "lu"] {
    assert!(
      x.contains(needle),
      "MR reviewer: {needle:?} lost from a reviewer name:\n{x}"
    );
  }
}
