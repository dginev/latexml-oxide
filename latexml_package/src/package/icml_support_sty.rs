/// Perl: icml_support.sty.ltxml — Support for ICML conference styles
use crate::{
  prelude::*,
  shipped_class::{
    class_code_argument, class_code_contains, class_text_loads, require_shipped_algorithm_packages,
    require_shipped_packages, shipped_copy_text,
  },
};

LoadDefinitions!({
  // Perl icml_support.sty.ltxml:9-10, reached by name (icml.sty.ltxml): `algorithm` and `algorithmic`.
  icml_support(false)?;
});

/// icml2016.sty … icml2025.sty, which this binding stands in for (Perl has no binding for them; its versioned-name
/// fallback answers them with icml.sty.ltxml, so the distributed pair loads whatever the copy says, KNOWN_PERL_ERRORS
/// #684): every copy as ICML distributes it loads `algorithm` and `algorithmic` (icml2016.sty:70-71, icml2017.sty:72-73,
/// icml2025.sty:95-96), but a paper using `algpseudocode` edits `algorithmic` out of its copy (icml2024.sty:94
/// "removed due to conflict with algpseudocode", 2402.05403) — loaded, it bails algorithmicx (algorithmicx.sty.ltxml:9-12)
/// and leaves `\State`/`\For` undefined (1706.03643, 1706.04918, 1706.04933, 1706.04964, 2002.06473, 2106.05074). The
/// paper's copy decides the algorithm packages; the distributed pair without one.
pub fn load_icml_style() -> Result<()> { icml_support(true) }

