//! Red/green guards for perfect-kernel batch 53 (sweep 28 KOMA cluster:
//! scrkbase font elements, `\DeclareSectionCommand` family, tocbasic).
//! Each test is the minimal reproduction distilled during triage; the
//! doc-comment names the ORIGINAL corpus witness (TeX Live doc corpus)
//! whose larger conversion was vetted separately.
use super::perfect_kernel_batch46::{convert, error_count};

/// eTeX `\numexpr`/`\dimexpr`/`\glueexpr` factor scanning (etex.ch
/// `scan_expr`, "Scan a factor f of type o or start a subexpression")
/// reads the next non-blank token with `get_x_token` and `back_input`s it
/// unless it is `(`. `back_input` re-inserts `cur_tok = cs_token_flag +
/// cur_cs` — the PLAIN control sequence — so a `\noexpand`'d macro at the
/// head of a factor loses its `no_expand_flag` and IS expanded by the
/// following `scan_int` (pdfTeX-probed: `\count255=\numexpr\noexpand\one+1
/// \relax` → 2, while the plain `\count255=\noexpand\one` → "Missing
/// number"). tocbasic.sty:2688-2690 relies on this:
/// `\edef…{\the\numexpr \noexpand\@nameuse{sectiontocdepth}+\@ne\relax}`.
/// RED: the expression reader unread the `\special_relax`-family token
/// unchanged, so `\@nameuse` stayed noexpand'd and `\numexpr` warned
/// "Missing number, treated as zero" (witness: every raw-tocbasic manual —
/// tikzlings-doc, glossaries-user, the KOMA classes' `\DeclareTOCStyleEntries`
/// probe). Perl's `readXToken` returns a bare `\special_relax` and warns
/// the same way; this is the noexpand-identity fidelity refinement already
/// recorded at the `\dont_expand` site in gullet.rs.
#[test]
fn numexpr_factor_reexpands_noexpanded_macro() {
  let (stderr, xml) = convert(
    r"\documentclass{article}
\makeatletter
\@namedef{sectiontocdepth}{1}
\edef\x{\the\numexpr \noexpand\@nameuse{sectiontocdepth}+\@ne\relax}
\def\one{1}
\count255=\numexpr(\noexpand\one+1)*2\relax
\dimen0=\dimexpr\noexpand\one pt+1pt\relax
\makeatother
\begin{document}
A\x.B\the\count255.C\the\dimen0.
\end{document}
",
    true,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!stderr.contains("Missing number"), "{stderr}");
  assert!(xml.contains("A2.B4.C2.0pt."), "{xml}");
}

/// `\IfFormatAtLeastTF` is a REAL definition in the latex.ltx dump
/// (`\@ifl@t@r\fmtversion`, latex.ltx L18405) — the always-true stub in
/// latex_constructs_rust_only.rs (issue #739, witnesses 2408.03197 /
/// 2408.04893, from before the dump carried it) shadowed it, so
/// scrbase.sty's `\IfLTXAtLeastTF{<KOMA year+2>/…}` (scrartcl.cls
/// L2028-2035) fired "Your are using a KOMA-Script version, that has not
/// been tested" on every KOMA document. RED: `{2099/01/01}` → Y.
#[test]
fn ifformatatleast_compares_real_fmtversion() {
  let (stderr, xml) = convert(
    r"\documentclass{article}
\begin{document}
A=\IfFormatAtLeastTF{2099/01/01}{Y}{N}.
B=\IfFormatAtLeastTF{2020/01/01}{Y}{N}.
\end{document}
",
    true,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("A=N."), "{xml}");
  assert!(xml.contains("B=Y."), "{xml}");
}

