//! Stub for sn-jnl.cls (Springer Nature journal class).
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
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
  // `\if@fleqn` stays false); natbib and apacite (chosen by reference style,
  // below); and program —
  // this port binds it and Perl does not, and loading it turns `\(`…`\)`
  // into a programbox and makes `;` active in math (program.sty:67-76,
  // 175-176; old copies L311).
  require_dependencies_except("sn-jnl", "cls", &[
    "article", "natbib", "apacite", "program",
  ]);

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
  DeclareOption!("sn-apa", {
    RequirePackage!("apacite");
  });
  ProcessOptions!();

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
  // \theHfigure / \theHtable are hyperref H-counter overrides (no body).
  def_macro_noop("\\theHfigure{}")?;
  def_macro_noop("\\theHtable{}")?;

  // Author-block — attach author names, affiliations and emails to structured
  // `<ltx:creator>`/`<ltx:contact>` frontmatter, NOT loose top-level notes.
  // Mirrors jheppub_sty.rs (the Perl PR #2767 labeled-affiliation pattern).
  //
  // sn-jnl.cls (the witness cls) defines `\author*[N]{name}` (`\@ifstar` →
  // corresponding vs regular; the optional `[N]` is the affiliation-id list) and
  // `\affil*[N]{text}`. We collapse the corresponding-vs-regular star (both
  // become role=author) — we don't distinguish it in the output.
  //
  // `OptionalMatch:*` (not a literal `\author*`): our DefMacro prototype parser
  // treats a `*` immediately after `\author` as a literal Token parameter, not a
  // CS suffix, so `\author*[]{}` would force a mandatory star and (via a naive
  // self-forwarding body) recurse forever on a plain `\author{X}` — the
  // root-cause for 2306.11901. `OptionalMatch:*` makes the star optional, and the
  // body forwards to `\lx@add@creator` (never re-enters `\author`). The optional
  // `[N]` becomes the creator's `annotations` (the affiliation ids it references).
  DefMacro!(
    "\\author OptionalMatch:* []{}",
    "\\lx@add@creator[annotations={#2}]{#3}"
  );
  // `\affil*[N]{text}`: attach as a structured `ltx:contact[role=affiliation]`
  // ON the author creators, not an orphaned top-level `ltx:note`. The numbered
  // form links by id (contact label <-> the author's annotations); the
  // unnumbered shared form (a single `\affil` for all authors — the witness's
  // shape) uses `annotate=all` to attach to every preceding creator.
  // arXiv/html_feedback#534, witness 2204.04741.
  RawTeX!(
    r#"\def\lx@sn@affil#1#2{\def\lx@sn@affil@id{#1}%
\ifx\lx@sn@affil@id\@empty\lx@add@contact[role=affiliation,annotate=all]{#2}%
\else\lx@add@contact[role=affiliation,label={#1}]{#2}\fi}"#
  );
  DefMacro!("\\affil OptionalMatch:* []{}", "\\lx@sn@affil{#2}{#3}");
  DefMacro!(
    "\\equalcont{}",
    "\\@add@frontmatter{ltx:note}[role=equal-contributors]{#1}"
  );
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
  // Frontmatter envs — internal_vertical mode for multi-paragraph bodies
  // (declarations especially carries author prose with \par separators). Without
  // explicit mode, restricted_horizontal default trips Endgroup mismatch on
  // \par-containing bodies.
  DefEnvironment!("{declarations}", "<ltx:acknowledgements name='declarations'>#body</ltx:acknowledgements>",
    mode => "internal_vertical");
  DefEnvironment!("{appendices}", "<ltx:appendix>#body</ltx:appendix>",
    mode => "internal_vertical");
});
