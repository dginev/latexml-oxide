//! Stub for sn-jnl.cls (Springer Nature journal class).
use latexml_package::prelude::*;

use crate::shipped_class::{
  class_code, class_code_argument, class_code_contains, shipped_class_text,
};

LoadDefinitions!({
  LoadClass!("OmniBus");
  // sn-jnl.cls loads amsmath in some versions only (140 of 799 copies in a 1/20 arXiv sample; 2509.03559's none):
  // amsmath's commands for papers that use them unloaded, its "Old form" error once the paper loads it; the paper's
  // copy decides (`amsmath_sty::require_amsmath_beyond_original`, KNOWN_PERL_ERRORS #993).
  amsmath_sty::require_amsmath_beyond_original(
    latexml_package::shipped_class::shipped_copy_loads("cls", "amsmath")?,
    &[],
  )?;
  RequirePackage!("amsthm");
  RequirePackage!("amssymb");
  // xcolor is not preloaded here: it comes from the class's own
  // `\usepackage{xcolor}` through the dependency scan below, as in Perl, and a
  // paper's later `\usepackage[table]{xcolor}` still loads colortbl (xcolor
  // keeps its `table` option handler for a repeat load, xcolor_sty.rs), so its
  // array `m{}`/`b{}` columns work. See ifacconf_cls.rs.
  RequirePackage!("hyperref");
  RequirePackage!("graphicx");
  // Real sn-jnl.cls loads geometry for page setup — papers commonly
  // call \\geometry{margin=2cm} without an explicit usepackage.
  // Witness 2503.06846.
  RequirePackage!("geometry");
  // The packages the shipped class names. Perl ships no sn-jnl binding, so
  // OmniBus runs `maybeRequireDependencies` over the raw class
  // (Package.pm:2776-2813): every `\RequirePackage`/`\usepackage`/`\LoadClass`
  // name that has a binding is loaded, with its options. This binding bypasses
  // that fallback, so it runs the same scan itself — booktabs's `\toprule`,
  // wrapfig, listings, appendix, rotating's `{sidewaystable}` (2101.02753),
  // algorithm/algorithmicx/algpseudocode only when this version names them
  // (older copies L308-310, witness 2201.08889: forcing algorithmicx on a newer
  // one defines `\algorithmic`, and the paper's own `\usepackage{algorithmic}`
  // is then refused — algorithmic_sty.rs, Perl algorithmic.sty.ltxml:20-23 — a
  // flood of undefined `\STATE` into `Fatal:TooManyErrors`, 2605.00003,
  // 2605.10685). Witness 2606.00121: `\toprule`/`\midrule`/`\bottomrule`
  // undefined. Not from the scan: article, which OmniBus has loaded — the
  // class's `\LoadClass[twoside,fleqn]{article}` re-load would apply a
  // leftover `fleqn` handler and set every display equation flush left, where
  // pdflatex centres them (the option is article's alone, and amsmath's
  // `\if@fleqn` stays false); and natbib and apacite (chosen by reference style,
  // below). program, which the older copies load (L311, 1812.02207's; L621,
  // 1911.08756's; 106 of 148 sampled copies), comes from the scan as the class
  // has it: its `\gt`, `\lt`, `\union`, `\normalbaroutside` (program.sty:124,
  // 713-722) were undefined in 138 papers of run 353 that use them (2010.12415,
  // 2102.03176, 2104.02781). With it `;` is active in math (program.sty:67-76),
  // as in pdflatex. This port binds program (a raw load) and Perl does not, so
  // Perl's scan skips it.
  require_dependencies_except("sn-jnl", "cls", &["article", "natbib", "apacite"]);
  // program.sty:175-176 also makes `\(`…`\)` a `{programbox}` (a `\textwidth`
  // minipage around a `{program}`), which the papers of these copies write as
  // inline maths and pdflatex sets in the line (2112.05038, on 413 lines: "For a
  // domain Y ⊂ Rn, there exists…"); its raw emulation here made each an
  // inline-block holding a one-item list (1331 in 2112.05038). They stay the
  // kernel's inline maths (OXIDIZED_DESIGN_DIVERGENCES #597, approved 2026-10-15;
  // program loaded under another class keeps the programbox).
  if package_loaded("program") {
    DefMacro!("\\(", "\\lx@begin@inline@math");
    DefMacro!("\\)", "\\lx@end@inline@math");
  }
  // The proof mark: the class's begin-document code (sn-jnl.cls:1571-1575 of 1907.11468's copy, in all 138 copies
  // of the run-353 sn-jnl papers) declares `\opensquare` (msa "03, □) and makes it the `\qedsymbol`, over
  // program.sty:971-973's `\let\qedsymbol=\blacksquare`, so pdflatex ends a proof with □ ("Proof Trivial. □";
  // 1907.11468, 2210.16458). This binding does not run the class body, so it runs these three lines of that hook
  // itself, where the copy has them (or the paper ships none: the class as distributed). The same hook then makes
  // `\proof` its `{spiproof}` (`\let\proof\spiproof\let\endproof\endspiproof`), a copy of amsthm's interface
  // (`[1][\proofname]`, `\pushQED{\qed}`), over program.sty:928's `\def\proof` (a head of its own, "Proof:", which the
  // proof lock lets through, 67ps). A copy is the binding's `<ltx:proof>` (OXIDIZED_DESIGN_DIVERGENCES #596), so the
  // hook's `\let` puts the binding's proof back, where the copy has it.
  // Without program the mark is amsthm's `\qed`, which does not read `\qedsymbol` (amsthm_sty.rs).
  let class_text = shipped_class_text("sn-jnl")?;
  if class_text
    .as_deref()
    .is_none_or(|text| class_code_contains(text, "\\def\\qedsymbol{\\opensquare}"))
  {
    RawTeX!(
      r#"\AtBeginDocument{\DeclareSymbolFont{AMSa}{U}{msa}{m}{n}\DeclareMathSymbol{\opensquare}{\mathord}{AMSa}{"03}\def\qedsymbol{\opensquare}}"#
    );
  }
  // The hook's `\let` runs with or without program: a paper that clears the class's proof in its preamble
  // (`\let\proof\relax\let\endproof\relax` before its own `\usepackage{amsthm}`, a no-op as the class loaded amsthm)
  // has it back at `\begin{document}` (2211.03599, 2501.00469, whose copies load no program; 2009.01631, 2301.05288).
  if class_text
    .as_deref()
    .is_none_or(|text| class_code_contains(text, "\\let\\proof\\spiproof"))
  {
    RawTeX!(r#"\AtBeginDocument{\let\proof\@proof\let\endproof\end@proof}"#);
  }

  // Real sn-jnl.cls loads natbib for EVERY reference style (L1649/1652/1662/
  // 1669/1677: `\usepackage[numbers,sort&compress]{natbib}` for the numeric
  // styles, `\usepackage[authoryear]{natbib}` for the author-year ones).
  // OmniBus only `def_autoload`s natbib off `\citet`/`\citep`/`\citeyear`/…
  // — deliberately NOT off `\cite`, which the kernel already defines. So a
  // paper that cites solely via natbib's TWO-optional `\cite[pre][post]{keys}`
  // never triggers the autoload, and the kernel's single-optional
  // `\cite[] Semiverbatim` reads `[` as the whole key list: the real keys are
  // dropped (never cited, silently absent from the References) and `]{keys}`
  // leaks as body text. Requiring natbib here mirrors the real class and also
  // spares these papers the autoload dance. Options are left at natbib's
  // default (numbers) — same as what the autoload path already produced, so
  // no rendering change for papers that did trigger it; the per-refstyle
  // numbers-vs-authoryear mapping needs class-option processing this stub
  // doesn't do yet. Witness 2605.23484 (sn-mathphys-num), 2606.10002
  // (sn-basic), 2606.10215, 2606.11534.
  RequirePackage!("natbib");

  // sn-jnl.cls reference styles select the bibliography package by class
  // option (sn-jnl.cls L1656-1694): the numeric/author-year styles use natbib
  // (loaded above), but the `sn-apa` style loads apacite —
  // `\if@APA@refstyle \usepackage[natbibapa]{apacite} \fi` (L1683-1685,
  // guarded by `\DeclareOption{sn-apa}{\@APA@refstyletrue}` L111). Without it,
  // an `sn-apa` paper's apacite-formatted `.bbl` (`\APACinsertmetastar`,
  // `\BOthers`, `\BDBL`, `{APACrefauthors}`, `\APACrefYearMonthDay`,
  // `\PrintBackRefs`, …) hits our apacite binding UNLOADED, so every one of
  // those macros floods `Error:undefined:` and the References render as raw
  // macro names. Mirror the class: fire apacite when `sn-apa` is passed.
  // (natbib above stays unconditional — the existing simplification; apacite
  // requires natbib anyway, so loading both under `sn-apa` is harmless.)
  // Perl ships no sn-jnl / apacite binding at all → Rust surpasses.
  // arXiv/html_feedback#1261, witness 2404.15224
  // (`\documentclass[referee,sn-apa,pdflatex,natbib]{sn-jnl}`).
  // With the class's `natbibapa` option, as the class loads it: natbib's own `\cite` (textual without notes), not
  // apacite's classic parenthetical one (apacite_sty.rs, apacite.sty:93-100).
  DeclareOption!("sn-apa", {
    RequirePackage!("apacite", options => vec![s!("natbibapa")]);
  });
  // sn-jnl.cls:67, :102 `\if@pdflatex`, set by the `pdflatex` option
  RawTeX!(r"\newif\if@pdflatex\global\@pdflatexfalse");
  DeclareOption!("pdflatex", {
    RawTeX!(r"\global\@pdflatextrue");
  });
  ProcessOptions!();
  // sn-jnl.cls:1462-1467 `\breakurldefns`: without the `pdflatex` option the class loads breakurl and makes `\href`
  // its `\burlalt` — the first argument shown, the second the link (breakurl.sty:50-51). That `\let` is not made
  // (OXIDIZED_DESIGN_DIVERGENCES #654, a surpass): the authors write hyperref's `\href{url}{text}` (10 of 12 sampled
  // papers: 2405.04967, 2207.12959, 2603.09685), which the swap turns into the URL shown and the text linked; the
  // template's own swapped order (`\href{here.}{https://…/Datasets_-_RMSC/…}`, 2401.05832) is the exception. breakurl
  // is still loaded, as the class does.
  RawTeX!(r"\if@pdflatex\else\RequirePackage[hyphenbreaks]{breakurl}\fi");

  // sn-jnl frontmatter — gobble layout-only / preserve author text.
  DefMacro!("\\bmhead{}", "\\subsubsection*{#1}");
  // sn-jnl.cls:877-878: numbered or unnumbered sections from here on (2609.05015).
  RawTeX!(
    r"\def\numbered{\setcounter{secnumdepth}{3}}\def\unnumbered{\setcounter{secnumdepth}{0}}"
  );
  DefMacro!("\\bmsection{}", "\\section*{#1}");
  // \sectiontitle{text} carries an author-typed section title used in
  // sn-jnl's TOC/running-head pipeline. Preserve as ltx:note rather
  // than silently dropping the words. Content-preserving.
  DefMacro!(
    "\\sectiontitle{}",
    "\\@add@frontmatter{ltx:note}[role=sectiontitle]{#1}"
  );
  // \headtype{...} / \extralength{...} are layout knobs (no author body).
  def_macro_noop("\\headtype{}")?;
  def_macro_noop("\\extralength{}")?;
  // \theHfigure / \theHtable: the kernel's (every counter has its `\theH<counter>`, counter/dialect.rs), which
  // hyperref's `\hyper@makecurrent` reads inside an `\xdef` — not argument-taking stubs, which took its `}`.

  // The author block, as sn-jnl.cls prints it (2412.01641's copy, line numbers of it; the census's 165 copies alike).
  // `\author[labels]{name}` prints the name and its labels in a superscript (:487-503); `\author*` adds a `*` after
  // them, only where it has labels (:505-516), and sends the `\email` after it to the corresponding list (:535-546),
  // which `\@maketitle` prints once, "*Corresponding author(s). E-mail(s): …" (:749-751). An author note
  // (`\equalcont{text}`, :584-586) stores its text and appends its glyph after the names printed so far; `\@maketitle`
  // prints the last text once, under the addresses (:754). Here the labels and the `*` are the creator's
  // `annotations`, the `*` answered by one correspondent note; each name ends with `\lx@sn@marks@<n>`, which the notes
  // written after it fill with their requests; `\maketitle` queues each note once, labelled, so its carriers share it
  // (OD #1059), marked with the glyph the paper's copy prints (2412.01641, 2603.01834, 2312.03687, 2512.07346).
  // (a name list in one `\author` is several people, read as the per-author classes read theirs: Perl, with no sn-jnl
  // binding, splits it with the kernel's `\author`; 2312.04825, 2501.00473, 2504.09158; the notes after it mark the
  // last, as the class prints them after the list). The star is `OptionalMatch:*`: a literal `*` parameter made it
  // mandatory, and a plain `\author{X}` recursed (2306.11901). The labels name the `\affil[n]` contacts below, the
  // labelled-affiliation pattern of Perl PR #2767 (jheppub_sty.rs).
  DefMacro!("\\author OptionalMatch:* []{}", sub[(star, labels, name)] {
    Ok(author_call(star.is_some(), labels, &name))
  });
  // (the `\email` after a starred name is a corresponding address, :535-546, `\if@corauemail`)
  DefMacro!("\\email{}", "\\lx@sn@email\\lx@add@email{#1}");
  DefPrimitive!("\\lx@sn@email", sub[()] {
    if lookup_bool(STARRED) {
      assign_value(CORRESPONDING_EMAIL, true, Scope::Global);
    }
    Ok(Vec::new())
  });
  // `\affil*[N]{text}`: attach as a structured `ltx:contact[role=affiliation]`
  // ON the author creators, not an orphaned top-level `ltx:note`. The numbered
  // form links by id (contact label <-> the author's annotations); the
  // unnumbered shared form (a single `\affil` for all authors — the witness's
  // shape) uses `annotate=all` to attach to every preceding creator.
  // arXiv/html_feedback#534, witness 2204.04741. The starred form's `*` after the label (`^{1*}`, 2412.01641's
  // sn-jnl.cls:561-566), the class's ornament of the corresponding address, links nothing (2603.01834).
  RawTeX!(
    r#"\def\lx@sn@affil#1#2{\def\lx@sn@affil@id{#1}%
\ifx\lx@sn@affil@id\@empty\lx@add@contact[role=affiliation,annotate=all]{#2}%
\else\lx@add@contact[role=affiliation,label={#1}]{#2}\fi}"#
  );
  DefMacro!("\\affil OptionalMatch:* []{}", "\\lx@sn@affil{#2}{#3}");
  DefMacro!(
    "\\presentaddress{}",
    "\\@add@frontmatter{ltx:note}[role=present-address]{#1}"
  );
  // sn-jnl.cls L1788: \gdef\orcid#1{\href{#1}{\orcidlogo}} (ORCID-logo
  // hyperlink, used INLINE inside \author{...\orcid{id}}). Render as an inline
  // ORCID link showing the id (content-preserving: the id stays in the output,
  // both as visible text and the proper orcid.org href; the real cls's logo
  // image Orcidlogo.eps isn't shipped). A frontmatter note does NOT work here —
  // it fires inside the author-name digestion and is dropped. Was undefined →
  // error (Perl defines it). Witness 2211.09693.
  DefMacro!("\\orcid{}", "\\href{https://orcid.org/#1}{#1}");
  // \sep — elsarticle-family keyword separator (`\def\sep{\unskip, }`). sn-jnl
  // papers use it in keyword lists (`kw1 \sep kw2 \sep ...`); the sn-jnl cls
  // doesn't define it but Perl provides it (elsarticle convention). Render as a
  // comma separator. Was undefined → error. Witness 2309.06763.
  DefMacro!("\\sep", "\\unskip, ");
  // Name part helpers (first-name, surname) — emit inline.
  DefMacro!("\\fnm{}", "#1");
  DefMacro!("\\sur{}", "#1");
  // sn-jnl.cls L599-606: \orgdiv / \orgname / \orgaddress / \street /
  // \postcode / \city / \state / \country — affiliation-element helpers
  // that pass through their argument as inline text. The paper-bundled
  // class file defines all of them as `\newcommand{\foo}[1]{#1}` (plain
  // pass-through). Our raw-load path doesn't always invoke them, so
  // bind explicitly. Without these stubs, papers using the standard
  // sn-jnl `\affil*[1]{\orgdiv{...}, \orgname{...}, \orgaddress{...}}`
  // pattern report undefined CS cascade. Witness 2311.09249, 2311.08387.
  DefMacro!("\\orgdiv{}", "#1");
  DefMacro!("\\orgname{}", "#1");
  DefMacro!("\\orgaddress{}", "#1");
  DefMacro!("\\street{}", "#1");
  DefMacro!("\\postcode{}", "#1");
  DefMacro!("\\city{}", "#1");
  DefMacro!("\\state{}", "#1");
  DefMacro!("\\country{}", "#1");
  // sn-jnl.cls defines \botrule as a bottom-rule table separator
  // (similar shape to \toprule / \midrule from booktabs). Authors use
  // it inside \begin{tabular}...\end{tabular} for Springer-Nature
  // bottom rules. Map to \hline so the table still renders.
  // Witness 2402.17342.
  Let!("\\botrule", "\\hline");

  // The class's own commands the papers use, as sn-jnl.cls defines them (1802.08831's copy, line numbers of it unless
  // said; 2006.16202's, 2203.15424's, 2404.03266's, 2603.28334's the same where they have them).
  // :550 `\let\nomail\relax`, written in place of an e-mail (34 papers of the inventory: 2111.06962).
  // :454-461 the title-page fonts and :1231-1235 the table and :1201 (2402.15353's copy) the figure-caption fonts
  // (`\Titlefont` 6 papers, 2210.01472; `\tablebodyfont` 7, 2307.08421; `\figurecaptionfont` 3, 2402.15353), with the
  // `\titraggedcenter` setting they end with (:443-444).
  RawTeX!(
    r"\let\nomail\relax
\def\titraggedcenter{\leftskip=12pt plus 0.5fil\rightskip=12pt plus 0.5fil\parfillskip=0pt\let\hb=\break}
\def\absraggedcenter{\leftskip=24pt plus 0.5fil\rightskip=24pt plus 0.5fil\parfillskip=0pt\let\hb=\break}
\def\Artcatfont{\reset@font\fontsize{8bp}{10bp}\selectfont}
\def\Titlefont{\reset@font\fontsize{17bp}{22.5bp}\selectfont\titraggedcenter}
\def\SubTitlefont{\reset@font\fontsize{14bp}{16.5bp}\selectfont\titraggedcenter}
\def\Authorfont{\reset@font\fontsize{12bp}{14.5bp}\selectfont\boldmath\titraggedcenter}
\def\addressfont{\reset@font\fontsize{11bp}{13.5bp}\selectfont\titraggedcenter}
\def\abstractheadfont{\reset@font\fontsize{9bp}{11bp}\bfseries\selectfont\titraggedcenter}
\def\abstractsubheadfont{\reset@font\fontsize{9bp}{11bp}\bfseries\selectfont}
\def\abstractfont{\reset@font\fontsize{9bp}{11bp}\selectfont\leftskip=24pt\rightskip=24pt}
\def\tablecaptionfont{\reset@font\fontsize{8bp}{9.5bp}\selectfont}
\def\tablebodyfont{\reset@font\fontsize{8bp}{9.5bp}\selectfont}
\def\tablecolheadfont{\reset@font\fontsize{8bp}{9.5bp}\selectfont\bfseries\boldmath}
\def\tablefootnotefont{\reset@font\fontsize{8bp}{9.5bp}\selectfont}
\def\figurecaptionfont{\reset@font\fontfamily{\rmdefault}\fontsize{8}{9.5}\selectfont}"
  );
  // :1280-1292 `{tablenotes}`, a one-item list of the notes under a table, and `\tnote{a}`, a superscript mark (20 and 4
  // papers: 2102.13454, 2108.07431). The table rules: the class as distributed loads booktabs (:307), the copies
  // that do not (2006.16202's, :1292-1309) define its `\toprule`/`\midrule` themselves (13 and 12 papers had them
  // undefined: 2006.16202).
  RawTeX!(
    r"\newenvironment{tablenotes}{\list{}{\setlength{\labelsep}{0pt}\setlength{\labelwidth}{0pt}\setlength{\leftmargin}{0pt}\setlength{\rightmargin}{0pt}\setlength{\topsep}{-6pt}\setlength{\itemsep}{2pt}\setlength{\partopsep}{0pt}\setlength{\listparindent}{0em}\setlength{\parsep}{0pt}}\item\relax}{\endlist}
\def\tnote#1{$^{#1}$}"
  );
  RequirePackage!("booktabs");
  // :1361-1364 `\sidewaystablefn` (74 papers: 1802.08831, 2012.15392, 2102.04394) makes `\footnotetext[a]{text}` in the
  // `{sidewaystable}` print its text in the float under a superscript mark, as the class's renewed `{table}` does in
  // every table (:1341-1349). This binding keeps the kernel's `{table}`, where the note is the kernel's footnote; the
  // sideways one is the same, so `\sidewaystablefn` changes nothing here.
  def_macro_noop("\\sidewaystablefn")?;
  // :631-638 `\artnote{text}` and `\miscnote{text}` (8 and 3 papers: 2203.15424, 2605.14802), a note `\maketitle`
  // prints as the title's footnote (:816-817); 2404.03266's copy :639-646 `\ackno{text}`, acknowledgments the title page
  // prints under the abstract (3 papers).
  DefMacro!(
    "\\artnote{}",
    "\\@add@frontmatter{ltx:note}[role=artnote]{#1}"
  );
  DefMacro!(
    "\\miscnote{}",
    "\\@add@frontmatter{ltx:note}[role=miscnote]{#1}"
  );
  DefMacro!(
    "\\ackno{}",
    "\\@add@frontmatter{ltx:acknowledgements}[name=Acknowledgments]{#1}"
  );
  // The author notes: `\equalcont` (2412.01641's :584-586, every copy) and the further ones some copies add, each
  // marking the names before it and printed under the addresses — `\intern` (2606.24824's :598-601, printed at :770;
  // 2307.08214's :591-595 with `\S`, whose print the copy comments out, :764), `\equalsup` (2603.28334's :600, :769;
  // 2607.26008), `\equaladv` (2501.01157's :598-601, :771), `\equalcontribution` (2206.10381's :590, `\star`). Each
  // stores the text (`\lx@sn@authornote`), which `\maketitle` queues once, with the glyph the copy appends to the names
  // (`note_glyph`; the class as distributed `\equalcont`'s †); the text is kept where the copy does not print it. A copy
  // may take the text as an optional argument whose default is the class's text, which the paper sets with
  // `\interntxt`/`\equalcontxt` (2606.24824's `\newcommand{\intern}[1][\@interntxt]`, its default "Work done as an
  // intern at Westlake University.", the paper's `\intern{}`; 2504.21788's :615-617 `\equalcont`, "These authors
  // contributed equally to this work.", the paper's bare `\equalcont`): there the command reads the copy's form, with
  // the copy's default.
  for &(command, ..) in &AUTHOR_NOTES {
    let glyph = match class_text.as_deref() {
      Some(text) => note_glyph(text, command),
      None if command == "equalcont" => Some(s!("\u{2020}")),
      None => None,
    };
    // (a copy whose command marks no name, `\g@addto@macro\artauthors{}` — 2209.11450's `\equalcont`, its text printed
    // under the addresses alone — or that has none: the document's note)
    assign_value(
      &s!("lx@sn@marked@{command}"),
      glyph.is_some(),
      Scope::Global,
    );
    assign_value(
      &s!("lx@sn@glyph@{command}"),
      Stored::String(pin(glyph.unwrap_or_default())),
      Scope::Global,
    );
  }
  let optional_form = |command: &str, store: &str| {
    class_text
      .as_deref()
      .filter(|text| class_code_contains(text, &s!("\\newcommand{{\\{command}}}[1][\\{store}]")))
  };
  for (command, setter, store) in [
    ("equalcont", "equalcontxt", "@equalconttxt"),
    ("intern", "interntxt", "@interntxt"),
  ] {
    if let Some(text) = optional_form(command, store) {
      let default = class_code_argument(text, &s!("\\{setter}")).unwrap_or_default();
      raw_tex(
        &[
          r"\def\",
          setter,
          r"#1{\gdef\",
          store,
          r"{#1}}\",
          setter,
          "{",
          &default,
          "}",
          r"\def\",
          command,
          r"{\@ifnextchar[{\lx@sn@optnote{",
          command,
          r"}}{\lx@sn@optnote{",
          command,
          r"}[\",
          store,
          "]}}",
        ]
        .concat(),
      )?;
    } else {
      raw_tex(
        &[
          r"\def\",
          command,
          r"#1{\lx@sn@authornote{",
          command,
          "}{#1}}",
        ]
        .concat(),
      )?;
    }
  }
  for command in ["equalsup", "equaladv", "equalcontribution"] {
    raw_tex(
      &[
        r"\def\",
        command,
        r"#1{\lx@sn@authornote{",
        command,
        "}{#1}}",
      ]
      .concat(),
    )?;
  }
  RawTeX!(r"\def\lx@sn@optnote#1[#2]{\lx@sn@authornote{#1}{#2}}");
  DefMacro!("\\lx@sn@authornote{}{}", sub[(command, text)] { Ok(author_note(&command.to_string(), text)) });
  // `\@maketitle`'s notes (:749-756), queued as `\maketitle` begins (`\lx@maketitle@begin`, ahead of the flush, so its
  // relocation makes the copies one note); the corresponding note in the copy's words.
  // (2209.11450's copy prints neither the `*`, `{}` in its `\@@corrauthor`, nor the line, commented out)
  let corresponding = match class_text.as_deref() {
    Some(text) => corresponding_text(text).unwrap_or_default(),
    None => s!("Corresponding author(s)."),
  };
  let star_prints = class_text.as_deref().is_none_or(star_prints);
  assign_value(STAR_PRINTS, star_prints, Scope::Global);
  assign_value(
    CORRESPONDING_TEXT,
    Stored::String(pin(&corresponding)),
    Scope::Global,
  );
  DefMacro!("\\lx@sn@print@notes", sub[()] { Ok(print_notes()) });
  AddToMacro!("\\lx@maketitle@begin", "\\lx@sn@print@notes");
  // The switches the notes set, which papers set themselves (`\equalconttrue`, 2 papers), and
  // `\presentaddresstxt{label}`, the heading of `\presentaddress` (2 papers; 2603.28334's copy :572-576).
  RawTeX!(
    r"\newif\ifequalcont\newif\ifpresentaddress\newif\ifintern\newif\ifequalsup\newif\ifequaladv
\def\presentaddresstxt#1{\gdef\@presentaddresstxt{#1:}}\presentaddresstxt{Present Address}"
  );
  // 2307.08421's copy :491-492: `\Newlabel{key}{value}`, the class's cross-reference record a paper's `.aux` replays
  // (3 papers), and `\jmkLabel` that writes it.
  RawTeX!(
    r"\def\jmkLabel#1{\@bsphack\protected@write\@auxout{}{\string\Newlabel{#1}{\@currentlabel}}\@esphack}
\def\Newlabel#1#2{\expandafter\xdef\csname X@#1\endcsname{#2}}"
  );
  // 2303.09648's copy :1084-1116 `{unenumerate}[widest]`, an enumerate whose items print no label (4 papers).
  RawTeX!(
    r"\def\unenumerate{\@ifnextchar[{\@unenumerate}{\@unenumerate[0.]}}
\def\@unenumerate[#1]{\par\ifnum \@enumdepth >3 \@toodeep\else\advance\@enumdepth \@ne\edef\@enumctr{enum\romannumeral\the\@enumdepth}\list{}{\setlength{\leftmargin}{\csname leftmargin\romannumeral\the\@enumdepth\endcsname}\usecounter{\@enumctr}\settowidth\labelwidth{#1}\def\makelabel##1{\hss\llap{##1}}}\fi}
\let\endunenumerate\endlist"
  );
  // :1529-1547 `{biography}{photo}{text}`: the author's photo beside the opening text, then the body; `\author{name}`
  // inside sets the name bold (5 papers: 2310.14485). The photo is set inline before the text, not wrapped. The
  // kernel's `\author` is locked against a file's redefinition, so the binding makes it the bold name (unlocked, as
  // before-digest code runs, Primitive.pm:44), in the environment's group; a primitive, which adds nothing to the
  // document.
  DefPrimitive!("\\lx@sn@biography@author", None,
  before_digest => {
    DefMacro!("\\author{}", "{\\bfseries #1}");
  });
  RawTeX!(
    r"\newenvironment{biography}[2]{\par\lx@sn@biography@author\noindent#1{#2\par}\par}{\par}"
  );
  // 2512.12101's copy :1796-1802: `\orcidlogo`, the iD logo `\orcid` links (2 papers); the logo image is the class's
  // `Orcidlogo` graphic, here the kernel's iD glyph.
  DefMacro!("\\orcidlogo", "\\lx@orcidlogo");
  // Commands some copies add under common names, defined only where the paper's copy has them, so a paper whose copy
  // does not can define its own (2206.00975's :317-318, 2301.12731, 2407.15443: `\abs`, 3 papers; `\argmin`).
  if let Some(text) = class_text.as_deref() {
    if class_code_contains(
      text,
      "\\newcommand{\\abs}[1]{\\left\\lvert #1 \\right\\rvert}",
    ) {
      RawTeX!(r"\newcommand{\abs}[1]{\left\lvert #1 \right\rvert}");
    }
    if class_code_contains(text, "\\DeclareMathOperator*{\\argmin}{argmin}") {
      RawTeX!(r"\DeclareMathOperator*{\argmin}{argmin}");
    }
  }

  // Do NOT override `\abstract` or the `{abstract}` environment here. The
  // kernel's `\abstract` (latex_constructs `\abstract`, L5153) is `locked`: the
  // brace form routes to the *deferred* `\lx@add@abstract` frontmatter
  // accumulator, so `\maketitle` flushes it in schema-canonical order (document
  // `<title>` first, `<abstract>` after). Perl relies on exactly this lock — the
  // raw sn-jnl.cls's `\long\def\abstract#1{\def\@abstract{...}}` (deferred store,
  // emitted by `\@maketitle`'s `\printabstract` AFTER title+authors) cannot pull
  // the core `\abstract` back to an immediate emit. A binding-level
  // `DefEnvironment!("{abstract}", <ltx:abstract>...)` + `\abstract{}` forwarder
  // bypasses that lock and emits `<ltx:abstract>` at the *call site* — which in
  // every sn-jnl paper is BEFORE `\maketitle`, so the abstract lands ahead of the
  // still-deferred title. That is the abstract-before-title regression; letting
  // the locked kernel macro run fixes it and keeps the env properly bounded
  // (`\begin{abstract}`→`\lx@begin@abstract`, `\end{abstract}`→`\lx@end@abstract`
  // both deferred). arXiv/html_feedback#3436, witnesses 2411.11158, 2306.11901.
  //
  // `{appendices}`: appendix.sty's environment, as the class loads it (sn-jnl.cls:1365 `\usepackage[title]{appendix}`
  // of 2609.33801's copy, in all 164 copies of the census; from the dependency scan above, here where the paper ships
  // none; the class's own copy of appendix.sty's `{appendices}` at :1369-1407 is the appendix binding's), re-dressed
  // where it is loaded (:1367 `\@ifpackageloaded{appendix}`) by the class's begin-document code (:1409-1426): the
  // environment resets the figure, table and equation counters and numbers them by the appendix letter,
  // `\@Alph\c@section\arabic{figure}` — "Fig. A1", "Table A1", "(A1)" (2609.24634, 2609.11123, 2609.04047; audit
  // #363). The hook's `\end{oldappendices}` calls `\endoldappendices`, which the class never defines (it saves
  // `\oldendappendices`), so appendix.sty's closing `\@ppsaveapp\@pprestoresec` does not run: the section counter goes
  // on, and only the group's end restores the section form `\@resets@pp` set locally — a section after the
  // environment is "3 After", its figure "Fig. C2", in pdflatex. That restore is the appendix binding's end
  // (`\lx@pp@appendix@end`, which ends the appendices and leaves the counter), run after the class's no-op. (No
  // `{declarations}`: no sn-jnl.cls defines one — 0 of 206 shipped copies, 2021-25 — and no paper of ~23,000 sampled,
  // 2019-25, uses it; the Springer template writes `\section*{Declarations}`/`\bmhead`, so a binding environment was
  // speculative, rev78c.)
  if !package_loaded("appendix")
    && class_text
      .as_deref()
      .is_none_or(|text| crate::shipped_class::class_text_loads(text, "appendix"))
  {
    RequirePackage!("appendix", options => vec![s!("title")]);
  }
  if class_text
    .as_deref()
    .is_none_or(|text| class_code_contains(text, "\\let\\oldappendices\\appendices"))
  {
    RawTeX!(
      r"\@ifpackageloaded{appendix}{\AtBeginDocument{\let\oldappendices\appendices\let\oldendappendices\endappendices
\renewenvironment{appendices}{\setcounter{figure}{0}\setcounter{table}{0}\setcounter{equation}{0}\begin{oldappendices}\gdef\thefigure{\@Alph\c@section\arabic{figure}}\gdef\thetable{\@Alph\c@section\arabic{table}}\gdef\theequation{\@Alph\c@section\arabic{equation}}}{\end{oldappendices}\lx@pp@appendix@end}}}{}"
    );
  }
});

// The binding's record of the author block, as the class's own (`\aucount`, `\if@corauemail`, `\corrauthemail`):
/// The `\author` calls so far.
const AUTHORS: &str = "lx@sn@authors";
/// Whether the last `\author` was starred (`\if@corauemail`, sn-jnl.cls:505).
const STARRED: &str = "lx@sn@starred";
/// Whether an `\email` followed a starred name (`\corrauthemail`, :537).
const CORRESPONDING_EMAIL: &str = "lx@sn@corresponding@email";
/// Whether a name holds an `\email`, which runs as the names print, under the last `\author`'s star.
const NAME_EMAIL: &str = "lx@sn@name@email";
/// The starred names with labels, which print `*` (:510-511).
const STARRED_MARKED: &str = "lx@sn@starred@marked";
/// Whether a starred name has no labels, so prints no `*`.
const STARRED_UNMARKED: &str = "lx@sn@starred@unmarked";
/// The corresponding line's words, the copy's.
const CORRESPONDING_TEXT: &str = "lx@sn@corresponding@text";
/// Whether a starred name with labels prints `*` (:510-511, the copy's).
const STAR_PRINTS: &str = "lx@sn@star@prints";
/// Whether `\maketitle` has queued the notes of the names before it, so a note given inside a name, which runs as the
/// names print, marks none (the class's `\g@addto@macro\artauthors` too late for the print in progress).
const PRINTED: &str = "lx@sn@printed";

/// The author notes: the command, the macro its text goes in, the switch it sets and the note's role.
const AUTHOR_NOTES: [(&str, &str, &str, &str); 5] = [
  (
    "equalcont",
    "@equalconttext",
    "equalcont",
    "equal-contributors",
  ),
  ("equalsup", "@equalsuptext", "equalsup", "equal-supervision"),
  ("equaladv", "@equaladvtext", "equaladv", "equal-advising"),
  ("intern", "@interntext", "intern", "intern"),
  (
    "equalcontribution",
    "@equalcontributiontext",
    "equalcont",
    "equal-contributors",
  ),
];

/// The glyphs the copies append to the names (`\g@addto@macro\artauthors{$^{\dagger}$}`).
const NOTE_GLYPHS: [(&str, &str); 11] = [
  ("\\dagger", "\u{2020}"),
  ("\\dag", "\u{2020}"),
  ("\\ddagger", "\u{2021}"),
  ("\\ddag", "\u{2021}"),
  ("\\S", "\u{a7}"),
  ("\\P", "\u{b6}"),
  ("\\|", "\u{2016}"),
  ("\\star", "\u{22c6}"),
  ("\\bigstar", "\u{2605}"),
  ("\\ast", "\u{2217}"),
  ("*", "*"),
];

/// The frontmatter call of `\author[*][labels]{name}`: the labels, and the `*` a starred name prints after them, its
/// `annotations`; the name, then its marks macro (`\lx@sn@marks@<n>`), whose requests the notes after it add. A
/// starred name without labels requests the unmarked correspondent note.
fn author_call(starred: bool, labels: Option<Tokens>, name: &Tokens) -> Tokens {
  let n = lookup_int(AUTHORS) + 1;
  assign_value(AUTHORS, n, Scope::Global);
  assign_value(STARRED, starred, Scope::Global);
  assign_value(PRINTED, false, Scope::Global);
  if name.unlist_ref().contains(&T_CS!("\\email")) {
    assign_value(NAME_EMAIL, true, Scope::Global);
  }
  let mut annotations = labels.map(Tokens::unlist).unwrap_or_default();
  let marks = T_CS!(s!("\\lx@sn@marks@{n}"));
  let mut call = vec![T_CS!("\\gdef"), marks, T_BEGIN!()];
  if starred && annotations.is_empty() {
    assign_value(STARRED_UNMARKED, true, Scope::Global);
    call.extend(tokenized(r"\lx@request@frontmatter@annotation[sn]{corr}"));
  } else if starred && lookup_bool(STAR_PRINTS) {
    assign_value(
      STARRED_MARKED,
      lookup_int(STARRED_MARKED) + 1,
      Scope::Global,
    );
    annotations.extend([T_OTHER!(","), T_OTHER!("*")]);
  }
  call.push(T_END!());
  call.extend(tokenized(r"\lx@add@authors@append[annotations={"));
  call.extend(annotations);
  call.extend(tokenized("}]{"));
  call.extend(name.unlist_ref().iter().copied());
  call.extend([marks, T_END!()]);
  Tokens::new(call)
}

/// An author note's text stored and its switch set, as the class's command does; the names written so far end with
/// its request (the class's `\g@addto@macro\artauthors{$^{\dagger}$}`). Before any name, none; inside a name, read
/// as the names print, the document's note, unmarked (pdflatex prints its text, no name marked).
fn author_note(command: &str, text: Tokens) -> Tokens {
  let Some(&(_, store, switch, role)) = AUTHOR_NOTES.iter().find(|note| note.0 == command) else {
    return Tokens::new(Vec::new());
  };
  if lookup_bool(PRINTED) {
    let mut call = tokenized(&s!("\\@add@frontmatter{{ltx:note}}[role={role}]{{"));
    call.extend(text.unlist());
    call.push(T_END!());
    return Tokens::new(call);
  }
  assign_value(&s!("lx@sn@given@{command}"), true, Scope::Global);
  let mut call = vec![T_CS!("\\gdef"), T_CS!(s!("\\{store}")), T_BEGIN!()];
  call.extend(text.unlist());
  call.push(T_END!());
  call.extend(tokenized(&s!("\\global\\{switch}true")));
  let n = lookup_int(AUTHORS);
  if n > 0 && lookup_bool(&s!("lx@sn@marked@{command}")) {
    assign_value(&s!("lx@sn@requested@{command}"), true, Scope::Global);
    call.extend([
      T_CS!("\\g@addto@macro"),
      T_CS!(s!("\\lx@sn@marks@{n}")),
      T_BEGIN!(),
    ]);
    call.extend(tokenized(&s!(
      "\\lx@request@frontmatter@annotation[sn]{{{command}}}"
    )));
    call.push(T_END!());
  }
  Tokens::new(call)
}

/// The notes `\@maketitle` prints under the addresses (sn-jnl.cls:749-756), each queued once with its label, which
/// the names carrying it request: the correspondent note where the class prints its line (an `\email` after a starred
/// name, or one in a name printed under a starred last author), then each author note given. A `*` printed with no
/// line to explain it is counted lost (as authblk's labels, `count_lost_labels`); a note given before any name, which
/// marks none, is the document's. Read once: a second `\maketitle` prints only what was given since.
fn print_notes() -> Tokens {
  let mut tex = String::new();
  let text = lookup_string(CORRESPONDING_TEXT);
  let marked = lookup_int(STARRED_MARKED);
  let line = !text.is_empty()
    && (lookup_bool(CORRESPONDING_EMAIL) || (lookup_bool(NAME_EMAIL) && lookup_bool(STARRED)));
  if line {
    if marked > 0 {
      tex.push_str(&s!(
        "\\lx@annotate@frontmatter{{ltx:creator}}{{ltx:note}}[label={{*}},role=correspondent,mark=*]{{{text}}}"
      ));
    }
    if lookup_bool(STARRED_UNMARKED) {
      tex.push_str(&s!(
        "\\lx@annotate@frontmatter{{ltx:creator}}{{ltx:note}}[label={{sn:corr}},role=correspondent]{{{text}}}"
      ));
    }
  } else if marked > 0 {
    Warn!(
      "frontmatter",
      "unanswered_mark_lost",
      s!(
        "sn-jnl prints * on {marked} starred name(s) with no corresponding line to explain it, written on no name"
      )
    );
  }
  for &(command, store, _, role) in &AUTHOR_NOTES {
    if !lookup_bool(&s!("lx@sn@given@{command}")) {
      continue;
    }
    if lookup_bool(&s!("lx@sn@requested@{command}")) {
      let glyph = lookup_string(&s!("lx@sn@glyph@{command}"));
      let mark = if glyph.is_empty() {
        String::new()
      } else {
        s!(",mark={glyph}")
      };
      tex.push_str(&s!(
        "\\lx@annotate@frontmatter{{ltx:creator}}{{ltx:note}}[label={{sn:{command}}},role={role}{mark}]{{\\{store}}}"
      ));
    } else {
      tex.push_str(&s!(
        "\\@add@frontmatter{{ltx:note}}[role={role}]{{\\{store}}}"
      ));
    }
    assign_value(&s!("lx@sn@given@{command}"), false, Scope::Global);
    assign_value(&s!("lx@sn@requested@{command}"), false, Scope::Global);
  }
  for key in [CORRESPONDING_EMAIL, NAME_EMAIL, STARRED_UNMARKED] {
    assign_value(key, false, Scope::Global);
  }
  assign_value(STARRED_MARKED, 0i64, Scope::Global);
  assign_value(PRINTED, true, Scope::Global);
  Tokens::new(tokenized(&tex))
}

/// The glyph `command`'s definition in a copy appends to the names (`\g@addto@macro\artauthors{$^{\dagger}$}`,
/// 2412.01641's :586; `$^{*}$` 2501.01157's :587, `$^{\S}$` 2307.08214's :595): one of [`NOTE_GLYPHS`], an empty one
/// for another; `None` where it marks no name (`{}`, 2209.11450's) or the copy does not define the command.
fn note_glyph(text: &str, command: &str) -> Option<String> {
  let code = class_code(text);
  let at = code.find(&s!("\\newcommand{{\\{command}}}"))?;
  let body = &code[at + 1..];
  let body = &body[..body.find("\\newcommand").unwrap_or(body.len())];
  let mark = class_code_argument(body, "\\g@addto@macro\\artauthors")?;
  let mark = mark
    .trim()
    .trim_matches('$')
    .trim()
    .trim_start_matches('^')
    .trim();
  let mark = mark
    .strip_prefix('{')
    .and_then(|m| m.strip_suffix('}'))
    .unwrap_or(mark)
    .trim();
  if mark.is_empty() {
    return None;
  }
  Some(
    NOTE_GLYPHS
      .iter()
      .find(|(tex, _)| *tex == mark)
      .map_or_else(String::new, |(_, glyph)| (*glyph).to_string()),
  )
}

/// Whether a copy's starred name prints `*` after its labels (`{*}` in its `\@@corrauthor`, 2412.01641's :511; `{}` in
/// 2209.11450's).
fn star_prints(text: &str) -> bool {
  let code = class_code(text);
  let Some(at) = code.find("\\newcommand{\\@@corrauthor}") else {
    return true;
  };
  let body = &code[at + 1..];
  body[..body.find("\\newcommand").unwrap_or(body.len())].contains("{*}")
}

/// The words of a copy's corresponding line before its e-mails (2412.01641's :751 "Corresponding author(s).";
/// 2211.08503's "Corresponding author.", 2509.24703's "Corresponding authors.").
fn corresponding_text(text: &str) -> Option<String> {
  let code = class_code(text);
  let line = &code[code.find("Corresponding author")?..];
  let end = line
    .find("E-mail")
    .or_else(|| line.find("\\corrauthemail"))?;
  Some(line[..end].trim().to_string())
}

/// `tex` as tokens, under the internal catcodes (`@` a letter).
fn tokenized(tex: &str) -> Vec<Token> {
  mouth::tokenize_internal(TeXString::assembled(tex.to_string())).unlist()
}