/// latex.ltx `\@sect` compares the `\@startsection` level with
/// `\ifnum #2>\c@secnumdepth` — a TeX <number>. scrartcl.cls L3421/L3425
/// pass every heading's level as `{\numexpr #2\relax}` (`#2` =
/// `\csname <name>numdepth\endcsname`). RED: the level was string-parsed
/// (Perl's `$level > …` coercion → 0), so `\paragraph` (level 4 >
/// secnumdepth 3) got NUMBERED under every raw KOMA class, and a
/// `\DeclareSectionCommand` heading with an unknown type opened a warned
/// `ltx:section` regardless of its level (witness tudaexercise
/// `\DeclareNewSectionCommand[level=2]{task}`: `<section xml:id="task1">`
/// plus `Warning:malformed:ltx:task`). The unknown type is now bound to
/// the element of its level (`SECTION_ELEMENT` mapping, OXIDIZED_DESIGN #175).
#[test]
fn startsection_level_is_a_tex_number() {
  let (stderr, xml) = convert(
    r"\documentclass{article}
\makeatletter
\newcounter{deep}\def\deepnumdepth{4}
\newcommand\deep{\@startsection{deep}{\numexpr\deepnumdepth\relax}{\z@}{1ex}{1ex}{\bfseries}}
\newcounter{task}[section]\renewcommand\thetask{\thesection.\arabic{task}}
\newcommand\task{\@startsection{task}{\numexpr 2\relax}{\z@}{1ex}{1ex}{\bfseries}}
\makeatother
\begin{document}
\section{S}
\task{T}
\deep{D}
\end{document}
",
    true,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!stderr.contains("malformed"), "{stderr}");
  assert!(
    xml.contains(r#"<subsection inlist="toc" xml:id="S1.task1">"#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<title><tag close=" ">1.1</tag>T</title>"#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<paragraph inlist="toc" xml:id="deepx1">"#),
    "{xml}"
  );
  assert!(xml.contains("<title>D</title>"), "{xml}");
}

const KOMA_TASK: &str = r"\documentclass{scrartcl}
\DeclareNewSectionCommand[style=section,level=2,counterwithin=section,tocstyle=section,indent=0pt,tocindent=1.5em,tocnumwidth=2.3em,beforeskip=1ex,afterskip=1ex,font=\bfseries]{task}
\begin{document}
\section{One}
\subsection{Sub}
\task{A task}
Body.
\paragraph{Para} text.
\end{document}
";

/// Raw scrartcl (host TeX Live; the class binding is a raw shim since
/// batch 53, `scrartcl_cls.rs`): `\DeclareNewSectionCommand[level=2]{task}`
/// opens a `<subsection>` and the raw class's `\paragraph` is unnumbered.
/// Witness: tudaexercise (DEMO-TUDaExercise), tikzlings-doc.
#[test]
fn koma_declaresectioncommand_heading_is_a_subsection() {
  let (stderr, xml) = convert(KOMA_TASK, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!stderr.contains("malformed"), "{stderr}");
  assert!(
    xml.contains(r#"<subsection inlist="toc" xml:id="task1">"#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<title><tag close=" ">1.1</tag>A task</title>"#),
    "{xml}"
  );
  assert!(
    xml.contains(
      r#"<paragraph inlist="toc" xml:id="section1.subsection1.subsubsection0.paragraphx1">"#
    ),
    "{xml}"
  );
  assert!(xml.contains("<title>Para</title>"), "{xml}");
}

/// Raw scrkbase font-element API (scrkbase.sty L452-670): `\newkomafont`
/// registers the element, `\usekomafont` EXPANDS to its switches; a later
/// `\usepackage{scrlayer-scrpage}` (→ scrlayer.sty L81 `\RequirePackage
/// {scrkbase}`) must not re-load anything that forgets the element. RED:
/// the scrartcl stub's no-op `\newkomafont` registered nothing, so the
/// real `\usekomafont` died with "font element myel not defined"
/// (witness contract-example-de/en).
#[test]
fn newkomafont_survives_scrlayer_scrpage_and_usekomafont_expands() {
  let (stderr, xml) = convert(
    r"\documentclass{scrartcl}
\newkomafont{myel}{\itshape}
\setkomafont{section}{\Large\bfseries}
\addtokomafont{title}{\rmfamily}
\usepackage{scrlayer-scrpage}
\begin{document}
Text {\usekomafont{myel}elem} plain.
\end{document}
",
    true,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains(r#"<text font="italic">elem</text> plain."#),
    "{xml}"
  );
}

/// KOMA title-page pieces (scrartcl.cls L2768-2803) are stored for the
/// class's own `\maketitle` (L2815), which is a locked constructor here;
/// `koma_script.rs` re-targets them at the frontmatter (witness 2305.01582
/// `\titlehead`; ar5iv #498).
#[test]
fn koma_title_pieces_reach_frontmatter() {
  let (stderr, xml) = convert(
    r"\documentclass{scrartcl}
\titlehead{Head}\subject{Subj}\subtitle{Sub}\title{Title}\author{A. U. Thor}\date{2026}\publishers{Pub}
\begin{document}
\maketitle
Body.
\end{document}
",
    true,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<subtitle>Sub</subtitle>"), "{xml}");
  assert!(
    xml.contains(r#"<note role="titlehead">Head</note>"#),
    "{xml}"
  );
  assert!(xml.contains(r#"<note role="subject">Subj</note>"#), "{xml}");
  assert!(
    xml.contains(r#"<note role="publishers">Pub</note>"#),
    "{xml}"
  );
  assert!(xml.contains("<title>Title</title>"), "{xml}");
}

/// Raw typearea (`typearea_sty.rs` is a raw shim since batch 53). RED: the
/// former typearea STUB left `\if@areasetadvanced` undefined, and
/// scrartcl.cls L2594-2628 tests it inside a skipped `\if…\else…\fi` branch
/// — tex.web `pass_text` only counts `if_test` commands, so an undefined
/// `\if@…` in the skipped text is not a conditional and its `\else`
/// terminated the OUTER skip ("Too many }'s" / "Extra \fi", exactly as
/// pdftex does with the same undefined `\if`). `\recalctypearea` /
/// `\areaset` exercised the same class code at body time (witness
/// bohr/bohr_en; arXiv 1502.06768, 1504.00554, 1504.00666).
#[test]
fn raw_typearea_defines_areaset_conditionals() {
  let (stderr, xml) = convert(
    r"\documentclass[11pt,DIV=12]{scrartcl}
\begin{document}
A\recalctypearea B\areaset{10cm}{20cm}C
\end{document}
",
    true,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!stderr.contains("unexpected:"), "{stderr}");
  assert!(
    xml.contains("A") && xml.contains("B") && xml.contains("C"),
    "{xml}"
  );
}

/// tikzlings-doc: `\RedeclareSectionCommand` + tocbasic's `\deftocheading`
/// on a raw scrartcl (both `undefined:` under the former stub).
#[test]
fn koma_redeclaresectioncommand_and_deftocheading() {
  let (stderr, xml) = convert(
    r"\documentclass{scrartcl}
\RedeclareSectionCommand[beforeskip=1ex,afterskip=1ex]{section}
\deftocheading{toc}{\section*{##1}}
\begin{document}
\tableofcontents
\section{One}
Body.
\end{document}
",
    true,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains(r#"<title><tag close=" ">1</tag>One</title>"#),
    "{xml}"
  );
}

/// l2tabu/l2tabuen: `\@declaredoptions` must expand to the declared option
/// list (latex.ltx L18536 `\xdef\@declaredoptions{\@declaredoptions,#1}`;
/// the Perl pool L784 binds it EMPTY). scrbase.sty L365 walks it after
/// `\FamilyProcessOptions` to retire every `\ds@<opt>`; with an empty list
/// the `\ds@` of a KOMA deprecated option (scrkbase.sty L365-407
/// `\KOMA@DeclareDeprecatedOption`) survived into typearea's own
/// `\KOMAProcessOptions` (typearea.sty L1053), which re-ran it as
/// "unknown option `captions=tableheading'".
#[test]
fn declaredoptions_lists_declared_options() {
  let (stderr, xml) = convert(
    r"\documentclass[tablecaptionabove]{scrartcl}
\makeatletter
\DeclareOption{alpha}{}\DeclareOption{beta}{}
\edef\x{\@declaredoptions}
\makeatother
\begin{document}
[\x]
\end{document}
",
    true,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!stderr.contains("unknown option"), "{stderr}");
  assert!(xml.contains("alpha,beta]"), "{xml}");
}

/// DEMO-TUDaPhD/TUDaThesis: `\@classoptionslist` / `\@raw@classoptionslist`
/// / `\@raw@opt@<file>` carry standard catcodes (Perl Package.pm L2564
/// `DefMacroI` string body → TokenizeInternal; the kernel stores the real
/// argument tokens). With every character OTHER, tudapub.cls L173/L358
/// forwarded an unknown option value to `\KOMAoption{parskip}{half-}` and
/// scrbase.sty L2354 `\FamilySetNumerical`'s `\ifx` against scrbook.cls
/// L825's literal `half-` failed ("unknown value"). The `\ifx` here is the
/// same comparison; the braced value checks that `{`/`}` group.
#[test]
fn classoptionslist_has_letter_catcodes() {
  let (stderr, xml) = convert(
    r"\documentclass[parskip=half-,thesis={type=dr,dr=rernat}]{article}
\begin{document}
\makeatletter
\def\lit{parskip=half-,thesis={type=dr,dr=rernat}}
\ifx\lit\@classoptionslist [same]\else [DIFFERENT]\fi
\ifx\lit\@raw@classoptionslist [rawsame]\else [rawDIFFERENT]\fi
\makeatother
\end{document}
",
    true,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("[same]") && xml.contains("[rawsame]"), "{xml}");
}

/// tutodoc-en/fr: a `\def` parameter text built by expansion keeps BOTH of
/// two adjacent space tokens in its delimiter (tex.web §473-476); Perl
/// TeX_Macro.pool.ltxml L127 collapsed them (KNOWN_PERL_ERRORS #119), so
/// expkv's `\ekv@set@was@blank` delimiter (two real spaces) never matched
/// and `\ekvset{clrstrip}{}` ran away to `Timeout:TokenLimit`.
#[test]
fn def_delimiter_keeps_adjacent_spaces() {
  let (stderr, xml) = convert(
    r"\documentclass{article}
\makeatletter
\def\A{}\def\B{}\def\SP{ }
\protected@edef\deltoks{\noexpand\A\SP\SP\noexpand\B}
\expandafter\def\expandafter\x\expandafter#\expandafter1\deltoks{[GOT:#1]OK}
\makeatother
\begin{document}
Before.
\expandafter\x\expandafter Q\deltoks
After.
\end{document}
",
    true,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!stderr.contains("Fatal:"), "{stderr}");
  assert!(xml.contains("[GOT:Q]OK"), "{xml}");
}

/// The expkv shape of the same defect: an EMPTY key list goes through
/// `\ekv@set@was@blank` (expkv.tex L709-712).
#[test]
fn expkv_blank_entry_does_not_leak_markers() {
  let (stderr, xml) = convert(
    r"\documentclass{article}
\usepackage{expkv}
\ekvdef{foo}{bar}{[V=#1]}
\begin{document}
X\ekvset{foo}{}Y\ekvset{foo}{bar=1, ,}Z
\end{document}
",
    true,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!stderr.contains("Fatal:"), "{stderr}");
  assert!(xml.contains("XY[V=1]Z"), "{xml}");
}

/// Convert `t.tex` next to a sidecar package file under the perfect-kernel
/// preload; `--includestyles --path .` makes the sidecar raw-loadable.
pub(super) fn convert_with_sty(tex: &str, sty_name: &str, sty_body: &str) -> (String, String) {
  super::perfect_kernel_batch46::convert_files(tex, &[(sty_name, sty_body)])
}

/// latex.ltx `\@pass@ptions` (L18509-18526) is the single writer of
/// `\@raw@opt@<name>.<ext>`, reached from `\PassOptionsToPackage` AND from
/// `\@onefilewithoptions` for the explicit `[…]` list; ltkeys'
/// `\ProcessKeyOptions` reads only that record (`\__keys_options_local:`,
/// L19457-19470). RED: the record was built from the explicit list alone,
/// so tudapub.cls L194 `\exp_args:Nx \PassOptionsToPackage{paper=…}
/// {tudarules}` never reached tudarules' `\ProcessKeyOptions[ptxcd/rules]`
/// and `\c_ptxcd_{large,small}rule_dim` stayed undefined (witness
/// DEMO-TUDaPhD, DEMO-TUDaThesis). pdflatex: `P=[A5]`.
#[test]
fn process_key_options_sees_passed_options() {
  let (stderr, xml) = convert_with_sty(
    r"\documentclass{article}
\usepackage{expl3}
\ExplSyntaxOn
\keys_define:nn {my/cls} {
  paper .choices:nn = { a4,a5 } {
    \exp_args:Nx \PassOptionsToPackage{paper=\l_keys_choice_tl}{mypk}
  },
}
\keys_set:nn {my/cls} {paper=a5}
\ExplSyntaxOff
\usepackage{mypk}
\begin{document}
P=[\csname g_my_paper_tl\endcsname] C=[\csname g_my_color_tl\endcsname]
\end{document}
",
    "mypk.sty",
    r"\ProvidesPackage{mypk}
\RequirePackage{expl3}
\ExplSyntaxOn
\tl_new:N \g_my_paper_tl
\keys_define:nn {my/rules} {
  paper .choice:,
  paper/a4 .code:n = { \tl_gset:Nn \g_my_paper_tl {A4} },
  paper/a5 .code:n = { \tl_gset:Nn \g_my_paper_tl {A5} },
  color .tl_gset:N = \g_my_color_tl,
  color .initial:n = black,
}
\ProcessKeyOptions[my/rules]
\ExplSyntaxOff
",
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("P=[A5] C=[black]"), "{xml}");
}

/// TeX's `\addcontentsline` (latex.ltx L17351-17363) writes its title to
/// the .toc through `\protected@write`, never typesetting it, which is
/// what makes LaTeX's write-only self-`\protect` idiom
/// `\def\appfmt#1{\protect\appfmt{#1}}` (nlctuserguide.sty L1553
/// `\@loe@disable@cmds`) safe. RED: the constructor digested the (then
/// discarded) title with `\protect`=`\relax`, so the macro re-expanded to
/// itself — `Fatal:Timeout:Recursion` (9-token window) or `TokenLimit`
/// (13-token, past the cycle guard's window). Witness glossaries-user
/// examples `ex:xdy`/`ex:mkidx`; Perl 0.8.8 hangs on this repro
/// (KNOWN_PERL_ERRORS #120).
#[test]
fn addcontentsline_title_is_not_digested() {
  let (stderr, xml) = convert(
    r"\documentclass{article}
\newcommand*{\appfmt}[1]{\texttt{#1}}
\makeatletter
\begin{document}
\def\thetitle{uses \appfmt{xindy}}%
\def\appfmt#1{\protect\appfmt{#1}}% \@loe@disable@cmds idiom
\addcontentsline{toc}{section}{\thetitle}%
done\end{document}
",
    true,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!stderr.contains("Fatal:"), "{stderr}");
  assert!(xml.contains(">done<"), "{xml}");
}

/// latex.ltx L18297-18300 defines `\pagestyle` as a plain `\def`;
/// scrlayer.sty L2183-2196 redefines it with the triple-`\expandafter`
/// freeze that inlines the OLD body at definition time. RED: `\pagestyle`
/// was a non-expandable primitive no-op, so the literal `\pagestyle{#1}`
/// survived in the new body and every later call recursed
/// (`Fatal:Timeout:Recursion`; raw scrlayer: `PushbackLimit` at
/// `\begin{document}` from `\AtBeginDocument{\pagestyle{test}}`). Perl
/// 0.8.8 hangs the same way (KNOWN_PERL_ERRORS #121). Witnesses
/// DEMO-TUDaPhD/TUDaThesis, neoschool, bfh-ci (raw scrlayer-scrpage).
#[test]
fn pagestyle_expandafter_freeze_terminates() {
  let (stderr, xml) = convert(
    r"\documentclass{article}
\makeatletter
\expandafter\expandafter\expandafter\renewcommand
\expandafter\expandafter\expandafter*%
\expandafter\expandafter\expandafter\pagestyle
\expandafter\expandafter\expandafter[%
\expandafter\expandafter\expandafter1%
\expandafter\expandafter\expandafter]%
\expandafter\expandafter\expandafter{\pagestyle{#1}}%
\makeatother
\begin{document}
\pagestyle{plain}\thispagestyle{empty}
Hello\end{document}
",
    true,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!stderr.contains("Fatal:"), "{stderr}");
  assert!(xml.contains(">Hello<"), "{xml}");
}

/// Raw `scrlayer-scrpage` on top of raw `scrlayer`/scrkbase: the former
/// stub defined no KOMA option keys, so a class's
/// `\KOMAoptions{headwidth=text,footsepline=…}` raised `unknown option`
/// and `\RedeclareLayer`/`\layerwidth`/`\DeclarePageStyleByLayers` were
/// undefined (witness DEMO-TUDaPhD, DEMO-TUDaThesis, neoschool, bfh-ci).
/// Structural: the body paragraph survives `\begin{document}` (where the
/// old raw load died with `PushbackLimit`).
#[test]
fn raw_scrlayer_scrpage_loads_and_sets_keys() {
  let (stderr, xml) = convert(
    r"\documentclass{scrbook}
\usepackage[automark]{scrlayer-scrpage}
\KOMAoptions{headwidth=text,footsepline=.5pt}
\DeclareNewLayer[background,contents={\layerwidth}]{mylayer}
\DeclareNewPageStyleByLayers{mystyle}{mylayer}
\RedeclareLayer[foreground]{mylayer}
\pagestyle{mystyle}
\begin{document}
\chapter{One}
Hello\end{document}
",
    true,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!stderr.contains("Fatal:"), "{stderr}");
  assert!(xml.contains(">Hello<"), "{xml}");
}

/// xltabular.sty L86-96: the environment restores longtable's
/// `\caption`/`\endhead`/`\endfirsthead`/`\endfoot`/`\endlastfoot` and runs
/// `\expandafter\longtable\the\toks@\endlongtable` — it IS a longtable with
/// `X` columns. RED: the binding aliased `\xltabular` to `\tabularx`, so the
/// class `\caption` ran inside a tabularx (`Use of \caption outside any
/// known float`); under raw KOMA the tocbasic expl3 `\caption` then read an
/// undefined `\@captype` and cascaded into a `TooManyErrors` fatal
/// (witnesses xltabular-doc 36→101 errors + fatal, hvfloat 27→90). GREEN:
/// one `<ltx:table>` carrying the caption, a `<thead>` from `\endfirsthead`
/// and the body rows.
#[test]
fn xltabular_is_a_longtable() {
  let (stderr, xml) = convert(
    r"\documentclass{scrartcl}
\usepackage{booktabs,xltabular}
\begin{document}
\begin{xltabular}{\textwidth}{@{} l>{\small\ttfamily}cX @{}}
\caption{The optional keywords}\label{tab:options}\\\toprule
Keyword & Default & Description\\\midrule
\endfirsthead
\midrule
\endhead
\bottomrule
\endfoot
\endlastfoot
onlyText & false & Only the text \\
capPos & b & caption position\\
\end{xltabular}
\end{document}
",
    true,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!stderr.contains("Fatal:"), "{stderr}");
  assert!(
    xml.contains("<table inlist=\"lot\" labels=\"LABEL:tab:options\""),
    "{xml}"
  );
  assert!(xml.contains("The optional keywords</caption>"), "{xml}");
  assert!(xml.contains("<thead>"), "{xml}");
  assert!(xml.contains(">caption position<"), "{xml}");
}

/// hvfloat surface the manual exercises (witness hvfloat manual, 79→7
/// errors; the 7 left are the SHARED `\endflushleft`-in-group mode-frame
/// family and a SHARED eager `backgroundcolor=\color` evaluation in the
/// listings binding). hvfloat.sty L630-636: an EMPTY float type is
/// `nonFloat,onlyText` — object and caption as plain text, no float, no
/// counter (was `undefined:{}` and a `\caption` outside any float);
/// L1264-1266 `hvFloatEnv` is a minipage; L24-36 `[fbox,hyperref]` options
/// are `\newif` switches; L306-307 `\hvDefFloatStyle` = `\defhvstyle`;
/// L55 `\RequirePackage{ifoddpage}` — pure kernel TeX that now loads raw
/// in both preload modes (`\checkoddpage`/`\ifoddpage` were `undefined`).
#[test]
fn hvfloat_only_text_env_options_and_ifoddpage() {
  let (stderr, xml) = convert(
    r"\documentclass{article}
\usepackage[fbox,hyperref]{hvfloat}
\begin{document}
\hvDefFloatStyle{main}{capPos=r}
\checkoddpage\ifoddpage odd\else even\fi
\hvFloat[onlyText=true]{}{\rule{2cm}{1cm}}{Only text, no float}{txt:only}
\begin{hvFloatEnv}
\rule{1cm}{1cm}
\captionof{figure}{Inside the env}\label{fig:env}
\end{hvFloatEnv}
\hvFloat{figure}{\rule{1cm}{1cm}}[Short]{A real figure}{fig:real}
\makeatletter\hv@fboxfalse\makeatother
\end{document}
",
    true,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!stderr.contains("Fatal:"), "{stderr}");
  assert!(!stderr.contains("missing_file"), "{stderr}");
  assert!(xml.contains(">odd<") || xml.contains("odd\n"), "{xml}");
  assert!(xml.contains("Only text, no float"), "{xml}");
  assert!(
    xml.contains("<figure inlist=\"lof\" labels=\"LABEL:fig:env\""),
    "{xml}"
  );
  assert!(
    xml.contains("<figure inlist=\"lof\" labels=\"LABEL:fig:real\""),
    "{xml}"
  );
  assert!(xml.contains("A real figure</caption>"), "{xml}");
  // The onlyText form opens no float: exactly the two real figures.
  assert_eq!(xml.matches("<figure ").count(), 2, "{xml}");
}

/// collcell binding (witness onedown-ref.tex:469 `\begin{bidding}`,
/// onedown.sty:1326 `>{\collectcell\ODw@BTfer}c<{\endcollectcell}`): the
/// raw `\collectcell#1#2\ignorespaces` (collcell.sty:76) is delimited by the
/// kernel cell template's `\ignorespaces` (latex.ltx:16671-16675), which
/// LaTeXML's alignment never inserts — the scan ran to end of input and
/// Rust's Until-at-EOF Fatal lost the whole 500-line manual (39-byte XML;
/// Perl: 6 errors, completes). The binding collects the cell unexpanded like
/// `\collect@cell@look`, letting a `\\`/`\tabularnewline` row end expand so
/// the `<{\endcollectcell}` column-after template fires (a plain
/// `Until:\endcollectcell` swallowed the rest of the document whenever the
/// collected column was the LAST of the row); `\cci{…}` is skipped
/// (onedown.sty:1338 bidding headers).
#[test]
fn collcell_hands_cell_to_macro() {
  let (stderr, xml) = convert(
    r"\documentclass{article}
\usepackage{array}
\usepackage{collcell}
\newcommand\Xfer[1]{[#1]}
\newcolumntype{B}{>{\collectcell\Xfer}c<{\endcollectcell}}
\begin{document}
\begin{tabular}{BB}
\cci{ West} & \cci{ North} \\
1S & 2 H \\[2pt]
{pass} \textbf{x} & 3NT \tabularnewline
a & b \cr
\end{tabular}
\end{document}
",
    true,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!stderr.contains("Fatal:"), "{stderr}");
  // `\cci{ West}` hands the macro the braced group as is, space included
  // (onedown's "there MUST be a ' '" first-token probe relies on it).
  for cell in [
    "[ West]", "[ North]", "[1S]", "[2 H]", "[3NT]", "[a]", "[b]",
  ] {
    assert!(xml.contains(&format!(">{cell}</td>")), "{cell}: {xml}");
  }
  assert!(
    xml.contains(">[pass <text font=\"bold\">x</text>]</td>"),
    "{xml}"
  );
  assert_eq!(xml.matches("<td ").count(), 8, "{xml}");
}

/// The `\opt@<file>` macro (latex.ltx `\@pass@ptions`, `\@ptionlist`)
/// holds the ARGUMENT tokens of `\usepackage[...]`, so `{`/`}` group. Ours
/// was rebuilt with `ExplodeText!` (braces OTHER), and every brace-aware
/// consumer of `\@ptionlist` split a braced value at its inner comma:
/// l3clist (`\clist_set:cx {…}{\@ptionlist{…}}`, URspecialopts → tudapub
/// lost `thesis={type=dr,dr=rernat}`, so DEMO-TUDaPhD never input
/// tudathesis.cfg: `\department`/`\affidavit` undefined) and xkeyval's
/// `\ProcessOptionsX` (glossaries-extra.sty:811 `\@for` read
/// `stylemods={mcols,bookindex}` as the style file `glossary-{mcols.sty`;
/// witness glossaries-user). Perl: 0 errors. pdflatex: `I=[mcols][bookindex]
/// T=[type=dr,dr=rernat]`.
#[test]
fn opt_macro_keeps_braced_option_values() {
  let (stderr, xml) = convert_with_sty(
    r"\documentclass{article}
\usepackage[english,stylemods={mcols,bookindex},thesis={type=dr,dr=rernat}]{mypk}
\begin{document}
I=[\csname my@items\endcsname] T=[\csname g_my_thesis_tl\endcsname]
\end{document}
",
    "mypk.sty",
    r"\ProvidesPackage{mypk}
\RequirePackage{expl3,xkeyval}
\ExplSyntaxOn
\tl_new:N \g_my_thesis_tl
\cs_new:Npn \my_kv:nn #1#2 { \tl_gset:Nn \g_my_thesis_tl {#2} }
\clist_set:Nx \l_tmpa_clist {\@ptionlist{mypk.sty}}
\clist_map_inline:Nn \l_tmpa_clist {
  \tl_if_in:nnT {#1} {thesis=} { \keyval_parse:NNn \use_none:n \my_kv:nn {#1} }
}
\ExplSyntaxOff
\def\my@items{}
\define@key{mypk.sty}{stylemods}{\@for\my@tmp:=#1\do{\edef\my@items{\my@items[\my@tmp]}}}
\define@key{mypk.sty}{thesis}{}
\define@key{mypk.sty}{english}[]{}
\ProcessOptionsX
",
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains("I=[[mcols][bookindex]] T=[type=dr,dr=rernat]"),
    "{xml}"
  );
}

/// tcbxparse's `\NewTCBListing{code}{ O{} m !O{} !O{x} !O{y} }`
/// (neoschool.cls:5168) takes ONE mandatory argument and, absent a `[`,
/// no optionals; our delegate counted every specifier as a mandatory
/// `\lstnewenvironment` argument, so a bare `\begin{code}{latex}` grabbed
/// body tokens through its own `\end`, the verbatim scan ran to the NEXT
/// `\end{code}` and swallowed the `\begin{sidebyside}` in between —
/// tcolorbox's global layer counter (tcolorbox.sty:1411 `\tcb@layer@inc`
/// never run for the eaten begin, `\tcb@layer@dec` run at its end) went
/// negative and every later box errored `every box on layer 0/-N`
/// (witness neoschool, 251 of 273 errors; Perl 0; pdflatex clean).
#[test]
fn tcb_listing_trailing_optionals_not_mandatory() {
  let (stderr, xml) = convert(
    r"\documentclass{article}
\usepackage[most]{tcolorbox}\usepackage{listings}
\NewTCBListing{code}{ O{} m !O{} !O{x} !O{y} }{listing only}
\newtcolorbox{sidebyside}[1][]{sidebyside,enhanced,bicolor,#1}
\begin{document}
\begin{code}{latex}
xx
\end{code}
\begin{sidebyside}[righthand width=.5\linewidth]
\begin{code}[numbers=none]{latex}
inner
\end{code}
\tcblower l\end{sidebyside}
\begin{sidebyside}u2\tcblower l2\end{sidebyside}
\end{document}
",
    false,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!stderr.contains("every box on layer"), "{stderr}");
  // Two separate listings: the first one ends at ITS `\end{code}`.
  assert_eq!(xml.matches("<listing ").count(), 2, "{xml}");
  assert!(
    !xml.contains("begin{sidebyside}"),
    "listing swallowed the box: {xml}"
  );
  assert!(xml.contains("u2"), "{xml}");
}

/// latex.ltx L1185-1188: `\typeout` writes its argument under
/// `\set@display@protect` (L1438 `\let\protect\string`), so a robust
/// command is written by NAME. Expanding it with `\protect`=`\relax`
/// entered raw KOMA's `\DeclareRobustCommand\small` (scrsize10pt.clo:62),
/// whose `\@setfontsize\small…` re-expands `\small` without end — the same
/// overflow pdflatex gives `\edef\x{\small}` — from hvextern.sty:325
/// `\hv@ex@typeout{Running BodyVerbatim with fontsize=\small,…}`
/// (witness hvextern manual: `Fatal:Timeout:PushbackLimit`; Perl 0, its
/// `\small` being a primitive). pdflatex writes exactly
/// `SIZE:[fontsize=\small \add@extra@listi{sml},fontfamily=tt]`.
#[test]
fn typeout_writes_robust_commands_by_name() {
  let (stderr, xml) = convert(
    r"\documentclass{scrartcl}
\begin{document}
\typeout{SIZE:[fontsize=\small,fontfamily=tt]}
x
\end{document}
",
    true,
  );
  assert!(!stderr.contains("Fatal:"), "{stderr}");
  assert!(
    stderr.contains("SIZE:[fontsize=\\small \\add@extra@listi{sml},fontfamily=tt]"),
    "{stderr}"
  );
  assert!(xml.contains("<p>x</p>"), "{xml}");
}

/// siunitx.sty:5014-5016 loads translations.sty when it exists, and
/// translations.sty:36 loads etoolbox — that is how a class using siunitx
/// early has `\AtEndPreamble` (neoschool.cls:1123). The binding required
/// neither, so `\AtEndPreamble` was `undefined` (Perl 0, pdflatex clean).
#[test]
fn siunitx_loads_translations() {
  let (stderr, xml) = convert(
    r"\documentclass{article}
\usepackage{siunitx}
\AtEndPreamble{\def\marker{HOOKED}}
\begin{document}
\marker
\end{document}
",
    false,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("HOOKED"), "{xml}");
}

/// hyperref.sty:3979-3992 defaults `\@pdftitle`/`\@pdfauthor`/… to
/// `\@empty` and each metadata key runs `\pdfstringdef\@pdf<key>{#1}`
/// (L3543-3596). nlctuserguide.sty:1630 reads `\@pdfauthor` directly
/// (witness glossaries-extra-manual.tex:48); Perl leaves it undefined.
#[test]
fn hypersetup_defines_pdf_info_macros() {
  let (stderr, xml) = convert(
    r"\documentclass{article}
\usepackage{hyperref}
\makeatletter
\begin{document}
A=[\@pdfauthor]
\hypersetup{pdfauthor={Nicola Talbot},pdftitle={The Title}}
B=[\@pdfauthor/\@pdftitle/\@pdfcreator]
\end{document}
",
    false,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("A=[]"), "{xml}");
  assert!(
    xml.contains("B=[Nicola Talbot/The Title/LaTeX with hyperref]"),
    "{xml}"
  );
}

/// xkeyval.tex:496-502: in `\ProcessOptionsX` — star or not — an unknown
/// option runs the `\DeclareOptionX*` handler `\XKV@doxs` when one is
/// defined; the star only adds the class options to the scan. The binding
/// (like Perl xkeyval.sty.ltxml:355, KNOWN_PERL_ERRORS #122) armed the
/// handler for `\ProcessOptionsX*` only, so the non-star form dropped
/// `english`/`foo=bar` with two "unknown KeyVals key" warnings. pdflatex:
/// `E=[[english][foo=bar]] W=[3cm]`.
#[test]
fn processoptionsx_unknown_option_reaches_star_handler() {
  let (stderr, xml) = convert_with_sty(
    r"\documentclass{article}
\usepackage[english,width=3cm,foo=bar]{mypk}
\makeatletter
\begin{document}
E=[\my@extra] W=[\my@width]
\end{document}
",
    "mypk.sty",
    r"\ProvidesPackage{mypk}
\RequirePackage{xkeyval}
\def\my@extra{}
\define@key{mypk.sty}{width}{\def\my@width{#1}}
\DeclareOptionX*{\edef\my@extra{\my@extra[\CurrentOption]}}
\ProcessOptionsX
",
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!stderr.contains("unknown KeyVals key"), "{stderr}");
  assert!(xml.contains("E=[[english][foo=bar]] W=[3cm]"), "{xml}");
}
