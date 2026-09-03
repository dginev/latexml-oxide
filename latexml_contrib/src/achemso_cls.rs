//! Stub for achemso.cls (ACS chemistry journals).
//!
//! achemso.cls is an article-derivative for ACS journals. Provides
//! authorship/affiliation primitives (\affiliation, \alsoaffiliation,
//! \altaffiliation, \email, \phone, \fax). Gobble for now since we
//! don't render ACS-style title pages.
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  RequirePackage!("amsthm");
  // Eager xcolor preload removed for Perl parity: it makes a later document
  // xcolor[table] load a no-op, so colortbl/array never load and array m{}/b{}
  // columns break (Unrecognized tabular template -> Extra alignment tab). The
  // document loads xcolor itself; color/definecolor stay via hyperref->color.
  // See ifacconf_cls.rs and SYNC_STATUS (eager-xcolor cluster).
  RequirePackage!("hyperref");
  RequirePackage!("natbib");
  // achemso uses setspace internally; user papers also call
  // \singlespacing / \doublespacing in preambles. Witness 2503.21357.
  RequirePackage!("setspace");
  // achemso.cls L308: `\RequirePackage[margin=2.54cm]{geometry}` and L1306
  // calls `\geometry{...}` for its own layout. The real class (which Perl
  // raw-loads) thus has `\geometry` defined; our OmniBus stub must mirror
  // that so authors' preamble `\geometry{...}` resolves. Layout itself is
  // moot in the XML/HTML paradigm, but the CS must exist. Witness 2407.02650
  // (`\geometry{voffset=10pt,...}` → undefined without this; Perl: 0 errors).
  RequirePackage!("geometry");
  // achemso.cls:311 `\RequirePackage{...,float,...}` and :1022-1030 — the
  // class's own float types; without them `\begin{scheme}` is undefined and
  // its `\caption` cascades into "outside any known float" (achemso-demo;
  // Perl raw-loads the class). Guard:
  // `perfect_kernel_batch54::achemso_declares_its_scheme_floats`.
  RequirePackage!("float");
  // achemso.cls:1413-1414 loads mciteplus when installed (`\mciteSubRef`).
  RequirePackage!("mciteplus");
  // achemso.cls:144-165 (its embedded notes2bib): `\bibnote[label]{text}` files
  // the text as a bibliography entry `Note-N` and cites it; the bibliography
  // is bibtex's, so the note is rendered here as a numbered note in place
  // (`\lx@note{bibnote}`), the mark/text halves as the footnote pair;
  // `\printbibnotes` is the .bbl step (achemso-demo). Guard:
  // `perfect_kernel_batch54::achemso_bibnote_is_a_numbered_note`.
  RawTeX!(
    r"\newcounter{bibnote}\def\thebibnote{Note-\the\value{bibnote}}
\newcommand*\bibnote[2][\thebibnote]{\stepcounter{bibnote}\lx@note{bibnote}[\thebibnote]{#2}}
\newcommand*\bibnotemark[1][\thebibnote]{\stepcounter{bibnote}\lx@notemark{bibnote}[\thebibnote]}
\newcommand*\bibnotetext[2][\thebibnote]{\lx@notetext{bibnote}[\thebibnote]{#2}}
\newcommand*\printbibnotes{}\def\bibnotetyperefname{note}\def\ext@bibnote{}"
  );
  RawTeX!(
    r"\newfloat{scheme}{htbp}{los}\floatname{scheme}{Scheme}
\newfloat{chart}{htbp}{loc}\floatname{chart}{Chart}
\newfloat{graph}{htbp}{loh}\floatname{graph}{Graph}
\newcommand*\schemename{Scheme}\newcommand*\chartname{Chart}\newcommand*\graphname{Graph}"
  );

  // ACS authorship primitives — preserve author content as ltx:note
  // frontmatter entries.
  DefMacro!(
    "\\affiliation[]{}",
    "\\@add@frontmatter{ltx:note}[role=affiliation]{#2}"
  );
  DefMacro!(
    "\\alsoaffiliation[]{}",
    "\\@add@frontmatter{ltx:note}[role=affiliation]{#2}"
  );
  DefMacro!(
    "\\altaffiliation[]{}",
    "\\@add@frontmatter{ltx:note}[role=affiliation]{#2}"
  );
  DefMacro!("\\email{}", "\\@add@frontmatter{ltx:note}[role=email]{#1}");
  DefMacro!("\\phone{}", "\\@add@frontmatter{ltx:note}[role=phone]{#1}");
  DefMacro!("\\fax{}", "\\@add@frontmatter{ltx:note}[role=fax]{#1}");
  DefMacro!(
    "\\suppinfo{}",
    "\\@add@frontmatter{ltx:note}[role=suppinfo]{#1}"
  );
  DefMacro!(
    "\\manuscript{}",
    "\\@add@frontmatter{ltx:note}[role=manuscript]{#1}"
  );
  DefMacro!(
    "\\abbreviations{}",
    "\\@add@frontmatter{ltx:note}[role=abbreviations]{#1}"
  );
  // \acsAuthorList — emit the author-list text inline (no frontmatter slot).
  DefMacro!("\\acsAuthorList{}", "#1");
  DefMacro!(
    "\\notetext{}",
    "\\@add@frontmatter{ltx:note}[role=notetext]{#1}"
  );
  // \acsSection — section opener with text becoming heading.
  DefMacro!("\\acsSection{}", "\\section*{#1}");

  // {tocentry} environment — the journal's table-of-contents graphic, which
  // does not belong in the body.
  //
  // It must NOT be suppressed with `\iffalse`…`\fi`. Conditional skipping
  // matches `\fi` by MEANING and expands nothing on the way (TeX's rule, and
  // ours — `skip_conditional_body` in `conditional.rs`), so an
  // `\end{tocentry}` whose macro *body* is `\fi` is invisible to it: the skip
  // ran to end of file and swallowed the rest of the paper, `\bibliography`
  // included. That one line cost 42 of the 342 residual papers in the
  // 2605+2606 bibliography-absence cohort their entire bibliography, and
  // showed up as `Error:expected:\fi \iffalse` pointing at EOF — which reads
  // like a source defect and is not one. Witnesses 2606.14933, 2605.00451,
  // 2606.00264; audit family F6.
  //
  // Perl never reaches this: it has no achemso binding at all and falls back
  // to OmniBus (`Warning:missing_file:achemso`), leaving `{tocentry}`
  // undefined. pdflatex of course renders these papers fine.
  //
  // Nor may the body simply be DIGESTED and dropped (`DefEnvironment!
  // ("{tocentry}", "")`): these graphics are TikZ/`\includegraphics` blocks
  // that error heavily out of context. Measured on the same cohort, that
  // turned three papers from "no bibliography" into no OUTPUT AT ALL —
  // 2606.08929, 2606.12056 and 2606.15422 went from 1 error to 513 and a
  // fatal (`\lxSVG@endscope`, `undefined:\@startsection`).
  //
  // So skip the body as RAW LINES, the way comment.sty's excluded
  // environments do (`comment_sty.rs`) — no digestion, no conditionals. The
  // `\end{tocentry}` is consumed as text, so it is deliberately left
  // undefined.
  DefConstructor!(T_CS!("\\begin{tocentry}"), None, None,
  after_digest => {
    // The rest of the `\begin{tocentry}` line — which may already carry the
    // matching `\end{tocentry}`, as the one-line form does.
    if let Some(first) = read_raw_line()
      && first.contains("\\end{tocentry}")
    {
      return Ok(Vec::new());
    }
    let mut nlines = 0;
    while let Some(line) = read_raw_line() {
      if line.contains("\\end{tocentry}") {
        break;
      }
      nlines += 1;
    }
    note_progress(&s!("[Skipped tocentry ({nlines} lines)]"));
    Ok(Vec::new())
  });

  // {acknowledgement} — ACS-spelt acknowledgement section.
  DefEnvironment!(
    "{acknowledgement}",
    "<ltx:acknowledgements>#body</ltx:acknowledgements>"
  );

  // achemso.cls extras commonly hit in ACS papers. The class L1087 sets
  // a section-numbering policy via `\SectionNumbersOn` (preamble only);
  // HTML rendering inherits LaTeX's default numbering so the toggle is
  // a no-op. L294 `\providecommand{\latin}[1]{#1}` is an identity
  // wrapper for italicized Latin abbreviations. Witness 2312.12737.
  DefMacro!("\\SectionNumbersOn", None);
  DefMacro!("\\SectionNumbersOff", None);
  DefMacro!("\\latin{}", "#1");
});
