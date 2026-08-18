//! Cluster regressions in bibliography handling — `.bib`/`.bbl` reading, the
//! recursive BibTeX session, citation styles and rendered `References`.
//!
//! Split out of `06_cluster_regressions`; shares its helpers via
//! [`mod cluster`](cluster).

mod cluster;
use cluster::{
  convert_and_post, convert_and_post_clean, convert_and_post_contrib_clean,
  convert_and_post_logging, convert_to_xml, convert_to_xml_ar5iv, convert_to_xml_contrib,
  convert_to_xml_contrib_clean,
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

/// `\bibliography{}` — an EMPTY argument — still inputs `\jobname.bbl`, because
/// that is what `latex.ltx` does: the argument feeds the `.aux` `\bibdata`
/// record, not the decision to read the `.bbl`. Measured under pdflatex on
/// `docs/parity/bib_absence_2026-07-29/repros/f3_empty_arg_bbl/`: "References /
/// [1] A. Uthor. A paper. 2020.", with `\cite` resolving to `[1]`.
///
/// Perl stops at `return unless $bib_files` (`latex_constructs.pool.ltxml`
/// L3901) and says nothing, so the references are dropped in silence — this
/// port did the same until OXIDIZED_DESIGN #86. 7 papers in the 2605+2606
/// sandboxes share one template that writes `\bibliography{}` and ships the
/// `.bbl` (the GWTC-5 LIGO set); witness 2605.27226.
///
/// The second half is the load-bearing one: with no `\jobname.bbl` on disk
/// there is nothing to input, so neither clause may fire — an empty argument
/// must not conjure a placeholder bibliography.
#[test]
fn bib_empty_argument_still_reads_the_jobname_bbl() {
  let x = convert_to_xml("tests/cluster_regressions/bblbib/emptyarg.tex");
  assert!(
    x.contains("BBLCHOSEN") && !x.contains("BIBCHOSEN"),
    "empty \\bibliography{{}} beside a <jobname>.bbl should read the bbl, got:\n{x}"
  );
  let x = convert_to_xml("tests/cluster_regressions/bblbib/emptyargnobbl.tex");
  assert!(
    !x.contains("BBLCHOSEN") && !x.contains("BIBCHOSEN"),
    "empty \\bibliography{{}} without a .bbl should choose neither, got:\n{x}"
  );
}
/// `\nocite{*}` includes the whole library, as bibtex does.
///
/// Queueing per BIBLABEL record cannot express that: in a document whose only
/// citation is `\nocite{*}` the sole record IS `*`, Step 3 skips that key by
/// name, and the outcome was "N bibentries, **0 cited**" over an empty
/// References list — while `pdflatex`+`bibtex` on the same source prints the
/// full list (measured 2 against our 0 on a 2-entry `.bib`).
///
/// Perl skips the star key the same way (`MakeBibliography.pm` L279-313), so
/// this is deliberately beyond Perl. 7 papers of the 2605+2606 residual are
/// `\nocite{*}`-only, all recovered: 2605.24970 0 -> 46, 2606.05528 0 -> 42,
/// 2605.28000 0 -> 39.
#[test]
fn bib_nocite_star_includes_the_whole_library() {
  let x = convert_and_post("tests/cluster_regressions/bib_nocite_star.tex");
  let n = x.matches("<bibitem").count();
  assert_eq!(
    n, 2,
    "\\nocite{{*}} did not include the whole library: {n} entries\n{x}"
  );
  assert!(
    x.contains("First starred entry") && x.contains("Second starred entry"),
    "an uncited-but-starred entry is missing:\n{x}"
  );
}

/// `\DeclareCiteCommand` must actually define the command it declares.
///
/// It is biblatex's own API for a custom citation command and papers use it
/// heavily. As a pure no-op it consumed its arguments and defined NOTHING, so
/// every call raised `Error:undefined:\foo`, no citation record was made, and
/// `MakeBibliography` reported "N bibentries, **0 cited**" over an empty
/// References list — the bibliography was read and then discarded for want of a
/// `\cite`. Witness 2605.02115 (`\DeclareCiteCommand{\citeq}` at main.tex
/// L321, used 83 times): 14 entries read, 0 rendered; now 11 cited and
/// rendered, and pdflatex renders the list too.
///
/// The declared command is aliased to biblatex's `\cite`, keeping the citation
/// semantics and dropping only the author's bespoke label format.
#[test]
fn biblatex_declarecitecommand_defines_its_command() {
  let x = convert_to_xml_contrib("tests/cluster_regressions/biblatex_ay/declarecite.tex");
  assert!(
    !x.contains("ERROR"),
    "the declared cite command stayed undefined:\n{x}"
  );
  assert!(
    x.contains("<bibitem") && x.contains("Smith"),
    "citations made with the declared command did not select any entry:\n{x}"
  );
}

/// docmute must strip an included stand-alone document's preamble and turn its
/// `\end{document}` into `\endinput`.
///
/// docmute exists so a paper can `\input` files that are each a complete
/// document. The real package hooks `\let\documentclass=…` and
/// `\renewenvironment{document}`; neither reaches us, because `\begin{document}`
/// and `\end{document}` are their own control sequences here rather than the
/// `document` environment docmute patches — so raw-loading docmute.sty is inert.
/// The included preamble then leaks (`\usepackage … can only appear in the
/// preamble`, once per line) and the included `\end{document}` terminates the
/// OUTER document, discarding everything after the `\input` with the
/// bibliography in it.
///
/// Witness 2606.09184: 0 -> 18 entries, and pdflatex renders 17 under
/// "References" from a clean extract. 6 papers in the residual load docmute.
/// Perl has no docmute binding either, so this is surpass-Perl with the arXiv
/// PDF as ground truth. Red/green is the bibliography length; 0 is red.
#[test]
fn bib_docmute_input_keeps_the_outer_document() {
  let x = convert_to_xml("tests/cluster_regressions/docmute/outer.tex");
  let n = x.matches("<bibitem").count();
  assert!(
    n >= 2,
    "docmute: the included \\end{{document}} ended the outer document: {n} entries\n{x}"
  );
  assert!(
    x.contains("AFTERTEXT reached"),
    "content after the \\input was discarded:\n{x}"
  );
  // The included body must still be spliced in — stripping the preamble must
  // not throw away the file's content.
  assert!(
    x.contains("INNERBODY"),
    "the included document's body was lost:\n{x}"
  );
}

/// pnas-new must declare the booleans its own style files switch.
///
/// The real class does `\RequirePackage{xifthen}` (pnas-new.cls L97) and then
/// `\newboolean{shortarticle}` and friends (L98-118). Our binding replaces the
/// class, so it has to do both: without them the FIRST line of
/// `pnasresearcharticle.sty` — `\setboolean{shortarticle}{true}` — raises
/// `Error:undefined:\setboolean` and is defined as `<ltx:ERROR/>`, the
/// article-type style unravels, and the document loses its tail with the
/// bibliography in it. The class ships with the paper and is absent from TeX
/// Live, so nothing else declares them.
///
/// 6 papers in the 2605+2606 cohort, every one `\documentclass{pnas-new}` and
/// every one 0 entries before: 2606.02411 0 -> 45, 2606.29674 0 -> 60,
/// 2605.07504 0 -> 53. Red/green is the bibliography length; 0 is red.
#[test]
fn bib_pnas_setboolean_keeps_the_document_tail() {
  let x = convert_to_xml_contrib("tests/cluster_regressions/bib_pnas_setboolean.tex");
  let n = x.matches("<bibitem").count();
  assert!(
    n >= 2,
    "pnas \\setboolean failure swallowed the bibliography: {n} entries\n{x}"
  );
}

/// achemso's `{tocentry}` must not swallow the rest of the document.
///
/// It used to be suppressed with `\begin{tocentry}` → `\iffalse` and
/// `\end{tocentry}` → `\fi`. Conditional skipping matches `\fi` by MEANING and
/// expands nothing on the way (TeX's rule, and `skip_conditional_body`'s), so
/// an `\end{tocentry}` whose macro *body* is `\fi` is invisible to it: the skip
/// ran to end of file and took the rest of the paper with it, `\bibliography`
/// included. It surfaced as `Error:expected:\fi \iffalse` pointing at EOF,
/// which reads like a source defect and is not one — the `\iffalse` was ours.
///
/// 42 of the 342 residual papers in the 2605+2606 bibliography-absence cohort
/// lost their whole bibliography to this one line. Witness 2606.14933
/// (0 -> 69 entries). Perl never reaches it: no achemso binding, OmniBus
/// fallback, `{tocentry}` simply undefined.
///
/// The body is skipped as RAW LINES (comment.sty's idiom), not digested:
/// digesting it and dropping the result turned three papers from "no
/// bibliography" into no OUTPUT at all — 2606.08929, 2606.12056, 2606.15422
/// went from 1 error to 513 and a fatal, because these graphics are TikZ /
/// `\includegraphics` blocks that error out of context.
///
/// Red/green is the bibliography length — 0 entries is red. The third
/// assertion keeps the suppression honest: the graphic's text must stay out of
/// the body.
#[test]
fn bib_achemso_tocentry_does_not_swallow_the_bibliography() {
  let x = convert_to_xml_contrib("tests/cluster_regressions/bib_achemso_tocentry.tex");
  let n = x.matches("<bibitem").count();
  assert!(
    n >= 2,
    "{{tocentry}} swallowed the bibliography: {n} entries\n{x}"
  );
  assert!(
    x.contains("Body cites"),
    "the document body after {{tocentry}} was swallowed:\n{x}"
  );
  assert!(
    !x.contains("graphic caption"),
    "the tocentry graphic leaked into the body:\n{x}"
  );
}

/// `\captionof{lstlisting}` must not swallow the rest of the document.
///
/// Perl's binding wraps the caption in the named environment — "it isn't
/// necessarily IN a figure or any float, so we'll wrap it in an otherwise empty
/// one!" (`caption.sty.ltxml` L124-125) — which is fatal when that environment
/// reads its body verbatim: `\begin{lstlisting}` makes listings scan the raw
/// INPUT for `\end{lstlisting}`, never the token stream, so it finds no
/// terminator and consumes the rest of the file. The tail — `\bibliography`
/// included — comes out as line-numbered listing text.
///
/// Real caption.sty never opens the environment: `\caption@of` is
/// `\setcaptiontype*{#2}#1` (caption.sty L391), i.e. it only sets the type, and
/// pdflatex renders such papers correctly.
///
/// The red/green signal is the bibliography length — 0 entries is red. Witness
/// 2606.08339: one such line cost all 30 of its entries (verified 0 -> 30).
/// OXIDIZED_DESIGN #89.
#[test]
fn bib_captionof_verbatim_env_does_not_swallow_the_bibliography() {
  let x = convert_to_xml("tests/cluster_regressions/bib_captionof_listing.tex");
  let n = x.matches("<bibitem").count();
  assert!(
    n >= 2,
    "\\captionof{{lstlisting}} swallowed the bibliography: {n} entries\n{x}"
  );
  assert!(
    x.contains("Tail text after the figure"),
    "the document tail after the figure was swallowed:\n{x}"
  );
}

/// A raw `\def\cite` must not displace LaTeXML's binding.
///
/// arXiv submissions ship their conference style, and under `--includestyles`
/// it is raw-loaded — aaai/iccc/flairs/kr/achicago/fixbib all `\def\cite`.
/// The replacement records no citation, so MakeBibliography selects nothing
/// and the References section renders empty while the body shows bold `?`
/// markers: "N bibentries, **0 cited**". The core `\cite` is now `locked`,
/// exactly as natbib locks its own variants (`natbib.sty.ltxml` L151), so raw
/// `.sty`/`.cls` and document redefinitions are ignored while native bindings
/// — which load UNLOCKED (`content.rs`, Perl `Package.pm:loadLTXML` L2318) —
/// still override freely.
///
/// Deliberately beyond Perl, and beyond the PDF: these submissions ship no
/// `.bbl` and arXiv does not run bibtex, so the official PDF has no reference
/// list either. Keeping the semantic `ltx:cite` is what makes the HTML usable.
/// 13 of 15 witnesses recovered, all 0 before: 2605.07102 (0 -> 50),
/// 2606.21959 (0 -> 54), 2605.00671 (0 -> 44), 2605.09519 (0 -> 24).
/// OXIDIZED_DESIGN #88; audit family F9(a).
#[test]
fn bib_raw_cite_redefinition_is_ignored() {
  let x = convert_and_post("tests/cluster_regressions/bib_cite_clobber.tex");
  assert!(
    !x.contains("CLOBBERED"),
    "a raw \\def\\cite displaced the binding:\n{x}"
  );
  assert!(
    x.contains("<bibitem") || x.contains("bibitem"),
    "the bibliography did not survive the clobber attempt:\n{x}"
  );
}

/// The flip side of `bib_raw_cite_redefinition_is_ignored`: etoolbox's
/// `\pretocmd`/`\apptocmd` on the LOCKED core `\cite` must assign THROUGH the
/// lock. Unlike a raw `\def\cite`/`\renewcommand\cite` (which stays refused),
/// the hooks wrap the original via `\expandonce#2`, so they are non-destructive
/// and legitimate. Witness ar5iv 2606.01320 does
/// `\pretocmd{\cite}{\stepcounter{cite}}` then gates its whole bibliography on
/// `\ifnum\value{cite}>0`; pre-fix the refused assignment left the counter at 0
/// and the References vanished with no diagnostic. The hook opens a scoped
/// unlock window (`\lx@etb@unlock`/`\lx@etb@relock`, `etoolbox_sty.rs`). The
/// lock is Rust-only (OXIDIZED_DESIGN #88), so this accommodation is too.
#[test]
fn etoolbox_pretocmd_assigns_through_cite_lock() {
  let x = convert_to_xml("tests/cluster_regressions/etoolbox_pretocmd_cite_counter.tex");
  assert!(
    x.contains("GATED-REFERENCES-VISIBLE"),
    "\\pretocmd{{\\cite}}{{\\stepcounter{{cite}}}} was refused by the \\cite \
     CS-lock, so the counter stayed 0 and the counter-gated content vanished:\n{x}"
  );
}

/// natbib citations of a numeric `.bbl` must render as the bracketed number
/// `[N]`/`[N, M]`, not the raw citation key.
///
/// Witness arXiv:2308.06262 (html_feedback#62): `neurips_2023.sty` loads natbib
/// in its default author-year mode, `\bibliographystyle{unsrt}` sits AFTER all
/// the `\cite`s, and the shipped `.bbl` is numeric (plain `\bibitem{key}`, no
/// `[author(year)]` label). Real pdflatex/bibtex handle this via natbib's
/// `\NAT@force@numbers`: a numeric `.bbl` forces numbers mode GLOBALLY, so every
/// `\cite` prints `[N]` regardless of the late `\bibliographystyle`. Golden
/// pdflatex: `[1]`, `[2]`, `[1, 2]`.
///
/// Single-pass LaTeXML freezes the bibref's author-year `show` at `\cite` time,
/// and Perl `CrossRef.pm:542` keeps it (its `|| $keytag` guard is always
/// satisfied), so BOTH engines render the raw key (`alpha ()` / `alpha `). This
/// is the surpass-Perl fix (OXIDIZED_DESIGN #123, KNOWN_PERL_ERRORS #89): when a
/// frozen author-year bibref resolves to entries that are all numeric-only, we
/// collapse to natbib's bracketed number.
#[test]
fn cluster_bib_natbib_late_numeric_style_forces_numbers() {
  // Text content of each `<cite>` in document order, inner tags stripped and
  // whitespace normalized ("[<ref>1</ref>, <ref>2</ref>]" -> "[1, 2]").
  fn cite_texts(xml: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = xml;
    while let Some(i) = rest.find("<cite") {
      let after = &rest[i..];
      let open_end = after.find('>').map(|e| e + 1).unwrap_or(after.len());
      let close = after.find("</cite>").unwrap_or(after.len());
      let inner = &after[open_end..close.max(open_end)];
      let (mut txt, mut depth) = (String::new(), 0);
      for c in inner.chars() {
        match c {
          '<' => depth += 1,
          '>' => depth -= 1,
          _ if depth == 0 => txt.push(c),
          _ => {},
        }
      }
      out.push(txt.split_whitespace().collect::<Vec<_>>().join(" "));
      rest = &after[close.min(after.len())..];
      rest = rest.strip_prefix("</cite>").unwrap_or(rest);
    }
    out
  }
  let x = convert_and_post("tests/cluster_regressions/natbib_late_numeric_style.tex");
  assert_eq!(
    cite_texts(&x),
    vec!["[1]", "[2]", "[1, 2]"],
    "numeric .bbl cited before a late \\bibliographystyle{{unsrt}} must render \
     as the bracketed number, not the raw key:\n{x}"
  );
}

/// arXiv/html_feedback#6742 — a CVPR paper's References showed author-year labels
/// ("Mo et al. [2022]") instead of the numeric `[N]` its PDF prints.
///
/// `cvpr.sty` does `\RequirePackage[numbers,sort&compress]{natbib}` (cvpr.sty L37),
/// so pdflatex+bibtex render numeric citations and a numeric References list. Perl
/// has no cvpr binding, raw-loads cvpr.sty, and gets `numbers` from that first
/// natbib load. Our contrib cvpr binding eagerly pre-loaded natbib WITHOUT options
/// (`cvpr_sty.rs` L30) BEFORE the raw cvpr.sty's `[numbers]` load, which then no-ops
/// as a repeat load (classic `\DeclareOption` clash-and-drop) — leaving `CITE_STYLE`
/// at the default `authoryear`. The fix mirrors the `xcolor` pre-load right above it
/// (which already passes `[dvipsnames,table]` for the same reason): pre-load natbib
/// with the `[numbers,sort&compress]` options cvpr always uses. GENUINE-RUST-ONLY.
#[test]
fn cvpr_natbib_numbers_option_forces_numeric_labels() {
  let x = convert_to_xml_contrib("tests/cluster_regressions/cvpr_natbib_numbers.tex");
  // Bibliography labels are the entry NUMBER `[N]`, not the author-year form.
  assert!(
    x.contains(r#"<tag role="refnum">[1]</tag>"#) && x.contains(r#"<tag role="refnum">[2]</tag>"#),
    "cvpr's [numbers]natbib must label the References [1]/[2], not author-year:\n{x}"
  );
  assert!(
    !x.contains(r#"<tag role="refnum">Mo et al."#),
    "the author-year refnum label leaked — numbers mode was not applied:\n{x}"
  );
  // The inline \cite is in numbers mode too (show="Number"), not author-year.
  assert!(
    x.contains(r#"show="Number""#) && !x.contains(r#"show="Authors Phrase1YearPhrase2""#),
    "inline citations stayed in author-year mode:\n{x}"
  );
}

/// arXiv/html_feedback#6876 (arXiv:2311.15365v3): the witness's biblatex
/// `\DeclareSourcemap{…\maps{…\map{…\step[…\regexp{…}]}}}` block preceded a
/// preamble that an older binary dumped literally into the body (the reporter saw
/// the raw source start exactly at `\DeclareSourcemap\maps`). Guard that the
/// sourcemap block is consumed and the body after it survives, with none of its
/// internals (`\regexp`/`\maps`/`\step`) leaking into the document.
#[test]
fn biblatex_declaresourcemap_does_not_leak_the_preamble() {
  let x = convert_to_xml_contrib("tests/cluster_regressions/biblatex_sourcemap.tex");
  for leak in [
    "DeclareSourcemap",
    "regexp",
    "\\maps",
    "\\step",
    "fieldsource",
  ] {
    assert!(
      !x.contains(leak),
      "the biblatex sourcemap preamble leaked `{leak}` into the document:\n{x}"
    );
  }
  assert!(
    x.contains("BODYAFTERSOURCEMAP"),
    "the document body after the \\DeclareSourcemap block was lost:\n{x}"
  );
}

/// A `refcontext` block must not eat the `\printbibliography` inside it, and
/// `\addbibresource` must accept its optional argument.
///
/// Both are argument-signature bugs in surrogate code Perl does not have.
/// `\refcontext`/`\newrefcontext` read a mandatory group ONLY IF one follows —
/// `\refcontext@i` guards it with `\@ifnextchar\bgroup` and supplies `{}`
/// itself otherwise (biblatex L10437-10445) — so declaring the group
/// unconditionally made the noop swallow whatever came next, which in the
/// idiomatic block IS the bibliography:
///
/// ```text
/// \begin{refcontext}[sorting=nyt]
///     \printbibliography
/// \end{refcontext}
/// ```
///
/// And `\blx@addbib` does `\@ifnextchar[` before the resource
/// (biblatex L11285-11288), so `\addbibresource[location=local]{refs.bib}` is
/// valid; without the `[]` the `[` became the resource name.
///
/// Both lost the whole bibliography in silence. Witnesses 2606.11276 (0 -> 24
/// entries), 2606.02676 (0 -> 93), 2605.27263 (0 -> 58). Audit family F4(a)/(b).
#[test]
fn biblatex_refcontext_block_keeps_its_printbibliography() {
  let x = convert_to_xml_contrib("tests/cluster_regressions/biblatex_ay/refctx.tex");
  assert!(
    x.contains("<bibitem"),
    "refcontext swallowed \\printbibliography — no bibliography at all:\n{x}"
  );
  assert!(
    x.contains("Smith"),
    "the cited entry is missing from the bibliography:\n{x}"
  );
}

/// bibunits' `\putbib` inputs the per-unit `bu<N>.bbl`.
///
/// The real package does exactly that (`bibunits.sty` L324-330): the optional
/// argument only feeds the `.aux` `\bibdata` record, then
/// `\@input@{\@bibunitname.bbl}` runs unconditionally — the same shape
/// `\bibliography` has with `\jobname.bbl`. Perl's binding
/// (`bibunits.sty.ltxml` L78) routes to `\lx@bibliography` only, so it looks
/// for a `.bib` that arXiv submissions do not ship (they ship the generated
/// `bu1.bbl`/`bu2.bbl`, because arXiv does not run bibtex) and the References
/// section comes out empty.
///
/// 15 papers measured across the 2605+2606 sandboxes, every one 0 entries
/// before and complete after — 2606.04416 (79), 2606.28854 (180), 2605.21570
/// (46 = bu1's 34 + bu2's 12, matching each unit's own
/// `\begin{thebibliography}{N}`). Audit family F3(c); OXIDIZED_DESIGN #87.
#[test]
fn bibunits_putbib_reads_the_per_unit_bbl() {
  let x = convert_to_xml("tests/cluster_regressions/bblbib/bibunits.tex");
  assert!(
    x.contains("First unit reference"),
    "\\putbib did not input bu1.bbl, got:\n{x}"
  );
}

/// A bibliography file that cannot be found must be an **Error**, not an Info.
///
/// Perl raises `Error:missing_file` (`Post/MakeBibliography.pm` L138-140) and
/// then falls through to the `Info:expected` below it, so both reach the log.
/// This port hoisted the raw-`.bib` lookup out of that branch and lost the
/// raise, leaving only the Info — which is why a whole family of lost
/// bibliographies reported telemetry `ok` and stayed invisible to every
/// error-keyed sweep: 14 silent papers in the 2605+2606 sandboxes and 691
/// corpus-wide. Witness 2606.04416 (bibunits — the shipped `bu1.bbl` is never
/// consulted and the named `.bib` does not exist; Perl loses the bibliography
/// too, but says so). Audit family F2.
#[test]
fn bib_missing_file_is_an_error_not_just_an_info() {
  let (_xml, log) = convert_and_post_logging("tests/cluster_regressions/bib_missing_file.tex");
  assert!(
    log.contains("Error:missing_file:"),
    "a missing bibliography must raise Error:missing_file, log was:\n{log}"
  );
  // The name that could not be resolved has to be in the message — a bare
  // "couldn't find a bibliography" is not actionable for a multi-`.bib`
  // document. (Perl's `Info:expected` fallthrough is emitted too, but Info
  // sits below this harness's log threshold, so it is not asserted here.)
  assert!(
    log.contains("no_such_bibliography_file"),
    "the raise must name the unresolved bibliography, log was:\n{log}"
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

/// A biblatex *style/variant* package — `biblatex-chicago` here, but the whole
/// `biblatex-<style>` family (`-apa`/`-ieee`/`-nature`/`-science`/`-phys`/…) —
/// must load the biblatex binding, because each such `.sty` in reality does
/// `\RequirePackage{biblatex}` and then only configures it. Its configuration
/// commands (`\DeclareFieldFormat`, `\DeclareFieldAlias`) run in the preamble,
/// so if the variant name is unbound they are undefined at use, and every biber
/// `.bbl` `\endinput`s itself at its `\@ifundefined{ver@biblatex.sty}` guard —
/// emptying the References list outright. html_feedback #6601, witness arXiv
/// 2605.11180 (`\usepackage[authordate,backend=biber]{biblatex-chicago}` via
/// `biblatex-aer.tex`): 13 errors and 0 bibitems before the fix, 0 errors and a
/// full bibliography after. Guards both halves: the preamble `\DeclareFieldFormat`
/// does not leak/error, and the `.bbl` renders its entries.
#[test]
fn cluster_bib_biblatex_variant_loads_and_renders() {
  let x = convert_to_xml_contrib("tests/cluster_regressions/biblatex_ay/chicago.tex");
  assert!(
    !x.contains("DeclareFieldFormat"),
    "preamble \\DeclareFieldFormat leaked raw / errored (biblatex-chicago did not load biblatex):\n{x}"
  );
  assert!(
    x.contains(r#"<bibitem key="smith2020""#),
    "biblatex-chicago produced no References (biber .bbl guard emptied the list):\n{x}"
  );
  assert!(
    x.contains("Smith"),
    "biblatex-chicago bibitem missing its author text:\n{x}"
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

/// amsrefs papers commonly open their references with `\begin{bibsection}`
/// rather than `\begin{bibdiv}` — in real `amsrefs.sty` (L1251/L1265) `bibdiv`
/// IS `bibsection` in article mode, the section-heading wrapper around
/// `{biblist}`. Perl (and, before this, the Rust port) bound only `bibdiv`, so
/// `\begin{bibsection}` was an undefined environment: the `<ltx:biblist>`
/// floated in an `<ltx:p>` (schema-invalid) and the entire References list was
/// lost. The environment now renders the same `<ltx:bibliography>` container as
/// `bibdiv`, titled from its optional arg (default `\refname`). Shared gap with
/// Perl, faithful port of the real `.sty`. html_feedback #1393, witness arXiv
/// 2405.18501 (`\documentclass{amsart}`, `\usepackage[numeric]{amsrefs}`).
#[test]
fn amsrefs_bibsection_environment_renders() {
  let x = convert_and_post("tests/cluster_regressions/amsrefs_bibsection.tex");
  assert!(
    x.contains("<bibitem"),
    "\\begin{{bibsection}} produced no References (environment undefined?):\n{x}"
  );
  assert!(
    !x.contains("<bibentry"),
    "an ltx:bibentry survived unconverted from the bibsection list:\n{x}"
  );
  for needle in ["References", "Chakerian", "Gruber", "Convex"] {
    assert!(
      x.contains(needle),
      "bibsection entry content `{needle}` missing:\n{x}"
    );
  }
}

/// apacite's OLD `.bbl` format (pre-2012 apacite.bst) labels each `\bibitem` with
/// `\BCAY{full}{short}{year}` and formats entries with `\Bem` (emphasis) and
/// `\BBACOMMA`. The modern format (`\citeauthoryear` + `\APACrefauthors`) already
/// converts; these old compat macros were undefined, so a `theapa`/apacite `.bbl`
/// flooded `Error:undefined:\BCAY`/`\Bem`/`\BBACOMMA` and leaked the macro names.
/// apacite_sty now defines them (`\Bem`=`\emph`, `\BCAY`→author label,
/// `\BBACOMMA`=","). Perl ships no apacite binding — Rust surpasses. Related
/// html_feedback#6489 (arXiv 2304.11127, multibib+theapa). The `_clean` helper
/// gates on 0 post-stage errors — the primary red→green signal.
#[test]
fn cluster_apacite_old_bbl_format_renders() {
  let x = convert_and_post_contrib_clean("tests/cluster_regressions/apacite_old_bbl.tex");
  // No old-format macro leaks into the rendered bibliography as raw text.
  for leak in ["BCAY", "\\Bem", "BBACOMMA"] {
    assert!(
      !x.contains(leak),
      "apacite old-format macro `{leak}` leaked into the output:\n{x}"
    );
  }
  // Entries render their real content (authors, title, journal, year).
  for needle in [
    "Smith",
    "Doe",
    "A study of things",
    "Journal of Testing",
    "2020",
  ] {
    assert!(
      x.contains(needle),
      "apacite bibitem content `{needle}` missing:\n{x}"
    );
  }
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

/// An unmatched `$` in a `.bib` field is a currency sign, not a math shift.
///
/// Witness 2605.00166 (`annote = {… of the order of $10 million …}`): **0
/// errors** before the raw `.bib` became a real conversion, **103 and a Fatal**
/// after. A `.bib` is digested as ONE unit, so the never-closed inline-math
/// group crossed `\end{bib@entry}` and every subsequent element landed inside
/// `<ltx:XMath>` — `<ltx:bib-organization> isn't allowed in <ltx:XMath>` ~100
/// times, tripping the 100-error cap. Across the 2605+2606 sandboxes, 33 of the
/// 69 papers whose bibliography sub-conversion newly failed carry an odd `$`.
///
/// Authorized surpass-Perl (OXIDIZED_DESIGN #79): same-host Perl cascades
/// identically (`latexmlc` on 2605.00166 gives 102 errors,
/// `Fatal:too_many_errors:100`, rc=1), because `\bibentry@create` interpolates
/// the field raw into a fresh Mouth. Upstream already agrees an unmatched `$`
/// is not math — `\bib@@title` errors and DELETES it — it just applies that to
/// one field and drops the character. Nothing is suppressed here: the cascade
/// is gone because the entry is well-formed, and `convert_and_post_clean`
/// enforces the zero-post-error gate.
///
/// The load-bearing assertion is `aftermath`, the LAST entry: containment. If a
/// stray `$` leaks again it is the first thing to disappear.
#[test]
fn bib_unmatched_dollar_does_not_leak_math() {
  let x = convert_and_post_clean("tests/cluster_regressions/bib_unmatched_dollar.tex");
  assert!(
    x.contains("<bibitem"),
    "unmatched $: no bibliography at all:\n{x}"
  );
  // Every entry survives — `aftermath` sits after all three stray-`$` entries.
  for needle in [
    "Okonkwo",
    "Lindqvist",
    "Diallo",
    "Park",
    "Raghavan",
    "containment canary",
  ] {
    assert!(
      x.contains(needle),
      "unmatched $: {needle:?} was swallowed by a leaked math group:\n{x}"
    );
  }
  // The currency amount survives as TEXT rather than being absorbed into math.
  assert!(
    x.contains("10 million"),
    "unmatched $: the currency amount vanished:\n{x}"
  );
  // …while a genuinely balanced span is still parsed as math: `$x_1+x_2$` in
  // `mathandcurrency` keeps its subscript, so demoting the later `$8` did not
  // disturb the earlier pair.
  assert!(
    x.contains(r#"role="SUBSCRIPTOP""#),
    "unmatched $: balanced `$x_1+x_2$` stopped being math:\n{x}"
  );
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
  // 1. The five specials, bare, are literal text. `&` is XML-escaped on the way
  //    out, which is the correct rendering of the character. `recase_title`
  //    lowercases (capitalize1, Perl parity), hence `at1g01010`.
  //    The trailing `^` is the one whose escape is NOT `\` + the character:
  //    `\^` is the circumflex accent, so a generic escape would have rendered
  //    the following letter accented ("ĉaret") instead of a caret.
  let bare = "AT&amp;T dataset at1g01010_v2 at 95% with #3 replicates ^ caret";
  assert_eq!(
    title_of("barespecials"),
    bare,
    "bib specials: bare `& _ % # ^` did not render literally:\n{x}"
  );
  // 2. Idempotency: the entry that already wrote `\&`/`\_`/`\%`/`\#`/
  //    `\textasciicircum{}` renders the SAME string. Had the escaper
  //    double-escaped, `\&` would have become `\\&` — a line break followed by
  //    an ampersand.
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

/// The `mathscinet.sty` binding, on the load path a document actually uses.
///
/// `\Dbar` (Đ), `\cprime` (ь) and friends are not kernel commands: they belong
/// to `mathscinet.sty`, AMS v1.05, shipped in the amsrefs bundle. Perl LaTeXML
/// has `amsrefs.sty.ltxml` but no `mathscinet.sty.ltxml`, so the binding is a
/// Rust-only addition — ported from the real `.sty`, whose T1 branches say what
/// each glyph IS (`\Dbar`->`\DJ`, `\cprime`->`\tprime`) rather than how the
/// Default branches draw it with `\accent` and `\hbox` kerning.
///
/// Both arrival points in one fixture: body prose (2508.13753 uses `\cprime`
/// there, at L2131) and a `.bib` `MRREVIEWER` (2605.11579's `biblo.bib` L2059).
/// All three `\cprime` witnesses load the package by name — 2508.13753 L7,
/// 2508.20226 L3, 2509.07628 L13 — which is the evidence that moved the family
/// out of `latex_constructs.rs` and into this binding.
#[test]
fn bib_mathscinet_package_supplies_its_transliteration_glyphs() {
  let x = convert_and_post_clean("tests/cluster_regressions/bib_mathscinet_package.tex");
  // Composed characters, not source and not a silently-dropped letter.
  for (needle, what) in [
    ("Kondratʹev", "`\\cprime` in body prose"),
    ("Đokovi", "`\\Dbar` in a bib reviewer field"),
  ] {
    assert!(
      x.contains(needle),
      "mathscinet: {what} did not compose ({needle:?} missing):\n{x}"
    );
  }
  for leaked in ["Dbar", "cprime"] {
    assert!(
      !x.contains(leaked),
      "mathscinet: `\\{leaked}` leaked into the output as source:\n{x}"
    );
  }
  assert!(
    x.contains("<bibitem"),
    "mathscinet: no bibliography at all:\n{x}"
  );
}

/// A document that does NOT load mathscinet keeps its own `\Dbar` — and nothing
/// loads the package on its behalf.
///
/// Two things are pinned here, and the second is the one that is easy to lose.
///
/// 1. The binding must not become an always-present kernel definition. That
///    would run BEFORE the document's preamble, and LaTeXML's `\newcommand` over
///    an already-defined CS silently keeps the OLD meaning — no error, no
///    warning — so the author's macro would be shadowed with nothing in the log.
///    Measured on 4,000 papers of arXiv 2605: six define `\Dbar` themselves
///    (four with `\newcommand`) against two that use it undefined, and all
///    twelve `\dbar` definitions mean a thermodynamic differential or a barred
///    derivative, never a Croatian letter.
/// 2. The recursive `.bib` session must not auto-load the package either. It
///    would be tempting — the session already loads `url.sty` that way — but
///    `\Dbar` has no `.bst` behind it: witness 2605.11579 uses
///    `\bibliographystyle{alpha}` and `alpha.bst` contains no `Dbar`, so real
///    pdflatex raises the same undefined control sequence. Auto-loading would
///    lower an error count below what the author's own build produces.
#[test]
fn bib_mathscinet_macro_yields_to_the_authors_own_definition() {
  let x = convert_and_post_clean("tests/cluster_regressions/bib_mathscinet_author_macro.tex");
  // The author's own `\Dbar` is a barred D in math, and must survive verbatim.
  assert!(
    x.contains("<XMApp>"),
    "author `\\Dbar`: the barred-D math did not render as math:\n{x}"
  );
  assert!(
    !x.contains("Đ"),
    "author `\\Dbar`: MathSciNet's Đ shadowed the author's own \
     `\\newcommand{{\\Dbar}}` — the binding must not reach a document that \
     never loads the package:\n{x}"
  );
  // …and the bibliography still renders, so the guard cannot pass by losing it.
  assert!(
    x.contains("<bibitem"),
    "author `\\Dbar`: no bibliography at all:\n{x}"
  );
}

/// A `.bib`'s `@preamble` is TeX source, executed once before any entry.
///
/// This is the one part of a `.bib` that `bibtex(1)` does not treat as data: it
/// copies the block verbatim to the top of the `.bbl` (plain.bst `begin.bib`,
/// `preamble$ write$`, ahead of `\begin{thebibliography}`), so pdflatex has the
/// definitions before the first `\bibitem`. It is how a `.bib` ships the macros
/// its own fields use, and MathSciNet's exporter emits one routinely —
/// `@preamble{"\def\cprime{$'$} "}` in 2605.00097's `referLiu.bib` (L11) and
/// fourteen times over in 2605.11579's `biblo.bib` (L4768, L6910, …).
///
/// We read `.bib` directly, so performing that copy is ours to do: Perl
/// `Pre/BibTeX.pm::toTeX` L118-122 joins the preamble lines ahead of
/// `\begin{bibtex@bibliography}`, and `pre_bibtex::to_tex` mirrors it. Verified
/// end to end against same-host `latexmlc` on this exact fixture: both engines
/// 0 errors, both expand every macro.
///
/// Under the two-treatment frame (`docs/parity/BIBLIOGRAPHY_WORKLIST.md`) the
/// preamble belongs to treatment 2 — the TeX regime — and must therefore NOT
/// pass through `escape_bib_data_specials`, which would turn `\def\cprime{$'$}`
/// into inert text. Nothing else guards that: `to_tex_includes_preamble`
/// (`pre_bibtex.rs`) asserts on the emitted STRING, so it stays green even if
/// the string is never executed, is escaped on the way to the tokenizer, or is
/// read under `Mouth::with_bib_data_literals`.
///
/// The fixture's macro names are unique to it on purpose. `\cprime` is defined
/// always-on in `latex_constructs_rust_only.rs`, so probing with it would pass
/// whether or not the preamble ran at all.
#[test]
fn bib_preamble_defines_macros_for_the_whole_bibliography() {
  let x = convert_and_post_clean("tests/cluster_regressions/bib_preamble.tex");
  // A parameter survives: `#` reaches the tokenizer at catcode 6, so
  // `\newcommand{\bibpreamblearg}[1]{[[#1]]}` is a real one-argument macro.
  assert!(
    x.contains("[[ARGWORKS]]"),
    "@preamble: the parameterized macro did not expand:\n{x}"
  );
  // `\&` and `\%` in the preamble are TeX escapes, not data to be re-escaped.
  assert!(
    x.contains("Taylor &amp; Francis"),
    "@preamble: `\\&` in a preamble macro body did not render:\n{x}"
  );
  // The SECOND entry uses a preamble macro: the definitions are installed once,
  // before the entries, not scoped to whichever entry happens to be first.
  assert!(
    x.contains("100%"),
    "@preamble: the second entry lost the preamble macro — definitions must \
     outlive the first entry:\n{x}"
  );
  // The MathSciNet shape verbatim, in the field it actually occupies: a name.
  // `$'$` composes to a prime, and no source leaks.
  assert!(
    x.contains("Gel\u{2032}fand"),
    "@preamble: `\\def\\bibpreambleprime{{$'$}}` did not compose in a name:\n{x}"
  );
  assert!(
    !x.contains("bibpreamble"),
    "@preamble: a preamble macro leaked into the output as source:\n{x}"
  );
}
/// The secondary `MakeBibliography` parity gaps of SYNC_STATUS R5 item 2, on one
/// alpha-styled document: the swapped `citestyle` mapping, the disambiguation
/// key read off the wrong name string, collating `unisort`, format-order
/// numbering — and the fourth audit item that turned out NOT to be a gap. See
/// `bib_alpha_style.tex` for the list and `bib_alpha_style_refs.bib` for why
/// each entry is present.
///
/// Every expectation here is ground-truthed against same-host Perl LaTeXML
/// 0.8.8 on this exact fixture, after which the two engines' bibliographies are
/// byte-identical on it.
#[test]
fn cluster_bib_alpha_style_labels() {
  let x = convert_and_post_clean("tests/cluster_regressions/bib_alpha_style.tex");

  // Every bibitem in document order, as (refnum class, refnum text).
  let mut labels: Vec<(String, String)> = Vec::new();
  let mut rest = x.as_str();
  while let Some(i) = rest.find("<bibitem") {
    let item_end = rest[i..]
      .find("</bibitem>")
      .map(|e| i + e)
      .unwrap_or(rest.len());
    let item = &rest[i..item_end];
    if let Some(r) = item.find(r#"role="refnum""#) {
      let tag_start = item[..r].rfind("<tag").expect("refnum tag start");
      let tag = &item[tag_start..];
      let class = tag
        .find(r#"class=""#)
        .map(|c| {
          let c = tag_start + c + 7;
          let e = item[c..].find('"').expect("class end") + c;
          item[c..e].to_string()
        })
        .unwrap_or_default();
      let open = tag.find('>').expect("tag open") + tag_start + 1;
      let close = item[open..].find("</tag>").expect("tag close") + open;
      labels.push((class, item[open..close].to_string()));
    }
    rest = &rest[item_end..];
  }

  // Gap 1 — `\bibliographystyle{alpha}` (CITE_STYLE=AY) is Perl's ABBREVIATED
  // label branch, not the spelled-out author-year one.
  assert!(
    labels.iter().all(|(c, _)| c == "ltx_bib_abbrv"),
    "every refnum should be an abbreviated alpha label, got {labels:?}\n{x}"
  );
  assert!(
    !x.contains("ltx_bib_author-year"),
    "an alpha bibliography must not emit author-year refnums:\n{x}"
  );

  // Gap 4 — collation: `ångström` sorts between `adams` and `baker`, not after
  // every ASCII letter as a codepoint sort would place it. (The numbering that
  // rides on the same order is asserted by construction: the labels are read in
  // document order.)
  // Gap 2 — the two `Smith`-led entries share Perl's short-form `{ay}`, so they
  // are disambiguated `a`/`b`; a full-author key never collides.
  // Also covers the `make_alpha_label` char-boundary panic: `Ångström`'s
  // multi-byte initial used to be byte-sliced.
  let texts: Vec<&str> = labels.iter().map(|(_, t)| t.as_str()).collect();
  assert_eq!(
    texts,
    vec!["ADA01", "\u{c5}NG02", "BAK03", "SBC99a", "SDE99b"],
    "bibitem order and alpha labels:\n{x}"
  );

  // Gap 3 was NOT a gap — KNOWN_PERL_ERRORS #67. Perl's `do_year` reads the
  // ARRAY `@…::SUFFIX` while `formatBibEntry` binds the SCALAR `$…::SUFFIX`,
  // so the letter never reaches the entry body; same-host Perl 0.8.8 prints
  // ` (1999)` here, and `alpha.bst` agrees. Pinned so a future reader of
  // L613-615 does not "fix" it back.
  for needle in ["An alpha study", "A beta study"] {
    let i = x
      .find(needle)
      .unwrap_or_else(|| panic!("no entry titled {needle} in:\n{x}"));
    let start = x[..i].rfind("<bibitem").expect("bibitem start");
    let end = x[start..]
      .find("</bibitem>")
      .map(|e| start + e)
      .unwrap_or(x.len());
    let item = &x[start..end];
    assert!(
      item.contains("(1999)") && !item.contains("(1999a)") && !item.contains("(1999b)"),
      "the entry body of {needle} should carry the bare year, as Perl does:\n{item}"
    );
  }
}

/// `\xpatchcmd` must not swallow the rest of the document.
///
/// xpatch is an expl3 package: each public command is a `\NewDocumentCommand`
/// dispatching to `\xpatch_main:NN`, which re-reads the target's body delimited
/// by a sentinel token list, `\c__xpatch_bizarre_tl` = `**)-(**/**]-[**`. With
/// no binding, `--includestyles` raw-loaded it and that delimited scan ran to
/// end-of-file, so everything after the first `\xpatchcmd` was discarded — with
/// **zero** diagnostics.
///
/// Witness 2605.25157: `\xpatchcmd{\@tocline}` in the preamble, output truncated
/// mid-proof at source line 1292 of 1749, its own `\begin{thebibliography}` with
/// 33 `\bibitem`s never reached; now 33 entries and no errors. Perl truncates
/// identically (it raises `Error:expected:Until:**)-(**/**]-[**` twice, so our
/// silence was strictly worse) — beyond-Perl, ground truth the arXiv PDF.
/// 10 papers of the 2605+2606 residual load xpatch.
///
/// The `comment` environment here is the original witness's shape and is what
/// made the loss visible: its raw-line body skip found the mouth already at EOF
/// and reported "Skipped comment (0 lines)".
///
/// Note what this guards: the runaway needs the RAW `.sty`, which only
/// `--includestyles` loads, and a registered binding now wins over the raw load.
/// So this pins the binding — that every `\x…` command is defined and leaves the
/// document intact — which is what keeps the raw-load path unreachable.
#[test]
fn bib_xpatch_does_not_truncate_the_document() {
  let x = convert_and_post_clean("tests/cluster_regressions/bib_xpatch_truncation.tex");
  assert!(
    x.contains("After the comment block"),
    "\\xpatchcmd swallowed the rest of the document:\n{x}"
  );
  let n = x.matches("<bibitem").count();
  assert_eq!(
    n, 2,
    "expected both bibitems past the \\xpatchcmd, got {n}\n{x}"
  );
  assert!(
    !x.contains("suppressed line"),
    "the comment environment's body leaked into the output:\n{x}"
  );
}

/// A `&` inside a delimiter-fenced macro argument must not end an alignment cell.
///
/// `tex.web` §394 `macro_call` sets `align_state:=1000000` while it scans a
/// macro's parameters, so an argument's `&` is an ordinary token. Neither Perl
/// nor this port modelled that, and the brace form hid it — cell scanning skips
/// balanced groups, but `(…)` is not a group. So `\myfence( a &0 \\ 0 &b )`
/// inside an `eqnarray` split the row mid-argument, orphaned the fences, and the
/// alignment could not close its group:
/// `Error:unexpected:\lx@begin@alignment Attempt to close a group that switched
/// to mode restricted_horizontal`. The document was truncated there and the
/// bibliography went with it.
///
/// Perl raises the identical error (11 on the 14-line repro), so `pdflatex` —
/// which renders it silently — is the ground truth. Divergence #90. This was the
/// largest cluster of the 2026-07-29 bibliography-absence residual, 28 papers;
/// witnesses 2605.05903 and 2007.06211 (revtex4-1 + physics `\mqty`).
#[test]
fn alignment_fenced_amp_does_not_split_a_row() {
  let x = convert_and_post_clean("tests/cluster_regressions/bib_alignment_fenced_amp.tex");
  assert!(
    x.contains("After the equation"),
    "the fenced `&` truncated the document:\n{x}"
  );
  let n = x.matches("<bibitem").count();
  assert_eq!(
    n, 2,
    "expected both bibitems past the alignment, got {n}\n{x}"
  );
}

/// A `\columncolor` in a p/m column, followed by a cell whose content starts with
/// `\lbrack` (a CS `\let` to `[`), made colortbl's overhang-arg gobble run forward
/// past the row's `\\`, cascading the paragraph-column mode
/// (`\@end@tabular … internal_vertical`) — truncating the document and dropping the
/// bibliography below. `\toprule[1pt]` (a booktabs rule with an optional-width arg)
/// completes the trigger. Reading the overhang args with LaTeXML's optional reader
/// (literal `[` only, never a `\lbrack` CS) keeps the cell content, as pdflatex
/// renders it. Rust-only; witness arXiv:2606.02077 (0 -> 15 bibitems, matching Perl).
#[test]
fn columncolor_lbrack_cell_does_not_cascade_the_column_mode() {
  let x = convert_and_post_clean("tests/cluster_regressions/bib_columncolor_lbrack_overhang.tex");
  assert!(
    x.contains("[3]") && x.contains("second cell"),
    "the `\\lbrack` cell content was eaten as a colortbl overhang arg:\n{x}"
  );
  let n = x.matches("<bibitem").count();
  assert_eq!(
    n, 1,
    "the paragraph-column mode cascade truncated the bibliography, got {n}\n{x}"
  );
}

/// chapterbib's per-unit bibliography `lists` id must be the RAW unit name so it
/// matches the citation-side `<ltx:bibref inlist=...>`; otherwise MakeBibliography
/// selects nothing (0 cited) and the References render empty. An `\include`
/// basename with `_` was the trigger: `\lx@cb@unitname` explodes it to catcode-12
/// OTHER tokens, but the digested `[unit]` arg then rendered `_` through the active
/// OT1 font (slot 0x5F = `˙` U+02D9), so `lists="chap˙one"` no longer equalled
/// `inlist="chap_one"`. Building `lists` from the arg's reverted SOURCE fixes it,
/// matching Perl (which keeps `_`). Rust-only; witness arXiv:2605.15421 (0 -> 101
/// bibitems). The `\include`d `chap_one.tex` + `refs.bib` sit in the fixture's own
/// subdir so the non-recursive `cluster_regressions` streaming sweep skips them.
#[test]
fn chapterbib_underscore_unit_lists_matches_inlist() {
  let x = convert_and_post_clean("tests/cluster_regressions/chapterbib_underscore/main.tex");
  let n = x.matches("<bibitem").count();
  assert_eq!(
    n, 1,
    "underscore unit name dropped the cited entry (lists/inlist OT1 mismatch), got {n}\n{x}"
  );
}

/// A class we have no binding for must still get biblatex when the document
/// asks for it.
///
/// A paper whose CLASS loads biblatex on its behalf never writes
/// `\usepackage{biblatex}` itself. With no binding for that class the document
/// falls back to OmniBus, `\addbibresource` is undefined, and the bibliography
/// is lost outright. `maybe_require_dependencies` does scan the shipped `.cls`
/// text for `\RequirePackage` names, but cannot resolve one built from macros —
/// now-journal.cls L267 asks for `now-\now@journalkey-biblatex` — and under
/// OmniBus that line never executes either.
///
/// OmniBus now `def_autoload`s biblatex from `\addbibresource` and
/// `\printbibliography`: both are biblatex-only and neither overlaps the natbib
/// trigger list beside it, so no document can pull in both backends this way.
/// Perl autoloads natbib but never biblatex (`OmniBus.cls.ltxml` L48-51), so
/// this is beyond Perl.
///
/// 7 papers of the 2026-07-29 residual recovered, 548 entries: 2606.06995
/// 0 -> 250, 2606.06922 0 -> 76, 2606.10538 0 -> 66, 2605.09073 0 -> 59,
/// 2605.01204 0 -> 46 (`\documentclass[sip,biber]{now-journal}`), 2605.06766
/// 0 -> 46, 2605.30377 0 -> 5 (which cites exactly 5 keys of a 19-entry
/// `.bib` — bibtex emits only what is cited). Four of those had an unrelated
/// first error (`\alsoaffiliation`, `\maintitleauthorlist`), a reminder that the
/// first error is often incidental to the loss.
#[test]
fn bib_biblatex_autoloads_under_an_unbound_class() {
  let x = convert_and_post_contrib_clean("tests/cluster_regressions/bib_biblatex_autoload.tex");
  let n = x.matches("<bibitem").count();
  assert_eq!(
    n, 2,
    "biblatex did not autoload under the fallback class: {n} entries\n{x}"
  );
  assert!(
    x.contains("autoloaded biblatex entry") && x.contains("second autoloaded entry"),
    "the autoloaded entries did not render their titles:\n{x}"
  );
}

/// The recursive `.bib` session must mint `bib`-rooted entry ids, so a
/// single-bibliography document's anchors come out `bib.bibN` — byte-identical
/// to same-host Perl (`<li id="bib.bib1">`).
///
/// Perl builds a FRESH State per `.bib` (`MakeBibliography.pm` L181-215), so
/// its inner session always sees `n_bibliographies == 0` and names ids
/// `bib<radix_alpha(0)>` = `bib`. We deliberately SHARE the live State
/// (`bib_session.rs` module docs — the enhancement Perl's own comment asks
/// for), which leaked the outer document's count into
/// `begin_bibliography_clean` (`latex_constructs.rs:2392-2398`): the inner
/// session saw 1, minted `biba`, and every anchor came out `biba.bibN` — the
/// SECOND slot of the multi-bibliography sequence, on a document with one
/// bibliography. `bib_session::convert` now hides that count for the duration
/// of the recursive digestion.
///
/// Post then strips `^bib` and re-prepends the OUTER bibliography's id
/// (`MakeBibliography.pm` L407-415), which is why the inner ids must be
/// `bib`-rooted whichever bibliography they land in — `structure/crazybib`
/// covers the multi-bibliography sequence (`bib` / `biba` / `bibb`).
#[test]
fn bib_entry_ids_are_bib_rooted_like_perl() {
  let x = convert_and_post_clean("tests/cluster_regressions/bib_abstract_percent.tex");
  assert!(
    x.contains(r#"xml:id="bib.bib1""#),
    "first bibitem must be bib.bib1 (Perl parity), got ids: {:?}",
    x.match_indices("xml:id=\"bib")
      .map(|(i, _)| x[i..(i + 24).min(x.len())].to_string())
      .take(6)
      .collect::<Vec<_>>()
  );
  assert!(
    !x.contains(r#"xml:id="biba."#),
    "no entry may use the SECOND bibliography's prefix in a one-bibliography document"
  );
  // The citation links must agree with the anchors they point at.
  for n in 1..=4 {
    let id = format!("bib.bib{n}");
    assert!(
      x.contains(&format!(r#"xml:id="{id}""#)),
      "missing bibitem anchor {id}"
    );
  }

  // The enclosing biblist needs a `fragid` as well as an `xml:id`: the HTML5
  // XSLT's `add_id` emits the HTML `id` from `@fragid` ALONE, and `fragid` is
  // stamped by `CrossRef::fill_in_frags` only for nodes carrying an ObjectDB
  // `ID:` entry. Without MakeBibliography registering the list, Perl's
  // `<ul id="bib.L1" class="ltx_biblist">` came out here as a bare `<ul>`.
  assert!(
    x.contains(r#"xml:id="bib.L1""#),
    "biblist must keep its xml:id"
  );
  assert!(
    x.contains(r#"fragid="bib.L1""#),
    "biblist must receive a fragid, or the HTML <ul> loses its id entirely"
  );
}

/// Id-bearing markup CLONED INTO a bibliography entry (here `$E=mc^2$` in a
/// title) must keep an id all the way to HTML.
///
/// Perl's `Collector::rescan` (Collector.pm L97, called from
/// MakeBibliography.pm L71/L78) re-runs the whole Scan over the generated
/// bibliography, so such nodes get an ObjectDB entry — which is the
/// precondition `CrossRef::fill_in_frags` needs before it stamps `fragid`,
/// and the HTML5 XSLT emits the HTML `id` from `@fragid` alone. This port has
/// no rescan, so the cloned `ltx:Math` reached HTML with no id at all
/// (measured against same-host Perl 0.8.8, which emits `bib.bib1.m1a`).
/// MakeBibliography now registers every id-bearing node of the generated
/// subtree.
///
/// The id NAME is allowed to differ from Perl's: Perl's trailing `a` is a
/// `uniquifyID` artifact of cloning while the source bibentry still carried
/// the same id, which it then deletes. What must hold is that the node keeps
/// an id and that nothing dangles.
#[test]
fn bib_entry_cloned_markup_keeps_its_id() {
  let x = convert_and_post_clean("tests/cluster_regressions/bib_title_math.tex");
  let math_ids: Vec<&str> = x
    .match_indices("<Math")
    .filter_map(|(i, _)| {
      let tail = &x[i..(i + 200).min(x.len())];
      tail
        .find("xml:id=\"")
        .map(|j| &tail[j + 8..])
        .and_then(|s| s.find('"').map(|e| &s[..e]))
    })
    .collect();
  assert!(
    math_ids.iter().any(|id| id.starts_with("bib.")),
    "the math cloned into the bib title must keep a bib-rooted xml:id, got {math_ids:?}"
  );
  // …and the fragid that turns it into an HTML id.
  for id in math_ids.iter().filter(|i| i.starts_with("bib.")) {
    assert!(
      x.contains(&format!(r#"fragid="{id}""#)),
      "cloned bib markup {id} must receive a fragid"
    );
  }
}

/// Issue #410: a space-form cedilla accent `\c c` in a `.bib` field — the
/// MathSciNet `Fran\c cois` idiom — must digest to "François", NOT weld into an
/// undefined `\ccois` control sequence (the control-word-terminating space
/// dropped). The recursive BibTeX session digests EVERY field of a cited entry
/// including the non-rendered `MRREVIEWER`, so a weld surfaces as an
/// `undefined:\ccois` Error that `convert_and_post_clean`'s 0-error gate
/// catches; the rendered AUTHOR field pins the visible "François". This was
/// GENUINE-RUST-ONLY at filing (same-host Perl 0.8.8 renders "François"); the
/// expl3-cedilla catcode fix + the bib-absence campaign resolved it, and this
/// guards the `.bib` field path specifically (the `expl3_accent_catcode` test
/// only covers the INLINE accent, not the BibTeX session). Witness
/// arXiv:2605.11579 (`MRREVIEWER = {Fran\c cois\ Digne}`, 9 errors + empty
/// bibliography → 0 errors + 36 cited entries).
#[test]
fn cluster_bib_review_field_cedilla_not_welded() {
  let x = convert_and_post_clean("tests/cluster_regressions/bib_accent_review.tex");
  assert!(
    x.contains("François"),
    "the .bib AUTHOR `Digne, Fran\\c cois` must render François:\n{x}"
  );
  assert!(
    !x.contains("ccois"),
    "the space-form accent `\\c c` welded into an undefined `\\ccois` \
     (control-word-terminating space lost in the .bib field path):\n{x}"
  );
}

/// html_feedback #6720 (arXiv:2607.03177): a biblatex `\DeclareSourcemap`
/// preamble block (a biber source-mapping stage LaTeXML does not run) must not
/// break the conversion. `\DeclareSourcemap` was undefined, so its nested
/// `\maps`/`\map`/`\step`/`\regexp` reached the stomach as undefined control
/// sequences and cascaded into math-mode/`\fi` fatals. It (and its
/// `\DeclareStyleSourcemap` sibling) now gobble the rule argument as a no-op;
/// the 0-error gate below catches any regression of that cascade.
#[test]
fn cluster_biblatex_declaresourcemap_is_a_noop() {
  let x = convert_to_xml_contrib_clean("tests/cluster_regressions/biblatex_declaresourcemap.tex");
  // The document body survives — the citation and its bibresource are intact.
  assert!(
    x.contains(r#"bibrefs="beta""#),
    "the \\cite survived the sourcemap preamble:\n{x}"
  );
}

// ===========================================================================
// Citation ⇄ reference label CONSISTENCY cluster.
//
// BibTeX's core guarantee: the label of an inline citation matches the label
// printed next to that entry in the References list, in whatever style the
// bibliography uses (numeric `[N]`, or author-year `Author (Year)`). Perl
// LaTeXML's CrossRef fill phase (`make_bibcite`) honors this by reading the
// bibitem's author/year tags from the ObjectDB (populated by Scan's
// `bibitem_handler`, re-run over the generated bibliography by `rescan`) and
// interleaving them with the bibref's `<ltx:bibrefphrase>` markup.
//
// Minimal reproductions for the class of arXiv html_feedback reports where the
// inline citation does NOT match the list — the "bucket A" inline-vs-list
// mismatch: #6276 ("citations are numbered but bibliography is not"), #6302
// ("reference to [number] but hard to tell which one"), #6307 (inline by number,
// bibliography by author-year), #6534 (inline `[5]`, list "name et al" with no
// number), #6026 (list "Sensoy et al. [2018]", unclear which in-text cite maps).
// All are the same root cause: in author-year mode the inline `\cite` fell back
// to a bare number because the bibitem's author/year tags were never registered
// into the CrossRef DB. Expand this cluster with a new fixture per distinct
// reproducer.
//
// Reference numbering ORDER for citation-order styles (`unsrt`/`IEEEtran`,
// #6294/#5930) is a sibling cluster guarded just below
// (`cluster_bib_*_citation_order`): those styles number by first-citation
// appearance, not alphabetically — a deliberate surpass-Perl (Perl always
// alphabetizes, OXIDIZED_DESIGN divergence).
//
// NOT this cluster (distinct root causes, separate work): raw citekeys leaking
// from an un-emulated cite command (#6203/#6355/#6549/#6050), genuine
// non-resolution `?`/`[undef]` (#6489/#6601/#6222), a missing/empty References
// list (#6432/#6288/#6179).
// ===========================================================================

/// Tag-stripped text of each inline `<ltx:cite>` in post XML.
fn inline_cite_texts(xml: &str) -> Vec<String> {
  let mut out = Vec::new();
  let mut rest = xml;
  while let Some(s) = rest.find("<cite") {
    let after = &rest[s..];
    let Some(e) = after.find("</cite>") else {
      break;
    };
    let seg = &after[..e];
    let mut txt = String::new();
    let mut in_tag = false;
    for c in seg.chars() {
      match c {
        '<' => in_tag = true,
        '>' => in_tag = false,
        _ if !in_tag => txt.push(c),
        _ => {},
      }
    }
    out.push(txt.trim().to_string());
    rest = &after[e + "</cite>".len()..];
  }
  out
}

/// A purely numeric citation label (`2` or `[2]`) — the numeric style. In
/// author-year mode an inline cite that reduces to this is the bug: it can no
/// longer be matched to the `Author (Year)` label the References list shows.
fn numeric_label(s: &str) -> Option<u32> {
  let t = s
    .trim()
    .trim_start_matches('[')
    .trim_end_matches(']')
    .trim();
  if t.is_empty() { None } else { t.parse().ok() }
}

/// html_feedback #6276/#6302 — natbib AUTHOR-YEAR mode: the inline `\cite`/
/// `\citet`/`\citep` must render the author-year label the References list
/// shows, NOT a bare number.
///
/// The bug was GENUINE-RUST-ONLY: MakeBibliography registered only the bibitem
/// `number` into the CrossRef DB — never `authors`/`year` — so
/// `crossref::fill_in_bibrefs` fell back to the number for a `show="Authors …"`
/// bibref (inline `2` vs the list's `Beta (2002)`). Perl 0.8.8 renders
/// `Beta (2002)` inline. Fix: the rescan registers the bibitem's tags via the
/// shared `bibitem_tag_props`, and `fill_in_bibrefs` interleaves author/year
/// with the `<ltx:bibrefphrase>` children (`Beta (2002)` for `\citet`,
/// `(Beta, 2002)` for `\citep`), matching Perl `CrossRef::make_bibcite`.
#[test]
fn cluster_cite_authoryear_inline_matches_reference_label() {
  let x = convert_and_post_clean("tests/cluster_regressions/cite_labels_authoryear.tex");
  assert!(
    x.contains("ltx_bib_author-year"),
    "expected an author-year References list:\n{x}"
  );
  let cites = inline_cite_texts(&x);
  assert!(!cites.is_empty(), "no inline citations were rendered:\n{x}");
  // Every author-year inline cite carries the AUTHOR NAME (matching the list) —
  // none collapses to a bare/bracketed number.
  assert!(
    cites.iter().all(|c| numeric_label(c).is_none()),
    "an author-year inline citation rendered as a bare number, mismatching the \
     References list label: {cites:?}\n{x}"
  );
  // \citet{beta} → "Beta (2002)"; \citep{beta} → "(Beta, 2002)".
  assert!(
    cites
      .iter()
      .any(|c| c.contains("Beta") && c.contains("(2002)")),
    "\\citet should render \"Beta (2002)\": {cites:?}\n{x}"
  );
  assert!(
    cites.iter().any(|c| c.contains("(Beta, 2002)")),
    "\\citep should render \"(Beta, 2002)\": {cites:?}\n{x}"
  );
}

/// Control + regression guard for the numeric path: numeric natbib mode is
/// already consistent (inline `[N]` indexes the numbered list), and the
/// author-year fix must not disturb it.
#[test]
fn cluster_cite_numeric_inline_matches_reference_label() {
  let x = convert_and_post_clean("tests/cluster_regressions/cite_labels_numeric.tex");
  assert!(
    !x.contains("ltx_bib_author-year"),
    "numeric mode must not produce an author-year list:\n{x}"
  );
  let cites = inline_cite_texts(&x);
  assert!(!cites.is_empty(), "no inline citations were rendered:\n{x}");
  // Each inline numeric cite is the entry number (bracketed) indexing the list.
  assert!(
    cites.iter().all(|c| numeric_label(c).is_some()),
    "numeric inline citations should be numbers indexing the list: {cites:?}\n{x}"
  );
}

/// The same consistency guarantee for biblatex author-year (style=apa): the fix
/// is at the shared CrossRef/Scan layer, so a biblatex `\parencite`/`\textcite`
/// renders the author-year label the biblatex list shows — not a bare number.
/// The sibling `cluster_biblatex_authoryear` guards the CORE `<ltx:tag>`s; this
/// guards the POST inline-cite fill phase. (Reuses the rich biblatex_ay/ay.tex.)
#[test]
fn cluster_biblatex_authoryear_inline_matches_reference_label() {
  let x = convert_and_post_contrib_clean("tests/cluster_regressions/biblatex_ay/ay.tex");
  // biblatex's `.bbl`-formatted list carries author-year refnum labels (not the
  // MakeBibliography `ltx_bib_author-year` class, which this path skips).
  assert!(
    x.contains("Smith (2020)") && x.contains("Jones &amp; Brown (2019)"),
    "expected author-year labels in the biblatex References list:\n{x}"
  );
  let cites = inline_cite_texts(&x);
  // `\parencite{smith2020}` → "(Smith, 2020)"; `\textcite{jones2019}` →
  // "Jones & Brown (2019)" — the author-year label, matching the list entries
  // "Smith (2020)" / "Jones & Brown (2019)".
  assert!(
    cites.iter().any(|c| c.contains("(Smith, 2020)")),
    "\\parencite should render author-year \"(Smith, 2020)\": {cites:?}\n{x}"
  );
  assert!(
    cites
      .iter()
      .any(|c| c.contains("Jones") && c.contains("(2019)")),
    "\\textcite should render \"Jones & Brown (2019)\": {cites:?}\n{x}"
  );
  // The raw citekeys must never leak into the inline text.
  assert!(
    !x.contains(">smith2020<") && !cites.iter().any(|c| c.contains("smith2020")),
    "a raw biblatex citekey leaked into an inline citation:\n{x}"
  );
}

/// html_feedback #6609: the whole REVTeX4 family (`revtex4`, `revtex4-1`,
/// `revtex4-2`) must render NUMERIC citations BY DEFAULT — bracketed `[N]` inline
/// and a numbered reference list, like the published PDF — NOT natbib's
/// author-year default.
///
/// The real classes default to `numerical` (revtex4-2.cls L6024, revtex4-1.cls
/// L6003 → `\@booleanfalse\authoryear@sw`; revtex4 4.0 is numeric via
/// `\NAT@citenum`). Perl gives author-year for `revtex4`/`revtex4-1` and only
/// stumbles onto numeric for `revtex4-2` via a version-fallback to `revtex.cls`
/// — so this is a deliberate surpass-Perl to match the real class + PDF. Oxide
/// preloads natbib[numbers] in the class before revtex4_support pulls natbib in.
/// Witness arXiv 2606.09494 (`\bibliography{references/references}`, 65 cited).
#[test]
fn cluster_bib_revtex4_family_numeric_by_default() {
  for (tex, class) in [
    ("revtex4_2_numeric_cite.tex", "revtex4-2"),
    ("revtex4_1_numeric_cite.tex", "revtex4-1"),
    ("revtex4_numeric_cite.tex", "revtex4"),
  ] {
    let path = format!("tests/cluster_regressions/{tex}");
    // Root cause is the engine-stage citestyle attribute.
    let core = convert_to_xml(&path);
    assert!(
      core.contains(r#"citestyle="numbers""#),
      "{class} must default to citestyle=\"numbers\":\n{core}"
    );
  }
  // Full render for the witness class: numeric refnum (`ltx_bib_key` `[N]`),
  // never author-year.
  let x = convert_and_post_clean("tests/cluster_regressions/revtex4_2_numeric_cite.tex");
  assert!(
    !x.contains("ltx_bib_author-year"),
    "revtex4-2 rendered an author-year bibliography instead of numeric:\n{x}"
  );
  assert!(
    x.contains("ltx_bib_key"),
    "revtex4-2 bibliography is missing the numeric `[N]` refnum:\n{x}"
  );
}

/// The `author-year` class option (revtex4-1.cls L6001, revtex4-2.cls L6022) is
/// the option set that overrides the numeric default: `\documentclass[author-year]`
/// must render author-year. Guards that the numeric preload is a DEFAULT the
/// class option can still flip — not a hardcoded force-numbers. (`\setcitestyle`
/// / `\bibpunct` overrides are verified against Perl separately.)
#[test]
fn cluster_bib_revtex4_author_year_option_is_honored() {
  let core = convert_to_xml("tests/cluster_regressions/revtex4_authoryear_cite.tex");
  assert!(
    core.contains(r#"citestyle="authoryear""#),
    "\\documentclass[author-year]{{revtex4-1}} must render author-year:\n{core}"
  );
}

/// The `<tag role="refnum">` of the reference-list `<bibitem>` carrying the given
/// `key`, from core XML.
fn bbl_refnum(xml: &str, key: &str) -> String {
  let needle = format!("key=\"{key}\"");
  for chunk in xml.split("<bibitem").skip(1) {
    if !chunk[..chunk.find('>').unwrap_or(chunk.len())].contains(&needle) {
      continue;
    }
    let entry = &chunk[..chunk.find("</bibitem>").unwrap_or(chunk.len())];
    let open = entry
      .find(r#"<tag role="refnum">"#)
      .unwrap_or_else(|| panic!("no refnum tag for {key}:\n{entry}"))
      + r#"<tag role="refnum">"#.len();
    let close = entry[open..].find("</tag>").unwrap() + open;
    return entry[open..close].to_string();
  }
  panic!("no bibitem with key {key}:\n{xml}");
}

/// html_feedback#4295 (arXiv:2410.05202, revtex4-2 / apsrev4-2 numeric): a
/// numeric-mode natbib `.bbl` (the pre-formatted `thebibliography` path, NOT the
/// `.bib`/MakeBibliography path guarded by `cluster_bib_revtex4_family_numeric_by_default`)
/// must label its References with the entry NUMBER `[N]` — matching the inline
/// `[N]` cites and pdflatex+bibtex — even for entries whose `\bibitem[{Name(Year)}]`
/// label carries an author AND a year.
///
/// SURPASS-PERL (OXIDIZED_DESIGN #126): Perl's `\NAT@wrout` (`natbib.sty.ltxml`
/// L612-613) only forces number style on an EMPTY author/year — its `$style eq
/// 'number'` guard never matches the plural `CITE_STYLE` `'numbers'`/`'super'` —
/// so a numeric `.bbl` entry with a year keeps an author-year label. Verified
/// same-host: Perl 0.8.8 emits `Shor [1994]` (numbers) / `Shor 1994` (super)
/// identically, so matching the PDF is a deliberate divergence. The empty-year
/// entry already numbered `[N]` in both engines (KNOWN_PERL_ERRORS #86 neighbour).
#[test]
fn cluster_bib_natbib_bbl_numeric_refnum() {
  // numbers mode: BOTH entries are the bracketed entry number, never author-year.
  let x = convert_to_xml("tests/cluster_regressions/natbib_bbl_numeric_refnum.tex");
  assert_eq!(
    bbl_refnum(&x, "shor_1994"),
    "[1]",
    "numbers-mode .bbl refnum must be the entry number, not an author-year label:\n{x}"
  );
  assert_eq!(
    bbl_refnum(&x, "gidney_2021"),
    "[2]",
    "empty-year numbers-mode entry stays the entry number:\n{x}"
  );

  // super mode: bare number (empty CITE_OPEN/CLOSE), never `Shor 1994`.
  let x = convert_to_xml("tests/cluster_regressions/natbib_bbl_super_refnum.tex");
  assert_eq!(
    bbl_refnum(&x, "shor_1994"),
    "1",
    "super-mode .bbl refnum must be the bare entry number:\n{x}"
  );

  // Control: author-year mode KEEPS the author-year label (fix is scoped).
  let x = convert_to_xml("tests/cluster_regressions/natbib_bbl_authoryear_refnum.tex");
  assert_eq!(
    bbl_refnum(&x, "shor_1994"),
    "Shor (1994)",
    "author-year mode must keep its author-year label:\n{x}"
  );
  assert_eq!(
    bbl_refnum(&x, "gidney_2021"),
    "(2)",
    "author-year mode empty-year entry falls back to the number:\n{x}"
  );
}

// ===========================================================================
// Citation-ORDER numbering for unsorted bibliography styles (html_feedback
// #6294, IEEE `\bibliographystyle{IEEEtran}`; sibling #5930).
//
// bibtex's own guarantee: an UNSORTED `.bst` (`unsrt`, `unsrtnat`, `ieeetr`,
// `IEEEtran`) numbers the References in the order entries are first cited, NOT
// alphabetically. LaTeXML's MakeBibliography always `unisort`s — so a paper
// whose figure captions hard-code "[2], [3], [4]" (baked into the PDF) gets a
// mismatched, alphabetized list. Both Perl AND oxide alphabetize here, so this
// is a deliberate surpass-Perl: oxide honors the `sort='false'` flag
// (OXIDIZED_DESIGN divergence). Perl even maps `IEEEtran`→`sort='true'`
// (IEEEtran.cls.ltxml L331), itself unfaithful to `IEEEtran.bst`; oxide maps
// `IEEEtran`/`ieeetr`→`sort='false'` to match the real `.bst` and the PDF.
//
// Witness arXiv 2510.05438 (`\documentclass{IEEEtran}`,
// `\bibliographystyle{IEEEtran}`, `\bibliography{...}`, 17 entries).
// ===========================================================================

/// `(refnum, key)` of each reference-list `<bibitem>`, in list order, from
/// post-processed ltx XML — enough to prove WHICH bib key received `[1]`.
fn reference_list_order(xml: &str) -> Vec<(u32, String)> {
  let mut out = Vec::new();
  let mut rest = xml;
  while let Some(s) = rest.find("<bibitem") {
    let after = &rest[s..];
    let end = after.find("</bibitem>").unwrap_or(after.len());
    let seg = &after[..end];
    let key = seg
      .find("key=\"")
      .map(|i| {
        let tail = &seg[i + "key=\"".len()..];
        tail[..tail.find('"').unwrap_or(tail.len())].to_string()
      })
      .unwrap_or_default();
    // Content of the `role="refnum"` tag (the `>` after `role="refnum"` closes
    // the opening `<tag …>`; attributes never contain `>`, so this is robust to
    // attribute order).
    let refnum = seg
      .find("role=\"refnum\"")
      .and_then(|i| seg[i..].find('>').map(|j| &seg[i + j + 1..]))
      .and_then(|tail| tail.find('<').map(|k| &tail[..k]))
      .and_then(numeric_label);
    if let Some(n) = refnum {
      out.push((n, key));
    }
    rest = &after[end..];
  }
  out
}

/// html_feedback #6294 — `\bibliographystyle{unsrt}` and `{IEEEtran}` number the
/// References list by CITATION ORDER, and the inline `[N]` cites index that
/// order. Body cites in the order gamma, alpha, beta, so the fixed list is
/// gamma=[1], alpha=[2], beta=[3] (alphabetical would be alpha/beta/gamma).
#[test]
fn cluster_bib_unsrt_citation_order() {
  for tex in ["cite_order_unsrt.tex", "cite_order_ieeetran.tex"] {
    let x = convert_and_post_clean(&format!("tests/cluster_regressions/{tex}"));
    // Inline cites in reading order (gamma, alpha, beta) read [1], [2], [3].
    let nums: Vec<u32> = inline_cite_texts(&x)
      .iter()
      .filter_map(|c| numeric_label(c))
      .collect();
    assert_eq!(
      nums,
      vec![1, 2, 3],
      "{tex}: inline citations must follow citation order (gamma=1, alpha=2, \
       beta=3), got {nums:?}\n{x}"
    );
    // The References list itself is ordered gamma, alpha, beta — the first-cited
    // entry (Gamma) is [1] and appears first.
    let order = reference_list_order(&x);
    assert_eq!(
      order.len(),
      3,
      "{tex}: expected 3 reference entries, got {order:?}\n{x}"
    );
    assert_eq!(
      order,
      vec![
        (1, "gamma".to_string()),
        (2, "alpha".to_string()),
        (3, "beta".to_string()),
      ],
      "{tex}: the References list must be numbered by citation order \
       (gamma=[1], alpha=[2], beta=[3]), got {order:?}\n{x}"
    );
  }
}

/// The engine half of the fix: `\bibliographystyle{IEEEtran}` (and `{unsrt}`)
/// reach the core `<ltx:bibliography>` as `bibstyle="…"` + `citestyle="numbers"`
/// — the signals the post-processor keys on. `IEEEtran` in particular must be
/// KNOWN (it maps to numeric+unsorted); before the fix it was an "unknown
/// bibstyle". Core output carries no `sort` attribute (byte-identical to Perl,
/// which never emits it on the main node — the post derives order from bibstyle).
#[test]
fn cluster_bib_bibstyle_is_known_numeric() {
  for (tex, bibstyle) in [
    ("cite_order_unsrt.tex", "unsrt"),
    ("cite_order_ieeetran.tex", "IEEEtran"),
    ("cite_order_plain.tex", "plain"),
  ] {
    let core = convert_to_xml(&format!("tests/cluster_regressions/{tex}"));
    assert!(
      core.contains(&format!(r#"bibstyle="{bibstyle}""#)),
      "{tex}: expected bibstyle=\"{bibstyle}\" on ltx:bibliography:\n{core}"
    );
    assert!(
      core.contains(r#"citestyle="numbers""#),
      "{tex}: expected citestyle=\"numbers\":\n{core}"
    );
  }
}

/// Control: a SORTED style (`plain`) must stay ALPHABETICAL — the citation-order
/// path must not leak into the default. Body cites gamma, alpha, beta, so the
/// list stays alpha=[1], beta=[2], gamma=[3] and the inline cites read
/// [3], [1], [2].
#[test]
fn cluster_bib_plain_stays_alphabetical() {
  let x = convert_and_post_clean("tests/cluster_regressions/cite_order_plain.tex");
  let nums: Vec<u32> = inline_cite_texts(&x)
    .iter()
    .filter_map(|c| numeric_label(c))
    .collect();
  assert_eq!(
    nums,
    vec![3, 1, 2],
    "plain must stay alphabetical (alpha=1, beta=2, gamma=3), so inline cites \
     gamma/alpha/beta read [3],[1],[2], got {nums:?}\n{x}"
  );
  let order = reference_list_order(&x);
  assert_eq!(
    order,
    vec![
      (1, "alpha".to_string()),
      (2, "beta".to_string()),
      (3, "gamma".to_string()),
    ],
    "plain: the References list must stay ALPHABETICAL (alpha=[1], beta=[2], \
     gamma=[3]), got {order:?}\n{x}"
  );
}

// ===========================================================================
// Citation-ORDER numbering reaches the natbib/revtex arm (html_feedback
// #5930/#6095): revtex4-2 / natbib papers with `\bibliographystyle{ieeetr}`.
//
// natbib drops the bibstyle name at `\bibstyle` — its `[numbers]`/`nobibstyle`
// option does `\let\bibstyle\@gobble`, and its author-year `\bibstyle` has no
// `\bibstyle@ieeetr` preset — so the #6294 fix (which reads the bibstyle name)
// saw nothing. The kernel `\bibliographystyle` now records the name BEFORE
// dispatching, so an unsorted NUMERIC natbib/revtex list is numbered in citation
// order too. Author-year natbib lists stay alphabetical (the citestyle guard).
// Witness arXiv 2602.00643 (revtex4-2 + ieeetr; oxide now matches bibtex .bbl
// key-for-key across all 10 entries).
// ===========================================================================

/// The bib `key`s of the reference-list `<bibitem>`s in list order (works for
/// numeric AND author-year lists, unlike `reference_list_order`).
fn reference_list_keys(xml: &str) -> Vec<String> {
  let mut out = Vec::new();
  let mut rest = xml;
  while let Some(s) = rest.find("<bibitem") {
    let after = &rest[s..];
    let end = after.find('>').unwrap_or(after.len());
    let tag = &after[..end];
    if let Some(i) = tag.find("key=\"") {
      let tail = &tag[i + "key=\"".len()..];
      out.push(tail[..tail.find('"').unwrap_or(tail.len())].to_string());
    }
    rest = &after[end..];
  }
  out
}

/// html_feedback #5930/#6095 — `[numbers]natbib` (and revtex4-2) with an unsorted
/// numeric style numbers the References by citation order. Body cites
/// gamma/alpha/beta, so the fixed list is gamma=[1], alpha=[2], beta=[3].
#[test]
fn cluster_bib_natbib_numeric_citation_order() {
  let x = convert_and_post_clean("tests/cluster_regressions/natbib_num_ieeetr.tex");
  let nums: Vec<u32> = inline_cite_texts(&x)
    .iter()
    .filter_map(|c| numeric_label(c))
    .collect();
  assert_eq!(
    nums,
    vec![1, 2, 3],
    "[numbers]natbib+ieeetr inline cites must follow citation order: {nums:?}\n{x}"
  );
  assert_eq!(
    reference_list_order(&x),
    vec![
      (1, "gamma".to_string()),
      (2, "alpha".to_string()),
      (3, "beta".to_string()),
    ],
    "[numbers]natbib+ieeetr References must be numbered by citation order\n{x}"
  );
}

/// Control 1: `[numbers]natbib + plainnat` is numeric but SORTED, so it stays
/// ALPHABETICAL — the citation-order path is gated on an UNSORTED `.bst`, not on
/// natbib-numeric alone.
#[test]
fn cluster_bib_natbib_numeric_sorted_stays_alphabetical() {
  let x = convert_and_post_clean("tests/cluster_regressions/natbib_num_plainnat.tex");
  assert_eq!(
    reference_list_keys(&x),
    vec!["alpha", "beta", "gamma"],
    "[numbers]natbib+plainnat (sorted) must stay alphabetical\n{x}"
  );
}

/// Control 2: default natbib (AUTHOR-YEAR) + ieeetr must NOT be reordered — an
/// author-year list is always alphabetical by author regardless of the `.bst`
/// sort flag (the `citestyle="numbers"` guard). Guards against the natbib
/// bibstyle-recording leaking citation-order into author-year mode.
#[test]
fn cluster_bib_natbib_authoryear_stays_alphabetical() {
  let x = convert_and_post_clean("tests/cluster_regressions/natbib_ay_ieeetr.tex");
  assert_eq!(
    reference_list_keys(&x),
    vec!["alpha", "beta", "gamma"],
    "author-year natbib must stay alphabetical, not citation-ordered\n{x}"
  );
}
