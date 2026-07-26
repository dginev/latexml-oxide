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
