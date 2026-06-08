/// Perl: icml_support.sty.ltxml — Support for ICML conference styles
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  RequirePackage!("times");
  RequirePackage!("fancyhdr");
  RequirePackage!("color");
  // ICML 2024/2025 templates use xcolor's \colorlet for callout colors;
  // load xcolor eagerly. Pre-load with [dvipsnames, table] so that
  // user `\usepackage[dvipsnames, table]{xcolor}` doesn't silently
  // option-clash and leave colortbl/dvipsnam.def unloaded. Witness
  // 2405.18180 (icml2025).
  RequirePackage!("xcolor", options => vec!["dvipsnames".to_string(), "table".to_string()]);
  RequirePackage!("algorithm");
  RequirePackage!("algorithmic");
  RequirePackage!("natbib");

  // Citations
  DefMacro!("\\yrcite Semiverbatim", "\\citeyearpar{#1}");
  DefMacro!("\\cite Semiverbatim", "\\citep{#1}");

  // icml2018.sty L709: `\long\def\comment#1{}` — a review-annotation macro that
  // gobbles its argument (authors write `\comment{\ref{…}}` to hide draft
  // notes). Our binding intercepts icml2016/2017/2018 but had omitted it, so a
  // paper using `\comment{…}` (via arxiv.sty → `\RequirePackage{icml2018}`) hit
  // `undefined:\comment` where Perl (which loads icml2018) is clean. Gobbling
  // is the intended, source-faithful behavior. Witness 1803.00942.
  DefMacro!("\\comment{}", "");

  // Frontmatter
  Let!("\\icmltitle", "\\title");
  // Perl gobbles \icmltitlerunning; surpass: it's the running-head
  // variant of the title, genuine author metadata.
  DefMacro!("\\icmltitlerunning{}",
    "\\@add@frontmatter{ltx:toctitle}{#1}");
  // \icmlsetsymbol{name}{symbol} — creates `\<name>` macro expanding
  // to <symbol> for use in author lists. Real icml2024.sty L100-ish:
  //   `\def\icmlsetsymbol#1#2{\expandafter\def\csname #1\endcsname{#2}}`
  // Previous stub gobbled both args without defining the CS, breaking
  // `\icmlauthor{...}{equal,affil,icmlWorkDone}` which later references
  // `\icmlWorkDone` in `\printAffiliationsAndNotice`. Witness 2310.06430.
  DefMacro!("\\icmlsetsymbol{}{}",
    "\\expandafter\\def\\csname #1\\endcsname{#2}");

  DefEnvironment!("{icmlauthorlist}", "#body");

  DefMacro!("\\icmlauthor{}{}", "\\author{#1}");
  // icml20XX.sty: \newcommand{\icmlCorrespondingAuthor}{\textsuperscript{$\dagger$}Corresponding author}
  // (0-arg marker). icml2025.sty L530: \newcommand{\icmlCorr}{\textsuperscript{\dag}Corresponding author }
  // (shorter alias, used as `\printAffiliationsAndNotice{\icmlCorr}`). Our binding
  // intercepts the raw .sty so the `\newcommand`s never run → undefined where Perl
  // (which loads the raw .sty) defines them. Witnesses 2403.01475, 2507.11588.
  //
  // The real macros are a `\textsuperscript{\dagger}` footnote marker, but that is a
  // typographic cross-reference that is meaningless in our structured model (no author
  // is marked with the matching dagger). Per the frontmatter-framework spirit, route the
  // corresponding-author indication into structured metadata as an `ltx:note`
  // (role=corresponding-author, matching `\icmlcorrespondingauthor` below) rather than
  // emitting raw inline `\textsuperscript` text. INTERIM: a fuller author↔contact
  // association will come with the frontmatter-refactor (PR #241 / upstream PR #2767).
  DefMacro!("\\icmlCorrespondingAuthor",
    "\\@add@frontmatter{ltx:note}[role=corresponding-author]{Corresponding author}");
  DefMacro!("\\icmlCorr",
    "\\@add@frontmatter{ltx:note}[role=corresponding-author]{Corresponding author}");
  DefConstructor!("\\@@@address{}", "^ <ltx:contact role='address'>#1</ltx:contact>");
  DefMacro!("\\icmladdress{}", "\\@add@to@frontmatter{ltx:creator}{\\@@@address{#1}}");
  // ICML: \icmlaffiliation{shortname}{full text} maps a short id to
  // an affiliation string used in author list. Preserve only the
  // full-text — the shortname (#1) is an internal identifier that
  // commonly contains `_` characters (e.g. `mit_idss`, `osu_ece`),
  // which our frontmatter pipeline tokenizes as math-mode subscript
  // and errors with "Script _ can only appear in math mode" when
  // rendered into ltx:note text. Witness 2404.08592.
  DefMacro!("\\icmlaffiliation{}{}",
    "\\@add@frontmatter{ltx:note}[role=affiliation]{#2}");
  // \icmlcorrespondingauthor{name}{email} — preserve as ltx:note.
  // The email arg often contains `_` (e.g. `m_smith@apple.com`) which would
  // otherwise be tokenized as subscript-mode at digest time, triggering
  // "Script _ can only appear in math mode" cascades. Semiverbatim
  // neutralizes `_`/`#`/`&`/`%`/`^`/`~`/`$`/`{`/`}` in the email arg.
  // Perl's icml_support binding gobbles both args (empty body); we surpass
  // by preserving the contact text as a frontmatter note. Witness 2312.09299.
  DefMacro!("\\icmlcorrespondingauthor{} Semiverbatim",
    "\\@add@frontmatter{ltx:note}[role=corresponding-author]{#2 <#1>}");

  // \printAffiliationsAndNotice / \printAffiliationsAndWorkNotice emit
  // a re-iteration of the affiliation list + a free-form notice. Since
  // \icmladdress already feeds frontmatter, the affiliation list is
  // captured separately; preserve the notice arg as a ltx:note so the
  // author-supplied "Work done while at X" string survives.
  // Witness: 2502.18679 (icml2025.sty L564).
  DefMacro!("\\printAffiliationsAndNotice{}",
    "\\@add@frontmatter{ltx:note}[role=affiliationnotice]{#1}");
  DefMacro!("\\printAffiliationsAndWorkNotice{}",
    "\\@add@frontmatter{ltx:note}[role=affiliationnotice]{#1}");
  // ICML 2024: simpler `\printAffiliations` no-arg form (newer template
  // emits the affiliation list inline without trailing notice). Witness
  // 2310.18127.
  def_macro_noop("\\printAffiliations")?;
  DefMacro!("\\icmlEqualContribution", "Equal contribution");
  // ICML 2023 (icml2023.sty L526): per-paper "equal advising" marker.
  // Witness 2306.01153.
  DefMacro!("\\icmlEqualAdvising", "Equal advising");
  // ICML 2025: extended marker for joint first + senior authorship.
  // Witness: 2503.15703 (icml2025.sty L534).
  DefMacro!("\\icmlEqualContributionAndSenior",
    "\\textsuperscript{*}Equal contribution, \
     \\textsuperscript{\\char`\u{2020}}Equal senior authorship");
  // ICML 2024 introduced `\icmlEqualSeniorContribution` — senior-only
  // joint authorship marker (no joint-first). Witness 2305.xxxxx in
  // wp4 (2 papers in stage 1).
  DefMacro!("\\icmlEqualSeniorContribution",
    "\\textsuperscript{\\char`\u{2020}}Equal senior contribution");
  // Some paper-bundled icml*.sty templates add author-status markers
  // beyond the kernel `\icmlEqualContribution`. The most common is
  // `\icmlIntershipWork`, an internship-affiliation annotation passed
  // to `\printAffiliationsAndNotice{...}` in icml2024 papers.
  // Witness 2401.00604.
  DefMacro!("\\icmlIntershipWork",
    "\\textsuperscript{*}Work done during an internship");
  // `\icmlInternship` — paper-bundled internship marker (correct spelling,
  // distinct from the `\icmlIntershipWork` typo above). icml2019.sty L502
  // `\newcommand{\icmlInternship}{\textsuperscript{*}This work has been done
  // during the internship at <institution>.}`. Papers pass it into
  // `\printAffiliationsAndNotice{\icmlInternship}`; our binding intercepts
  // icml20xx (so the paper's bundled def never runs) and — unlike Perl,
  // whose `\printAffiliationsAndNotice{}` gobbles its arg — preserves the
  // notice arg as a frontmatter note, expanding the inner CS. Provide a
  // generic fallback (institution unknowable from the binding) so the note
  // survives error-free. Witness 1902.02603 (icml2019.sty L502).
  DefMacro!("\\icmlInternship",
    "\\textsuperscript{*}Work done during an internship");
  // `\airesident` — paper-bundled AI-residency marker, same family as
  // `\icmlInternship`. icml2019.sty L503 `\newcommand{\airesident}{
  // \textsuperscript{$\dagger$}This work was completed as part of the
  // <program> AI Residency}`. Passed into
  // `\printAffiliationsAndNotice{\icmlEqualContribution\airesident}`; our
  // binding intercepts icml20xx so the bundled def never runs, and the
  // preserved notice arg expands the undefined CS. Generic fallback
  // (program unknowable from the binding). Witness 1902.09574.
  DefMacro!("\\airesident",
    "\\textsuperscript{\\char`\u{2020}}Work completed as part of an AI Residency");
  // \icmlOutsideContribution — paper-bundled marker noting that the
  // contribution was made outside the author's primary affiliation.
  // Witness 2310.14751.
  DefMacro!("\\icmlOutsideContribution",
    "\\textsuperscript{*}Work done outside of primary affiliation");
  // \icmlEqualLast — paper-bundled co-last-authors marker (icml2024.sty
  // L525). Witness 2402.02526.
  DefMacro!("\\icmlEqualLast",
    "\\textsuperscript{*}Co last authors");
  // \icmlIntern — paper-bundled internship-affiliation marker.
  // Witness 2312.05253 (icml2024.sty L534).
  DefMacro!("\\icmlIntern",
    "\\textsuperscript{*}Work done while interning");
  // \icmlEqualwork — paper-bundled joint-work marker (icml2021 papers
  // routinely redefine this in their bundled icml2021.sty to a custom
  // institutional note). Provide a generic fallback so the canonical
  // binding fires when the paper's bundled .sty is masked by ours.
  // Witness 2111.13293.
  DefMacro!("\\icmlEqualwork",
    "\\textsuperscript{*}Joint work");
  // \icmlProjectLead — paper-bundled project-lead marker.
  // Witness 2402.04924 (icml2024.sty L535).
  DefMacro!("\\icmlProjectLead",
    "\\textsuperscript{\\char`\u{2020}}Project lead");
  DefMacro!("\\icmlkeywords{}", "\\@add@frontmatter{ltx:keywords}{#1}");
  // \iclrfinalcopy — some icml*-bundled papers also use ICLR's
  // `\iclrfinalcopy` macro (bundled icml2023.sty L: `\def\iclrfinalcopy
  // {\iclrfinaltrue}`). Stub both as no-op. Witness 2206.06661.
  DefConditional!("\\ificlrfinal");
  def_macro_noop("\\iclrfinalcopy")?;

  // `\toptitlebar`/`\bottomtitlebar` — the decorative rules drawn above and
  // below the title block. Real icml20XX.sty (e.g. icml2019.sty L410-411):
  //   \def\toptitlebar{\hrule height1pt \vskip .25in}
  //   \def\bottomtitlebar{\vskip .22in \hrule height1pt \vskip .3in}
  // No-arg macros (the `\toptitlebar{\Large\bf #1}` in some bundled `arxiv.sty`
  // is `\toptitlebar` followed by a separate title group). Our binding
  // intercepts the paper-bundled icml20XX.sty so those defs never run; Perl
  // ships no icml2019 binding and raw-loads the .sty, reaching L410. Supply
  // them directly. Witness 1905.03711 (article + arxiv.sty →
  // `\RequirePackage[accepted]{icml2019}`).
  DefMacro!("\\toptitlebar", "\\hrule height1pt \\vskip .25in");
  DefMacro!("\\bottomtitlebar", "\\vskip .22in \\hrule height1pt \\vskip .3in");

  // Random extra bits
  def_macro_noop("\\abovestrut{}")?;
  def_macro_noop("\\belowstrut{}")?;
  def_macro_noop("\\abovespace")?;
  def_macro_noop("\\aroundspace")?;
  def_macro_noop("\\belowspace")?;
  def_macro_noop("\\icmlruler{}")?;
});
