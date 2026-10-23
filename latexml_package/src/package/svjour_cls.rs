use crate::prelude::*;

/// The journal options whose `sv<option>.clo` turns `\if@mathematic` off or sets `\vec` to `\ensuremath{\mathbf{#1}}`
/// at the class's end, as every copy of it the census's svjour and svjour3 papers ship has it (svepj.clo: 46 of 53
/// copies leave `\vec` the class's; svepjST.clo 5 of 5, svepj-spec.clo 3, svepjCONF.clo 3, svwebconf.clo 1 set it;
/// 1504.01694's svepjST.clo:213-238, CMBX in its PDF): `(option, \if@mathematic, \vec is \mathbf)`, for a paper that does
/// not ship the file.
const JOURNAL_VECTORS: [(&str, bool, bool); 9] = [
  ("epj", false, false),
  ("epjST", false, true),
  ("epj-spec", false, true),
  ("epjCONF", false, true),
  ("webconf", false, true),
  ("epj-ND", false, false),
  ("jog", false, false),
  ("cmp", false, false),
  ("arma", false, false),
];

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: svjour.cls.ltxml — covers svjour.cls, svjour1.cls, svjour2.cls & svjour3.cls

  // Option handling — declare all known options as no-ops
  for option in [
    "onecollarge", "runningheads", "smartrunhead", "nosmartrunhead",
    "referee", "instindent", "smartand", "nospthms", "deutsch", "francais",
    // natbib
    "natbib",
    // journal style options
    "2epj", "epj", "glov2arxiv", "manmat-mod", "multphys", "aar",
    "epj-mine", "glov2", "matann2", "nummat",
    "ampa", "epjmod", "granma", "matann", "own", "arma",
    "epj_orig", "icps3", "matbio", "probth",
    "ceremade", "epj-spec", "ifip", "matprg", "publmath",
    "cmp", "gc10", "ijodl", "matzei", "tcfd",
    "cmt", "global", "invmat", "multph",
  ].iter() {
    DeclareOption!(*option, None);
  }
  // svjour.cls:78-81 inputs `sv<option>.clo` for any other option; `epjCONF` (EPJ Web of Conferences) is the one
  // papers use. Its svepjCONF.clo:233-235 (2009/4/9 v1, the copy 1010.2389 ships) adds the session title,
  // `\session-title{…}` (the parameter text is `-title#1`), which only its running head `\epjrunnhead` prints: print
  // furniture, so stored and never typeset. The clo's layout (\maketitle, \abstract, \institutename) stays the
  // binding's. Without it `\session` was undefined and "-title…" set as a paragraph (1010.2389, 1010.3788,
  // 1202.0149).
  DeclareOption!("epjCONF", {
    RawTeX!(r"\def\session-title#1{\gdef\@session-title{#1}}
\def\@session-title{Give the exact title of the conference}");
  });

  // svjour3.cls:46-56, :82-87 (1803.06644's copy, v3.2; svjour.cls:39-49, :66-71, the probe's 2003/04/15 copy): the
  // theorem numbering options, which choose how the class's theorem set is numbered (below); `numbook` also numbers the
  // equations, figures and tables within the sections.
  RawTeX!(r"\let\if@envcntreset\iffalse\let\if@envcntsame\iffalse\let\if@envcntsect\iffalse\let\if@numbook\iffalse");
  DeclareOption!("numbook", "\\let\\if@envcntsect\\iftrue\\let\\if@numbook\\iftrue");
  DeclareOption!("envcountreset", "\\let\\if@envcntreset\\iftrue");
  DeclareOption!("envcountsame", "\\let\\if@envcntsame\\iftrue");
  DeclareOption!("envcountsect", "\\let\\if@envcntsect\\iftrue");
  // Other options get passed to article
  DeclareOption!(None, {
    Digest!("\\PassOptionsToClass{\\CurrentOption}{article}")?;
  });

  ProcessOptions!();
  LoadClass!("article");
  RequireResource!("ltx-svjour.css");
  // The class's theorem set (svjour3.cls:885-933, svjour.cls:756-792), numbered as the options say: `{theorem}`
  // within the section ("1.1") under envcountsect, else on its own ("1") and reset by each section under
  // envcountreset; the others on the theorem counter under envcountsame ("Theorem 1, Lemma 2"), within the section
  // under envcountsect, else each on its own counter (reset by each section under envcountreset). sv_support's
  // `\@ifundefined` set (svmult's, `{theorem}` within the section) then skips them. Perl's binding gave every class
  // `{theorem}` within the section whatever the options (sv_support.sty.ltxml:216; svjour.cls.ltxml:24-26 "These could
  // affect numbering"; 1604.08616: "Theorem 3.1" where the PDF prints 1): KNOWN_PERL_ERRORS #1100. A counter reset
  // by each section is declared within it (`\lx@svjour@reset`) and printed alone, "1", so its ids carry the section,
  // `S1.Thmtheorem1`, not an order-dependent `Thmtheorem1a`. The separator between a within-counter's number and its
  // own, `\@thmcountersep`, is the class's (svjour3.cls:804, :887; svjour.cls:677, :757; every census copy of svjour,
  // svjour2 and svjour3): empty, "." only under envcountsect (and numbook), so a paper's own
  // `\newtheorem{x}{X}[section]` prints "11", "A1", as pdflatex (KNOWN_PERL_ERRORS #1111). Guards
  // `perfect_kernel_batch78q::{svjour_theorem_counters, theorem_counter_separator_is_the_class_s}`.
  RawTeX!(r"\if@numbook\renewcommand\thefigure{\thesection.\@arabic\c@figure}\renewcommand\thetable{\thesection.\@arabic\c@table}%
\renewcommand\theequation{\thesection.\@arabic\c@equation}%
\@addtoreset{figure}{section}\@addtoreset{table}{section}\@addtoreset{equation}{section}\fi
\def\@thmcountersep{}\if@envcntsect\def\@thmcountersep{.}\fi
\def\lx@svjour@reset#1#2{\newtheorem{#1}{#2}[section]\expandafter\gdef\csname the#1\endcsname{\arabic{#1}}}
\if@envcntsect\newtheorem{theorem}{Theorem}[section]\else\if@envcntreset\lx@svjour@reset{theorem}{Theorem}\else
  \newtheorem{theorem}{Theorem}\fi\fi
\def\lx@svjour@theorem#1#2{\if@envcntsame\newtheorem{#1}[theorem]{#2}\else\if@envcntsect\newtheorem{#1}{#2}[section]\else
  \if@envcntreset\lx@svjour@reset{#1}{#2}\else\newtheorem{#1}{#2}\fi\fi\fi}
\lx@svjour@theorem{case}{Case}\lx@svjour@theorem{conjecture}{Conjecture}\lx@svjour@theorem{corollary}{Corollary}%
\lx@svjour@theorem{definition}{Definition}\lx@svjour@theorem{example}{Example}\lx@svjour@theorem{exercise}{Exercise}%
\lx@svjour@theorem{lemma}{Lemma}\lx@svjour@theorem{note}{Note}\lx@svjour@theorem{problem}{Problem}%
\lx@svjour@theorem{property}{Property}\lx@svjour@theorem{proposition}{Proposition}\lx@svjour@theorem{question}{Question}%
\lx@svjour@theorem{solution}{Solution}\lx@svjour@theorem{remark}{Remark}");
  RequirePackage!("sv_support");
  // svjour.cls:546-556 (svjour3.cls:631-641): `\vec`, upright bold letters (`\mbox{\boldmath$\mathbf{#1}$}`) or, without
  // `\if@mathematic` (svjour.cls:38, svjour3.cls:45: true), bold italic (`\mbox{\boldmath$#1$}`), not the kernel's arrow
  // accent (KNOWN_PERL_ERRORS #1143) — pdflatex prints "v + ω" in CMBX and CMMIB (2609.05466, 2509.03659, 2207.14807;
  // 28 svjour3 and 25 svjour papers of the census use it). The journal file of an option (svjour.cls:78-81
  // `\input{sv\CurrentOption.clo}`) may turn `\if@mathematic` off, and may set `\vec` itself at the class's end
  // (`\AtEndOfClass{… \def\vec#1{\ensuremath{\mathbf{#1}}}}`, Greek left medium): the paper's copy says which; without
  // one, the census's copies (`JOURNAL_VECTORS`). The older svjour copies set `\ensuremath{\mathbf{#1}}` under
  // `\if@mathematic` (17 of 73: 0710.3184's), Greek left medium: the paper's copy says which.
  RawTeX!(r"\let\if@mathematic\iftrue");
  let plain_mathbf = |text: &str| crate::shipped_class::class_code_contains(text, "\\def\\vec#1{\\ensuremath{\\mathbf{#1}}}");
  let class_mathbf = crate::shipped_class::shipped_class_text("svjour")?.is_some_and(|text| plain_mathbf(&text));
  let mut journal_mathbf = false;
  for option in crate::shipped_class::class_options()? {
    let (mathematic, mathbf) = match crate::shipped_class::shipped_file_text(&s!("sv{option}"), "clo") {
      Some(text) => (
        !crate::shipped_class::class_code_contains(&text, "\\let\\if@mathematic\\iffalse"),
        plain_mathbf(&text),
      ),
      None => JOURNAL_VECTORS
        .iter()
        .find(|(name, ..)| *name == option)
        .map_or((true, false), |&(_, mathematic, mathbf)| (mathematic, mathbf)),
    };
    if !mathematic {
      RawTeX!(r"\let\if@mathematic\iffalse");
    }
    journal_mathbf |= mathbf;
  }
  if journal_mathbf {
    RawTeX!(r"\def\vec#1{\ensuremath{\mathbf{#1}}}");
  } else {
    let upright = if class_mathbf { r"\mathbf" } else { r"\lx@sv@vec@mathbf" };
    raw_tex(&s!(
      r"\if@mathematic\def\vec#1{{\ensuremath{{{upright}{{#1}}}}}}\else\def\vec#1{{\ensuremath{{\lx@sv@vec@boldmath{{#1}}}}}}\fi"
    ))?;
  }
  // svjour.cls:61-62 and svjour3.cls:392-393 (v3.3): `\preprint{<number>}`, the report number `\makeheadbox` sets
  // at the title page's top right (svjour.cls:489-492) — a publication note, as REVTeX's (1801.06113, 1605.08256,
  // 2305.07696).
  DefMacro!("\\preprint{}", "\\lx@add@pubnote[role=preprint]{#1}");
});
