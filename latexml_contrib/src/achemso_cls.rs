//! Binding for achemso.cls (ACS chemistry journals), on OmniBus.
//!
//! achemso.cls is an article derivative for ACS journals. Its authorship commands (`\author`, `\affiliation`,
//! `\alsoaffiliation`, `\altaffiliation`, `\email`, `\phone`, `\fax`) give each author its pieces as the class
//! numbers them (OXIDIZED_DESIGN_DIVERGENCES #1065).
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  // achemso.cls (TeX Live's) loads no amsmath: amsmath's commands for papers that use them unloaded, its "Old form"
  // error once the paper loads it; the paper's copy decides (`amsmath_sty::require_amsmath_beyond_original`,
  // KNOWN_PERL_ERRORS #993).
  amsmath_sty::require_amsmath_beyond_original(
    latexml_package::shipped_class::shipped_copy_loads("cls", "amsmath")?,
    &[],
  )?;
  RequirePackage!("amsthm");
  // Eager xcolor preload removed for Perl parity: it makes a later document
  // xcolor[table] load a no-op, so colortbl/array never load and array m{}/b{}
  // columns break (Unrecognized tabular template -> Extra alignment tab). The
  // document loads xcolor itself; color/definecolor stay via hyperref->color.
  // See ifacconf_cls.rs and SYNC_STATUS (eager-xcolor cluster).
  RequirePackage!("hyperref");
  // achemso.cls:1396-1398 `\RequirePackage[sort&compress,numbers,super]{natbib}` (`round` for a non-superscript
  // journal): a citation's numbers sorted and compressed ("1–3"; natbib_sty.rs, KNOWN_PERL_ERRORS #1072). The
  // numeric styles are not passed: the binding's citations stay as before.
  RequirePackage!("natbib", options => vec![s!("sort&compress")]);
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
  // achemso.cls:35 xkeyval and :309-315 the rest of the class's package list, caption, graphicx and url: documents
  // use `\captionsetup`/`\captionof` without loading caption (witness 2202.05883).
  RequirePackage!("xkeyval");
  RequirePackage!("caption");
  RequirePackage!("graphicx");
  RequirePackage!("url");
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

  // achemso.cls:331-539: each `\author` adds an author, who waits for the next affiliation the class numbers
  // (`\@author@affil@<n>`, :340-343); an `\affiliation` is every waiting author's, a repeated text the earlier number
  // (`\acs@affil@swap`, :359-427), so one right after another reaches no author; `\alsoaffiliation` is the last
  // author's, a new one also the waiting authors' (:428-458); `\altaffiliation` the last author's, a second one
  // replacing the first (`\xdef`, :468-485: it is given at the next `\author`);
  // `\email`, `\phone`, `\fax` the last author's (:513-539). The class's numbering hands each piece to the API: the
  // waiting ones `annotate=new` (the last n where one of them holds an `\alsoaffiliation` already), the last one
  // `annotate=1`. A piece no author holds at `\begin{document}` is kept unlinked, as the class lists it with a mark no
  // name carries (OXIDIZED_DESIGN #1065). Witnesses 2605.10968, 2306.05209, 2107.00078, 2512.17659, 2510.00865.
  RawTeX!(
    r"\newcount\acs@author@cnt\newcount\acs@affil@cnt\newcount\lx@acs@waiting\def\lx@acs@last{}\newif\iflx@acs@held
\newcount\acs@affil@alt@cnt\def\lx@acs@alt{}\newcount\lx@acs@alt@n\newif\iflx@acs@body
\def\lx@acs@orphans{}\def\lx@acs@altorphans{}
\def\acs@affil@ifdup#1{\begingroup\def\@tempa{#1}\@tempswafalse\@tempcnta\z@\relax
  \acs@affil@ifdup@aux\if@tempswa\aftergroup\@firstoftwo\else\aftergroup\@secondoftwo\fi\endgroup}
\def\acs@affil@ifdup@aux{\advance\@tempcnta\@ne\relax
  \expandafter\expandafter\expandafter\def\expandafter\expandafter\expandafter\@tempb\expandafter\expandafter
    \expandafter{\csname @address@\@roman\@tempcnta\endcsname}%
  \ifx\@tempa\@tempb\expandafter\@tempswatrue\else\ifnum\@tempcnta<\acs@affil@cnt\relax
    \expandafter\expandafter\expandafter\acs@affil@ifdup@aux\fi\fi}