#[rustfmt::skip]
fn icml_support(follow_paper_copy: bool) -> Result<()> {
  // The style as requested (`icml2026`, also through the versioned fallback: `fallback_request`), whose year decides
  // what differs between versions.
  let fallback_request = lookup_string("fallback_request");
  let request = match fallback_request.rsplit('/').next() {
    Some(name) if name.starts_with("icml") => fallback_request.clone(),
    _ => do_expand(T_CS!("\\@currname"))?.to_string(),
  };
  let version = request.rsplit('/').next().unwrap_or_default().to_string();
  let year = version.strip_prefix("icml").and_then(|year| year.parse::<u32>().ok());
  // The paper's copy of the style (`None` without one, and for icml.sty's own pair), whose edits of the notice
  // strings and the packages it loads are the paper's.
  let copy = if follow_paper_copy { shipped_copy_text("sty")? } else { None };
  RequirePackage!("times");
  RequirePackage!("fancyhdr");
  RequirePackage!("color");
  // ICML 2024/2025 templates use xcolor's \colorlet for callout colors;
  // load xcolor eagerly. Pre-load with [dvipsnames, table] so that
  // user `\usepackage[dvipsnames, table]{xcolor}` doesn't silently
  // option-clash and leave colortbl/dvipsnam.def unloaded. Witness
  // 2405.18180 (icml2025).
  RequirePackage!("xcolor", options => vec!["dvipsnames".to_string(), "table".to_string()]);
  if !(follow_paper_copy && require_shipped_algorithm_packages("sty")?) {
    RequirePackage!("algorithm");
    RequirePackage!("algorithmic");
  }
  RequirePackage!("natbib");
  // icml2026.sty:124-127 (and the 2025 copies that add them) `\RequirePackage{eso-pic}`, `{forloop}`, `{url}`,
  // `{caption}`: required as the paper's copy loads them. Its `\printAffiliationsAndNotice` runs `\forloop` over the
  // affiliations, which a paper's own redefinition copies (2601.04575).
  if follow_paper_copy {
    require_shipped_packages("sty", &["eso-pic", "url", "caption"])?;
  }
  // forloop has no binding, which the copy's dependency scan would load; 2026's style loads it (icml2026.sty:125), and
  // with the affiliation counters (icml2017.sty … icml2026.sty:434, :518; icml2016.sty has none) a paper's own copy
  // of the loop runs. A copy decides both; without one, the version.
  let copy_or = |code: &str, by_version: bool| copy.as_deref().map_or(by_version, |text| class_code_contains(text, code));
  let copy_loads_or = |package: &str, by_version: bool| copy.as_deref().map_or(by_version, |text| class_text_loads(text, package));
  if copy_loads_or("forloop", year == Some(2026)) {
    RequirePackage!("forloop");
  }
  if copy_or(r"\newcounter{@affiliationcounter}", year.is_some_and(|year| year >= 2017)) {
    // (icml2019.sty, read raw by its binding, has defined them already)
    RawTeX!(r"\@ifundefined{c@@affiliationcounter}{\newcounter{@affiliationcounter}\newcounter{@affilnum}}{}");
  }

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
    "\\lx@add@frontmatter{ltx:toctitle}{#1}");
  // \icmlsetsymbol{name}{symbol} — creates `\<name>` macro expanding
  // to <symbol> for use in author lists. Real icml2024.sty L100-ish:
  //   `\def\icmlsetsymbol#1#2{\expandafter\def\csname #1\endcsname{#2}}`
  // Previous stub gobbled both args without defining the CS, breaking
  // `\icmlauthor{...}{equal,affil,icmlWorkDone}` which later references
  // `\icmlWorkDone` in `\printAffiliationsAndNotice`. Witness 2310.06430.
  DefMacro!("\\icmlsetsymbol{}{}",
    "\\expandafter\\def\\csname #1\\endcsname{#2}");

  DefEnvironment!("{icmlauthorlist}", "#body");

  // \icmlauthor{author}{labels}   One per author; labels used to connect to affiliation
  DefMacro!("\\icmlauthor{} Semiverbatim", "\\lx@add@creator[role=author,annotations={#2}]{#1}");
  // icml20XX.sty: \newcommand{\icmlCorrespondingAuthor}{\textsuperscript{$\dagger$}Corresponding author}
  // (0-arg marker). icml2025.sty L530: \newcommand{\icmlCorr}{\textsuperscript{\dag}Corresponding author }
  // (shorter alias, used as `\printAffiliationsAndNotice{\icmlCorr}`). Our binding
  // intercepts the raw .sty so the `\newcommand`s never run → undefined where Perl
  // (which loads the raw .sty) defines them. Witnesses 2403.01475, 2507.11588.
  //
  // These are 0-arg footnote-style markers carrying no author identity, so — unlike the
  // 2-arg `\icmlcorrespondingauthor` below (name+email → `\lx@add@contact`) — they stay
  // note-style metadata (role=corresponding-author) via `\lx@add@frontmatter` rather
  // than emitting a raw inline `\textsuperscript` footnote marker.
  DefMacro!("\\icmlCorrespondingAuthor",
    "\\lx@add@frontmatter{ltx:note}[role=corresponding-author]{Corresponding author}");
  DefMacro!("\\icmlCorr",
    "\\lx@add@frontmatter{ltx:note}[role=corresponding-author]{Corresponding author}");
  // \icmlaffiliation{label}{address}  provides affiliation for author with same label
  DefMacro!("\\icmlaffiliation Semiverbatim {}", "\\lx@add@contact[role=affiliation,label={#1}]{#2}");
  DefMacro!("\\icmladdress OptionalSemiverbatim {}", "\\lx@add@contact[role=address,label={#1}]{#2}"); // deprecated
  // \icmlcorrespondingauthor{author}{email}
  // Attempt to use author name as a matching label to connect to author
  // (should be a fuzzy match?)
  DefMacro!("\\icmlcorrespondingauthor{}{}",
    "\\lx@add@contact[label={\\ifx.#1.\\else fuzzy:#1\\fi},role=email,name={Correspondence to: }]{#2}");

  // \printAffiliationsAndNotice / \printAffiliationsAndWorkNotice emit
  // a re-iteration of the affiliation list + a free-form notice. Since
  // \icmladdress already feeds frontmatter, the affiliation list is
  // captured separately; preserve the notice arg as a ltx:note so the
  // author-supplied "Work done while at X" string survives.
  // Witness: 2502.18679 (icml2025.sty L564).
  DefMacro!("\\printAffiliationsAndNotice{}",
    "\\lx@add@frontmatter{ltx:note}[role=affiliationnotice]{#1}");
  DefMacro!("\\printAffiliationsAndWorkNotice{}",
    "\\lx@add@frontmatter{ltx:note}[role=affiliationnotice]{#1}");
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
  // Every icml20xx.sty sends the keywords only to the PDF metadata (icml2018.sty:530-541 … icml2026.sty:546-550
  // `\ifdefined\nohyperref\else\ifdefined\hypersetup\hypersetup{pdfkeywords={#1}}\fi\fi`; the printed "Keywords:" line is
  // commented out in each), which hyperref_sty.rs writes as RDFa (`dcterms:subject`), not a displayed keywords block
  // (Perl `\lx@add@keywords`, icml_support.sty.ltxml:55; user ruling 2026-10-09; 2602.13503).
  DefMacro!("\\icmlkeywords{}",
    "\\ifdefined\\nohyperref\\else\\ifdefined\\hypersetup\\hypersetup{pdfkeywords={#1}}\\fi\\fi");
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

  // The notice under the first column and its options (icml2016.sty … icml2025.sty:103-133, icml2026.sty:129-177),
  // which papers print from their own `\printAffiliationsAndNotice` (`…#1\\[2pt]\Notice@String`, 32 papers;
  // 2601.23143), and icml2026.sty:504-516's check that the notice was printed, a flag the template's own redefinition
  // sets (`\global\icml@noticeprintedtrue`, example_paper.tex; 95 papers, 2601.22575): the review notice, `accepted`'s
  // proceedings line (`\ICML@appearing`) and 2026's `preprint` one, and the `\ificmlshowauthors` switch the two
  // options turn on. The strings are the paper's copy's where it ships one — authors edit them (a workshop's line:
  // 2606.28747 "3rd AI for Math Workshop", 2604.14572, 2507.02987, 2506.09099; `accepted` setting no notice,
  // 2106.00042) — else the version's as ICML distributed it; a version whose copies disagree (icml2020's PMLR 108 in
  // 2006.00731 and 2006.00866, 119 in 2006.00706, empty in 2006.00937 and 2009.02479) or that is unknown has an empty
  // line, so an accepted paper prints neither the review notice nor an invented proceedings line.
  let copy_definition = |name: &str| {
    copy
      .as_deref()
      .and_then(|text| class_code_argument(text, &s!("\\newcommand{{\\{name}}}")))
  };
  // An option's notice: the `\renewcommand{\Notice@String}{…}` of the copy's `\DeclareOption{option}` (`None` inside
  // when that option sets none; `None` outside when the copy declares no such option), or `by_version` without a copy.
  let option_notice = |option: &str, by_version: Option<&str>| match copy.as_deref() {
    Some(text) => class_code_argument(text, &s!("\\DeclareOption{{{option}}}"))
      .map(|body| class_code_argument(&body, r"\renewcommand{\Notice@String}")),
    None => by_version.map(|notice| Some(notice.to_string())),
  };
  let appearing = copy_definition("ICML@appearing").unwrap_or_else(|| {
    let proceedings = match year {
      Some(2015) => r"\textit{Proceedings of the $\mathit{32}^{nd}$ International Conference on Machine Learning}, Lille, France, 2015. JMLR: W\&CP volume 37.  Copyright 2015 by the author(s).",
      Some(2016) => r"\textit{Proceedings of the $\mathit{33}^{rd}$ International Conference on Machine Learning}, New York, NY, USA, 2016. JMLR: W\&CP volume 48.  Copyright 2016 by the author(s).",
      Some(2017) => r"\textit{Proceedings of the $\mathit{34}^{th}$ International Conference on Machine Learning}, Sydney, Australia, 2017. JMLR: W\&CP.  Copyright 2017 by the author(s).",
      Some(2018) => r"\textit{Proceedings of the $\mathit{35}^{th}$ International Conference on Machine Learning}, Stockholm, Sweden, PMLR 80, 2018. Copyright 2018 by the author(s).",
      Some(2019) => r"\textit{Proceedings of the $\mathit{36}^{th}$ International Conference on Machine Learning}, Long Beach, California, PMLR 97, 2019. Copyright 2019 by the author(s).",
      Some(2021) => r"\textit{Proceedings of the $\mathit{38}^{th}$ International Conference on Machine Learning}, PMLR 139, 2021. Copyright 2021 by the author(s).",
      Some(2022) => r"\textit{Proceedings of the $\mathit{39}^{th}$ International Conference on Machine Learning}, Baltimore, Maryland, USA, PMLR 162, 2022. Copyright 2022 by the author(s).",
      Some(2023) => r"\textit{Proceedings of the $\mathit{40}^{th}$ International Conference on Machine Learning}, Honolulu, Hawaii, USA. PMLR 202, 2023. Copyright 2023 by the author(s).",
      Some(2024) => r"\textit{Proceedings of the $\mathit{41}^{st}$ International Conference on Machine Learning}, Vienna, Austria. PMLR 235, 2024. Copyright 2024 by the author(s).",
      Some(2025) => r"\textit{Proceedings of the $\mathit{42}^{nd}$ International Conference on Machine Learning}, Vancouver, Canada. PMLR 267, 2025. Copyright 2025 by the author(s).",
      Some(2026) => r"\textit{Proceedings of the $\mathit{43}^{rd}$ International Conference on Machine Learning}, Seoul, South Korea. PMLR 306, 2026. Copyright 2026 by the author(s).",
      _ => "",
    };
    proceedings.to_string()
  });
  let review = copy_definition("Notice@String").unwrap_or_else(|| {
    r"Preliminary work.  Under review by the International Conference on Machine Learning (ICML)\@.  Do not distribute."
      .to_string()
  });
  let preprint = copy_definition("ICML@preprint").unwrap_or_else(|| r"\textit{Preprint. \today.}".to_string());
  RawTeX!(&s!("\\providecommand{{\\Notice@String}}{{{review}}}
\\providecommand{{\\ICML@appearing}}{{{appearing}}}
\\providecommand{{\\ICML@preprint}}{{{preprint}}}
\\newif\\ificmlshowauthors \\icmlshowauthorsfalse
\\newif\\ificml@noticeprinted \\icml@noticeprintedfalse"));
  let declare_notice_option = |option: &str, flag: &str, by_version: Option<&str>| -> Result<()> {
    if let Some(notice) = option_notice(option, by_version) {
      let set_notice = notice.map(|notice| s!("\\renewcommand{{\\Notice@String}}{{{notice}}}")).unwrap_or_default();
      let code = TokenizeInternal!(TeXString::assembled(s!("{set_notice}\\gdef\\{flag}{{1}}")));
      DeclareOption!(option, code);
    }
    Ok(())
  };
  declare_notice_option("accepted", "isaccepted", Some(r"\ICML@appearing"))?;
  declare_notice_option("preprint", "ispreprint", (year == Some(2026)).then_some(r"\ICML@preprint"))?;
  DeclareOption!("nohyperref", r"\gdef\nohyperref{1}");
  DeclareOption!(None, {});
  ProcessOptions!();
  RawTeX!(r"\ifdefined\isaccepted \icmlshowauthorstrue \fi
\ifdefined\ispreprint \icmlshowauthorstrue \fi");
  Ok(())
}