\def\acs@alsoaffil@find#1{\begingroup\def\@tempa{#1}\@tempcnta\z@\relax\acs@affil@ifdup@aux
  \expandafter\endgroup\expandafter\@tempcnta\the\@tempcnta\relax}
\def\lx@acs@number#1{\acs@affil@ifdup{#1}{\acs@alsoaffil@find{#1}\@tempswafalse}{%
  \global\advance\acs@affil@cnt\@ne\expandafter\gdef\csname @address@\@roman\acs@affil@cnt\endcsname{#1}%
  \@tempcnta\acs@affil@cnt\@tempswatrue}}
\def\lx@acs@add#1#2{\xdef\lx@acs@last{\lx@acs@last,\the\@tempcnta,}%
  \expandafter\gdef\csname lx@acs@given@\the\@tempcnta\endcsname{}%
  \edef\lx@acs@call{\noexpand\lx@add@affiliation[annotate=#1]}\lx@acs@call{#2}}
\def\lx@acs@waiting@add#1{\edef\lx@acs@call{\iflx@acs@held\the\lx@acs@waiting\else new\fi}%
  \global\lx@acs@waiting\z@\global\lx@acs@heldfalse\expandafter\lx@acs@add\expandafter{\lx@acs@call}{#1}}
\def\lx@acs@last@add#1{\edef\lx@acs@call{\noexpand\in@{,\the\@tempcnta,}{\lx@acs@last}}\lx@acs@call
  \ifin@\else\ifnum\lx@acs@waiting>\z@\global\lx@acs@heldtrue\fi\lx@acs@add{1}{#1}\fi}
\def\lx@acs@noauthor#1#2#3{\ClassWarning{achemso}{#1 with no author}\@add@frontmatter{ltx:note}[role=#2]{#3}}
\def\lx@acs@affiliation#1{\ifnum\acs@author@cnt>\z@\lx@acs@number{#1}%
    \ifnum\lx@acs@waiting>\z@\lx@acs@waiting@add{#1}\else\if@tempswa\iflx@acs@body\lx@acs@note{#1}{affiliation}%
      \else\xdef\lx@acs@orphans{\lx@acs@orphans,\the\@tempcnta}\fi\fi\fi
  \else\lx@acs@noauthor{Affiliation}{affiliation}{#1}\fi}
\def\lx@acs@alsoaffiliation#1{\ifnum\acs@author@cnt>\z@\lx@acs@number{#1}%
    \let\lx@acs@reach\lx@acs@last@add\if@tempswa\ifnum\lx@acs@waiting>\z@\let\lx@acs@reach\lx@acs@waiting@add\fi\fi
    \lx@acs@reach{#1}%
  \else\lx@acs@noauthor{Affiliation}{affiliation}{#1}\fi}
\def\acs@altaffil@ifdup#1{\def\@tempa{#1}\@tempswafalse\@tempcnta\z@\relax
  \ifnum\acs@affil@alt@cnt>\z@\relax\expandafter\acs@altaffil@ifdup@aux\fi
  \if@tempswa\expandafter\@firstoftwo\else\expandafter\@secondoftwo\fi}
\def\acs@altaffil@ifdup@aux{\advance\@tempcnta\@ne\relax
  \expandafter\expandafter\expandafter\def\expandafter\expandafter\expandafter\@tempb\expandafter\expandafter
    \expandafter{\csname @altaffil@\@roman\@tempcnta\endcsname}%
  \ifx\@tempa\@tempb\expandafter\@tempswatrue\else\ifnum\@tempcnta<\acs@affil@alt@cnt\relax
    \expandafter\expandafter\expandafter\acs@altaffil@ifdup@aux\fi\fi}
\def\lx@acs@altaffiliation#1{\ifnum\acs@author@cnt>\z@
    \ifx\lx@acs@alt\@empty\else\xdef\lx@acs@altorphans{\lx@acs@altorphans,\the\lx@acs@alt@n}\fi
    \begingroup\acs@altaffil@ifdup{#1}{\global\lx@acs@alt@n\@tempcnta}{\global\advance\acs@affil@alt@cnt\@ne
      \expandafter\gdef\csname @altaffil@\@roman\acs@affil@alt@cnt\endcsname{#1}\global\lx@acs@alt@n\acs@affil@alt@cnt}%
    \endgroup
    \gdef\lx@acs@alt{#1}\iflx@acs@body\lx@acs@alt@flush\fi
  \else\lx@acs@noauthor{Affiliation}{altaffiliation}{#1}\fi}
\def\lx@acs@alt@flush{\ifx\lx@acs@alt\@empty\else\expandafter\gdef\csname lx@acs@altgiven@\the\lx@acs@alt@n\endcsname{}%
  \expandafter\lx@add@altaffiliation\expandafter{\lx@acs@alt}\global\let\lx@acs@alt\@empty\fi}
\def\lx@acs@note#1#2{\@add@frontmatter{ltx:note}[role=#2]{#1}}
\def\lx@acs@leftover#1#2#3#4{\@for\lx@acs@n:=#1\do{\ifx\lx@acs@n\@empty\else
  \expandafter\ifx\csname #2\lx@acs@n\endcsname\relax\@tempcnta\lx@acs@n\relax\expandafter\expandafter\expandafter
    \lx@acs@note\expandafter\expandafter\expandafter{\csname #3\@roman\@tempcnta\endcsname}{#4}\fi\fi}}
\AtBeginDocument{\lx@acs@alt@flush\global\lx@acs@bodytrue
  \lx@acs@leftover\lx@acs@orphans{lx@acs@given@}{@address@}{affiliation}%
  \lx@acs@leftover\lx@acs@altorphans{lx@acs@altgiven@}{@altaffil@}{altaffiliation}}
\def\lx@acs@contact#1#2#3#4{\ifnum\acs@author@cnt>\z@#3{#4}\else\lx@acs@noauthor{#1}{#2}{#4}\fi}"
  );
  DefMacro!(
    "\\author{}",
    "\\lx@acs@alt@flush\\global\\advance\\acs@author@cnt\\@ne\\global\\advance\\lx@acs@waiting\\@ne\\gdef\\lx@acs@last{}\\lx@add@authors@append{#1}"
  );
  DefMacro!("\\affiliation[]{}", "\\lx@acs@affiliation{#2}");
  DefMacro!("\\alsoaffiliation[]{}", "\\lx@acs@alsoaffiliation{#2}");
  DefMacro!("\\altaffiliation{}", "\\lx@acs@altaffiliation{#1}");
  // (labelled as the class lists them, `\MakeUppercase#1: `, :998)
  DefMacro!(
    "\\phone{}",
    "\\lx@acs@contact{Phone number}{phone}{\\lx@add@contact[role=phone,name={Phone:~}]}{#1}"
  );
  DefMacro!(
    "\\fax{}",
    "\\lx@acs@contact{Fax number}{fax}{\\lx@add@contact[role=fax,name={Fax:~}]}{#1}"
  );
  // `\email` stores the address for the author's footnote (`\@email@<n>`, :513-520): an address, read as the other
  // classes' are (`\lx@address@textfont`, revtex4_support_sty.rs), so `ranojoy_bose@apple.com` is no subscript
  // ("Script _ can only appear in math mode", 1803.03591; OXIDIZED_DESIGN #551). Guard
  // `perfect_kernel_batch65m::file_arguments_are_not_typeset`.
  DefMacro!(
    "\\email Semiverbatim",
    "\\lx@acs@contact{E-mail}{email}{\\lx@add@email}{\\lx@address@textfont{#1}}"
  );
  // achemso.cls:1115-1124: `suppinfo` is an environment that opens an unnumbered "Supporting Information Available"
  // section (`\acs@section*{\suppinfoname}`), not a one-argument frontmatter command — which took the `\begin` of
  // the environment's first list as its argument (2609.03387, 2609.10829).
  DefMacro!("\\suppinfoname", "Supporting Information Available");
  DefMacro!("\\suppinfo", "\\section*{\\suppinfoname}");
  DefMacro!("\\endsuppinfo", "");
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
  // achemso.cls:1091-1102 `\SectionsOff`/`\SectionsOn` and :1160-1169 `\AbstractOff`/`\AbstractOn` switch the
  // section headings and the abstract per manuscript type (preamble only); the content is kept either way, so
  // both are no-ops here. :1266-1269 the TOC graphic's box size, which documents set (`{tocentry}` is skipped).
  // Witnesses 1710.02414 (`\AbstractOn`), 2006.15922 and 2112.07721 (`\acs@tocentry@height`).
  DefMacro!("\\SectionsOn", None);
  DefMacro!("\\SectionsOff", None);
  DefMacro!("\\AbstractOn", None);
  DefMacro!("\\AbstractOff", None);
  RawTeX!(
    r"\newlength{\acs@tocentry@height}\newlength{\acs@tocentry@width}
\setlength{\acs@tocentry@height}{1.75in}\setlength{\acs@tocentry@width}{3.25in}"
  );
  DefMacro!("\\latin{}", "#1");
});
