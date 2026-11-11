//! IEEEtran.cls — IEEE Transactions document class
//! Perl: IEEEtran.cls.ltxml — 458 lines
use crate::{
  engine::{
    base_utilities::{
      document_notes, grade_words, names_continue, notes_only_line, prints_nothing,
    },
    latex_constructs::{FontUnit, set_theorem_within_display, symbol_box_size},
  },
  package::caption_sty,
  prelude::*,
};

/// This binding's frontmatter conventions (the observer's, OXIDIZED_DESIGN_DIVERGENCES #837), declared at its load
/// (`declare_frontmatter_conventions`, user ruling 2026-10-28: each binding declares its own):
/// IEEEtran (and the classes this binding serves, ieeeconf.cls:3592, ieeecolor2.cls:3903 the same
/// `\def\thefootnote{}`): the title's `\thanks` print no mark, so a note's link is no name's by its place (2001.02600,
/// 1803.04996, 2504.08644, 2602.13176: a co-author's homepage, a video, data records, four co-authors' ORCIDs given to
/// the last author). `written`: the observer's writer writes IEEEtran's creators (brief 95b, OD #942: 2202.02533,
/// 2401.16729, 1303.4266, 2311.14729, 2405.16358; a paper whose reading it cannot apply keeps today's output)
pub const FRONTMATTER_CONVENTIONS: FmConventions = FmConventions {
  name: "IEEEtran",
  binding: "IEEEtran.cls",
  unmarked_notes: true,
  cite: "IEEEtran.cls:4756 `\\def\\thefootnote{}`; OD #837 S2a.2 r6 (2001.02600, 1803.04996, 2504.08644, 2602.13176)",
  ..FM_KERNEL_CONVENTIONS
};

#[rustfmt::skip]
LoadDefinitions!({
  // (this binding's frontmatter conventions, first: OXIDIZED_DESIGN_DIVERGENCES #837)
  declare_frontmatter_conventions(&FRONTMATTER_CONVENTIONS);
  // IEEEtran.cls L232-260 declares each class-option flag with
  // `\newif\ifCLASSOPTION<name>`, which creates BOTH the `\ifCLASSOPTION<name>`
  // conditional AND the `\CLASSOPTION<name>true`/`false` setters. IEEEtran
  // template papers toggle these directly (e.g. bare_adv.tex L1014
  // `\CLASSOPTIONcaptionsofftrue`). The DeclareOption `\let\ifCLASSOPTION<name>
  // \iftrue` logic below only sets the conditional VALUE — it never creates the
  // setters, and `captionsoff` (etc.) weren't handled at all → undefined.
  // `\newif` them all up front (faithful to IEEEtran.cls; defaults false, as
  // the class does) so the setters exist; the option logic below then refines
  // values. Witness 1810.05731 (`\CLASSOPTIONcaptionsofftrue`).
  RawTeX!(r"\newif\ifCLASSOPTIONcaptionsoff
\newif\ifCLASSOPTIONcompsoc \newif\ifCLASSOPTIONcomsoc \newif\ifCLASSOPTIONconference
\newif\ifCLASSOPTIONdraft \newif\ifCLASSOPTIONdraftcls \newif\ifCLASSOPTIONdraftclsnofoot
\newif\ifCLASSOPTIONfinal \newif\ifCLASSOPTIONjournal \newif\ifCLASSOPTIONnofonttune
\newif\ifCLASSOPTIONonecolumn \newif\ifCLASSOPTIONoneside \newif\ifCLASSOPTIONpeerreview
\newif\ifCLASSOPTIONpeerreviewca \newif\ifCLASSOPTIONromanappendices \newif\ifCLASSOPTIONtechnote
\newif\ifCLASSOPTIONtransmag \newif\ifCLASSOPTIONtwocolumn \newif\ifCLASSOPTIONtwoside");
  // IEEEtran.cls:268-270 (V1.6b) the paper-size flags its `a4paper`/`cspaper`/`letterpaper` options set (:296-320),
  // which a derived class's own `a4paper` handler sets too (ieeeaccess.cls:21-23 `\@IEEEusingAfourpapertrue`, in the
  // paper's copy read after the ieeeaccess binding: 2007.03025, 2009.04554, 2011.07807, 2209.03883, 2209.12464,
  // 2609.20467).
  RawTeX!(r"\newif\if@IEEEusingAfourpaper \newif\if@IEEEusingcspaper");
  // DeclareOption stubs — Perl L18-108
  // IEEEtran.cls:290-293 (`\CLASSOPTIONpt`, read by `\@IEEEabskeysecsize` below; 10pt the default, :374)
  RawTeX!(r"\def\CLASSOPTIONpt{10}");
  DeclareOption!("9pt", r"\def\CLASSOPTIONpt{9}");
  DeclareOption!("10pt", r"\def\CLASSOPTIONpt{10}");
  DeclareOption!("11pt", r"\def\CLASSOPTIONpt{11}");
  DeclareOption!("12pt", r"\def\CLASSOPTIONpt{12}");
  DeclareOption!("letterpaper", r"\@IEEEusingAfourpaperfalse\@IEEEusingcspaperfalse");
  DeclareOption!("a4paper", r"\@IEEEusingAfourpapertrue\@IEEEusingcspaperfalse");
  DeclareOption!("cspaper", r"\@IEEEusingcspapertrue\@IEEEusingAfourpaperfalse");
  DeclareOption!("draft", {});
  DeclareOption!("final", {});
  DeclareOption!("journal", { Let!("\\ifCLASSOPTIONjournal", "\\iftrue"); Let!("\\ifCLASSOPTIONconference", "\\iffalse"); });
  DeclareOption!("conference", { Let!("\\ifCLASSOPTIONjournal", "\\iffalse"); Let!("\\ifCLASSOPTIONconference", "\\iftrue"); });
  DeclareOption!("technote", { Let!("\\ifCLASSOPTIONtechnote", "\\iftrue"); });
  DeclareOption!("nofonttune", {});
  DeclareOption!("captionsoff", {});
  // TL `IEEEtran.cls` L254-255 and L364-366: `comsoc`, `compsoc`,
  // `transmag` are THREE distinct mutually-exclusive options. Each
  // setter flips one to true and clears the other two:
  //   \DeclareOption{comsoc}{\CLASSOPTIONcomsoctrue \CLASSOPTIONcompsocfalse \CLASSOPTIONtransmagfalse}
  //   \DeclareOption{compsoc}{\CLASSOPTIONcomsocfalse \CLASSOPTIONcompsoctrue \CLASSOPTIONtransmagfalse}
  //   \DeclareOption{transmag}{\CLASSOPTIONcomsocfalse \CLASSOPTIONcompsocfalse \CLASSOPTIONtransmagtrue}
  // Perl `IEEEtran.cls.ltxml:103` for `comsoc` additionally
  // `\RequirePackage{newtxmath}` so `\coloneqq` / `\bigstar` / TX-math
  // symbol family used by comsoc papers resolves cleanly (Perl-faithful;
  // earlier Rust loaded only amssymb here, missing `\coloneqq` —
  // witnesses 1902.10910, 2201.11831).
  DeclareOption!("comsoc", {
    Let!("\\ifCLASSOPTIONcomsoc",  "\\iftrue");
    Let!("\\ifCLASSOPTIONcompsoc", "\\iffalse");
    Let!("\\ifCLASSOPTIONtransmag","\\iffalse");
    RequirePackage!("newtxmath");
  });
  DeclareOption!("compsoc", {
    Let!("\\ifCLASSOPTIONcomsoc",  "\\iffalse");
    Let!("\\ifCLASSOPTIONcompsoc", "\\iftrue");
    Let!("\\ifCLASSOPTIONtransmag","\\iffalse");
  });
  DeclareOption!("transmag", {
    Let!("\\ifCLASSOPTIONcomsoc",  "\\iffalse");
    Let!("\\ifCLASSOPTIONcompsoc", "\\iffalse");
    Let!("\\ifCLASSOPTIONtransmag","\\iftrue");
  });
  DeclareOption!("romanappendices", { Let!("\\ifCLASSOPTIONromanappendices", "\\iftrue"); });
  // Perl IEEEtran.cls.ltxml L72-73: the column-mode options flip BOTH flags.
  // Default is twocolumn (set in the Let block below, matching Perl L19-20's
  // `\CLASSOPTIONtwocolumntrue`); the `onecolumn` option must flip it OFF.
  // These previously were empty no-ops, so `\ifCLASSOPTIONtwocolumn` stayed
  // hardcoded-true even for `\documentclass[onecolumn]{IEEEtran}` — a paper's
  // `\ifCLASSOPTIONtwocolumn \begin{strip}…\end{strip} \else …\fi` then took
  // the twocolumn branch and the `cuted` `strip` env errored "Not in outer par
  // mode". Witness 1508.02556. The handlers run during ProcessOptions (after
  // the default Lets), so the flip survives.
  DeclareOption!("onecolumn", {
    Let!("\\ifCLASSOPTIONonecolumn", "\\iftrue");
    Let!("\\ifCLASSOPTIONtwocolumn", "\\iffalse");
  });
  DeclareOption!("twocolumn", {
    Let!("\\ifCLASSOPTIONtwocolumn", "\\iftrue");
    Let!("\\ifCLASSOPTIONonecolumn", "\\iffalse");
  });
  // Perl L95-99: peerreview/peerreviewca set their flag and clear the
  // mutually-exclusive journal/conference/technote modes.
  DeclareOption!("peerreview", {
    Let!("\\ifCLASSOPTIONpeerreview",   "\\iftrue");
    Let!("\\ifCLASSOPTIONpeerreviewca", "\\iffalse");
    Let!("\\ifCLASSOPTIONjournal",      "\\iffalse");
    Let!("\\ifCLASSOPTIONconference",   "\\iffalse");
    Let!("\\ifCLASSOPTIONtechnote",     "\\iffalse");
  });
  DeclareOption!("peerreviewca", {
    Let!("\\ifCLASSOPTIONpeerreview",   "\\iftrue");
    Let!("\\ifCLASSOPTIONpeerreviewca", "\\iftrue");
    Let!("\\ifCLASSOPTIONjournal",      "\\iffalse");
    Let!("\\ifCLASSOPTIONconference",   "\\iffalse");
    Let!("\\ifCLASSOPTIONtechnote",     "\\iffalse");
  });
  // Option conditionals — Perl L18-108. These are the FALSE defaults
  // (mirroring `\newif\if@CLASSOPTIONcompsoc \@CLASSOPTIONcompsocfalse`).
  // MUST come BEFORE ProcessOptions so the option-handler `\let` flips to
  // `\iftrue` survive — the previous order placed these after ProcessOptions
  // and silently clobbered any positive option flag the user passed (driver
  // 2308.01854 `\documentclass[10pt,journal,compsoc]{IEEEtran}` had
  // \ifCLASSOPTIONcompsoc unexpectedly false → user's
  // `\ifCLASSOPTIONcompsoc \usepackage{url} \fi` skipped → \url undefined).
  // TL `IEEEtran.cls` L254-256: three separate `\newif` flags for the
  // mutually-exclusive comsoc/compsoc/transmag options. Pre-bind all
  // three to false so paper-side `\ifCLASSOPTION* … \fi` doesn't see
  // undefined when none of the options is passed. Witnesses:
  // arXiv:2603.07560 (`comsoc`-style probe), 2308.01854 (`compsoc`),
  // older IEEEtran/transmag papers.
  Let!("\\ifCLASSOPTIONcomsoc",   "\\iffalse");
  Let!("\\ifCLASSOPTIONcompsoc",  "\\iffalse");
  Let!("\\ifCLASSOPTIONtransmag", "\\iffalse");
  Let!("\\ifCLASSOPTIONjournal", "\\iftrue");
  Let!("\\ifCLASSOPTIONconference", "\\iffalse");
  Let!("\\ifCLASSOPTIONtechnote", "\\iffalse");
  Let!("\\ifCLASSOPTIONromanappendices", "\\iffalse");
  // IEEEtran.cls:264, 552-557: `\ifCLASSINFOpdf` is true in PDF output only (`\pdfoutput` non-zero), so a source
  // that ships EPS figures (`\pdfoutput` 0 then, the K6 ruling) takes its preamble's
  // `\ifCLASSINFOpdf…\else\usepackage[dvips]{graphicx}\fi` branch (22 run-329 papers had `\includegraphics`
  // undefined, 0902.1911). Perl sets it true always (IEEEtran.cls.ltxml:38-40).
  Digest!("\\newif\\ifCLASSINFOpdf\\ifcase\\pdfoutput\\else\\global\\CLASSINFOpdftrue\\fi")?;
  Let!("\\ifCLASSOPTIONonecolumn", "\\iffalse");
  Let!("\\ifCLASSOPTIONtwocolumn", "\\iftrue");
  Let!("\\ifCLASSOPTIONdraftcls", "\\iffalse");
  Let!("\\ifCLASSOPTIONpeerreview", "\\iffalse");
  Let!("\\ifCLASSOPTIONcaptionsoff", "\\iffalse");
  // TL `IEEEtran.cls` L238-244: the `draft`/`final`, `oneside`/`twoside`,
  // `peerreviewca`, `nofonttune`, `draftclsnofoot` `\newif` flags. User
  // code in IEEEtran papers freely reads `\ifCLASSOPTIONdraft` /
  // `\ifCLASSOPTIONoneside` without first checking they're defined —
  // e.g. arXiv:2509.12142 has a sectionhead probe
  // `\ifCLASSOPTIONdraft Draft\fi`. Pre-bind to the same defaults as
  // the real class so undefined-cond auto-define doesn't spam an Error.
  Let!("\\ifCLASSOPTIONdraft", "\\iffalse");
  Let!("\\ifCLASSOPTIONdraftclsnofoot", "\\iffalse");
  Let!("\\ifCLASSOPTIONfinal", "\\iftrue");
  Let!("\\ifCLASSOPTIONoneside", "\\iftrue");
  Let!("\\ifCLASSOPTIONtwoside", "\\iffalse");
  Let!("\\ifCLASSOPTIONpeerreviewca", "\\iffalse");
  Let!("\\ifCLASSOPTIONnofonttune", "\\iffalse");

  ProcessOptions!();

  // Load article as base
  LoadClass!("article");
  // The page a float's panels are set in rows against (the panel grid rule, `declare_panel_page`): IEEEtran.cls's
  // (:1734-1811), as pdflatex gives it — 516pt (43pc) in two 252pt columns (`\columnsep` 1pc); compsoc's 7in + 12bp,
  // 517.935pt in two of 252.945pt (12bp apart), and with `conference` the paper less 1.5in (`\IEEEsettextwidth{0.75in}
  // {0.75in}`: 505.89pt on letter paper, in two of 243.911pt 0.25in apart); under a draft option the paper less 2in
  // (469.755pt on letter, 452.968pt on A4, 460.721pt on cspaper less 1.5in), its columns the mode's `\columnsep` apart —
  // set in one column under `onecolumn`/`peerreview`, whose `\if@twocolumn` this binding keeps true (below). The class's
  // `\ProcessOptions` (:376) runs the options in their declaration order. Witnesses 2211.03584, 1604.07648, 1404.3152,
  // 1809.02562, 2103.06618 (`onecolumn`), 1010.0060 (`conference`, absolute widths against the 252pt column); review
  // rev78x2 finding 4 (`[conference,compsoc]`, `draftcls` with `a4paper`).
  {
    let paper = match last_declared_class_option(&["letterpaper", "a4paper", "cspaper"]) {
      Some("a4paper") => 597.50787,
      Some("cspaper") => 569.12625,
      _ => 614.295,
    };
    let draft = matches!(
      last_declared_class_option(&["draft", "draftcls", "draftclsnofoot", "final"]),
      Some("draft" | "draftcls" | "draftclsnofoot")
    );
    let compsoc = last_declared_class_option(&["comsoc", "compsoc", "transmag"]) == Some("compsoc");
    let mode = last_declared_class_option(&["journal", "conference", "technote", "peerreview", "peerreviewca"]);
    let conference = mode == Some("conference");
    let columnsep = match (compsoc, conference) {
      (true, true) => 18.0675,
      (true, false) => 12.045,
      _ => 12.0,
    };
    let text = if draft {
      paper - 144.54
    } else if compsoc && conference {
      paper - 108.405
    } else if compsoc {
      517.935
    } else {
      516.0
    };
    let one_column = last_declared_class_option(&["onecolumn", "twocolumn"]) == Some("onecolumn")
      || matches!(mode, Some("peerreview" | "peerreviewca"));
    declare_panel_page((text - columnsep) / 2.0, text, !one_column);
  }

  // `\@ptsize` as the size options set it (IEEEtran.cls:290-293), which article's own setting replaced.
  RawTeX!(r"\ifnum\CLASSOPTIONpt=11 \def\@ptsize{1}\fi\ifnum\CLASSOPTIONpt=12 \def\@ptsize{2}\fi");
  // Class internals a document's own `\abstract`/`\IEEEkeywords` (or "Note to Practitioners") redefinition
  // copies from IEEEtran.cls (2609.05249, 07516, 12903, 13235, 16206, 36487): the abstract/keywords size
  // (:5263-5270) and the leading `\par`/`\\`/space gobbler (:5357-5379), verbatim.
  RawTeX!(r"\def\@IEEEptsizenine{9}
\def\@IEEEabskeysecsize{\small}
\ifx\CLASSOPTIONpt\@IEEEptsizenine\def\@IEEEabskeysecsize{\footnotesize}\fi
\ifCLASSOPTIONcompsoc\def\@IEEEabskeysecsize{\footnotesize}\ifCLASSOPTIONconference\def\@IEEEabskeysecsize{\small}\fi\fi
\long\def\@IEEEgobbleleadPARNLSP#1{\let\@IEEEswallowthistoken=0%
\let\@IEEEgobbleleadPARNLSPtoken#1%
\let\@IEEEgobbleleadPARtoken=\par%
\let\@IEEEgobbleleadNLtoken=\\%
\let\@IEEEgobbleleadSPtoken=\ %
\def\@IEEEgobbleleadSPMACRO{\ }%
\ifx\@IEEEgobbleleadPARNLSPtoken\@IEEEgobbleleadPARtoken%
\let\@IEEEswallowthistoken=1%
\fi%
\ifx\@IEEEgobbleleadPARNLSPtoken\@IEEEgobbleleadNLtoken%
\let\@IEEEswallowthistoken=1%
\fi%
\ifx\@IEEEgobbleleadPARNLSPtoken\@IEEEgobbleleadSPtoken%
\let\@IEEEswallowthistoken=1%
\fi%
\ifx\@IEEEgobbleleadPARNLSPtoken\@IEEEgobbleleadSPMACRO%
\let\@IEEEswallowthistoken=1%
\fi%
\ifx\@IEEEswallowthistoken 1\let\@IEEEnextgobbleleadPARNLSP=\@IEEEgobbleleadPARNLSP\else%
\let\@IEEEnextgobbleleadPARNLSP=#1\fi%
\@IEEEnextgobbleleadPARNLSP}");

  // Real IEEEtran.cls L689 `\newif\if@technote \@technotefalse` — private flag
  // (separate from the public `\ifCLASSOPTION*` mirrors). User code in
  // technote-aware paragraphs (e.g. cs0502037 `\def\endkeywords{\if@technote
  // \vspace{1.34ex}\else\vspace{0.67ex}\fi}`) reads `\if@technote` directly.
  // Pre-define so `\if@technote` doesn't trip the "undefined cond"
  // auto-define-as-iffalse path (works but emits an Error).
  Let!("\\if@technote", "\\iffalse");
  Let!("\\if@confmode", "\\iffalse");
  // IEEEtran journal default is two-column. Real IEEEtran.cls invokes
  // `\twocolumn` (in journal mode) which sets the LaTeX kernel
  // `\@twocolumntrue`. Mirror by setting `\if@twocolumn` to `\iftrue`,
  // even under `onecolumn` (not honored; honoring it would reach a
  // document `\IEEEkeywords` that opens `\quotation`, RED repro
  // sectioning-frontmatter/ieeekeywords_redefined_with_quotation_in_one_column). Witness: cs0502037
  // user-installed `\def\endkeywords{\if@twocolumn\else\endquotation\fi}`
  // wants the if-true branch (two-column → no `\endquotation`); without
  // this, the `\else \endquotation` branch fires from `\keywords` opening
  // `\quotation` (article default `\if@twocolumn=false`), producing an
  // unmatched `\if`/`\fi` cascade by EOF.
  Let!("\\if@twocolumn", "\\iftrue");

  // Front matter macros (Perl L134-165)
  DefMacro!("\\IEEEtitleabstractindextext{}", "#1");

  // \thetitle / \theauthor / \thedate — titling.sty-style accessors that some
  // IEEEtran preambles or .bbl files reference. IEEEtran doesn't natively
  // export them, but users assume they exist. Provide empty defaults so
  // bibliographies that include `\thetitle` don't crash. Witness 2501.15830.
  def_macro_noop("\\thetitle")?;
  def_macro_noop("\\theauthor")?;
  def_macro_noop("\\thedate")?;
  def_macro_noop("\\IEEEdisplaynontitleabstractindextext")?;
  def_macro_noop("\\IEEEdisplaynotcompsoctitleabstractindextext")?;
  def_macro_noop("\\IEEEcompsoctitleabstractindextext")?;
  // IEEEtran.cls:4920-4932: the repeated title page of the peer-review modes; any other mode defines it `\relax`, so a
  // call before `\title` (1909.01874's journal paper) makes no title page there, and the title stays the
  // `\maketitle`'s (Perl IEEEtran.cls.ltxml:135 lets it to `\maketitle` in every mode, which flushed the frontmatter
  // before the title was given: no `<ltx:title>`; OXIDIZED_DESIGN_DIVERGENCES #950).
  RawTeX!(r"\ifCLASSOPTIONpeerreview\let\IEEEpeerreviewmaketitle\maketitle\else\def\IEEEpeerreviewmaketitle{\relax}\fi");
  // IEEEtran.cls:6278-6288 lets the conference-mode lock-outs go (`\IEEEmembership` prints again).
  DefMacro!("\\IEEEoverridecommandlockouts", "\\global\\let\\iflx@IEEE@lockouts@overridden\\iftrue");
  RawTeX!(r"\newif\iflx@IEEE@lockouts@overridden");
  def_macro_noop("\\overrideIEEEmargins")?;
  // \IEEEaftertitletext{text} — after-title note (often invited-
  // paper credit, conference name). Author content; preserve as
  // ltx:note frontmatter rather than gobble.
  DefMacro!("\\IEEEaftertitletext{}",
    "\\lx@add@frontmatter{ltx:note}[role=aftertitle]{#1}");
  // \IEEEspecialpapernotice{text} — e.g. "Invited Paper". Author
  // content; preserve.
  DefMacro!("\\IEEEspecialpapernotice{}",
    "\\lx@add@frontmatter{ltx:note}[role=papernotice]{#1}");
  // \IEEEmembership{text} — membership grade after an author name (e.g.
  // "Senior Member, IEEE"). Perl drops it (''). An earlier Rust rendering,
  // ",\space\textit{#1}", injected a top-level comma INTO the `\author` list;
  // the author-splitter (`\lx@add@authors`) then split that comma and turned
  // the membership grade into a phantom "Senior Member, IEEE" creator (witness
  // 2508.00603, whose flat `\author{A, B, \IEEEmembership{...}, and C}` yielded
  // 6 creators). In an author's own name line — before the first `\\` of each
  // `\and` group, or in an `\IEEEauthorblockN` — `\author` now moves each grade
  // in front of the comma before it (`name_memberships`) and makes it that
  // author's unlabelled `membership` contact (`\lx@IEEE@membership`, markup a
  // stylesheet shows or hides): "Yong Man Ro, Senior Member, IEEE" (2306.15457,
  // 2201.01230, 2408.09035; 63l). Anywhere else (a biography heading, a `\thanks`,
  // names misread as an affiliation line) whose author it names cannot be
  // known, so it stays dropped there, as in Perl (2408.01902). Conference mode
  // locks the grade out (IEEEtran.cls:6270 swallows it; pdflatex prints none)
  // unless `\IEEEoverridecommandlockouts` lets it back in.
  DefMacro!("\\IEEEmembership{}", "\\lx@IEEE@membership@drop{#1}");
  DefMacro!("\\lx@IEEE@membership@drop{}", "");
  DefMacro!("\\lx@IEEE@membership{}",
    "\\ifCLASSOPTIONconference\\iflx@IEEE@lockouts@overridden\\lx@add@contact[role=membership]{#1}\\fi\\else\\lx@add@contact[role=membership]{#1}\\fi");
  DefMacro!("\\lx@contact@membership@name", "");
  // `\author{\IEEEauthorblockN{Name}\IEEEauthorblockA{Affil}\and …}` is the
  // conference-mode author block. Perl maps `\IEEEauthorblockN{}`→`#1` (plain
  // text) and relies on `\author`→`\lx@add@authors` to split; but our beyond-Perl
  // author-splitter (OXIDIZED_DESIGN #52) misreads the ordinal `1\textsuperscript
  // {st}` prefixes as affiliation markers and drops every author (witness
  // 2602.05517 → 0 creators). Because the author block carries EXPLICIT
  // structure, bind each `\IEEEauthorblockN` to a creator directly and each
  // `\IEEEauthorblockA` to that creator's affiliation, then have `\author` run
  // the block body verbatim (below) instead of guessing.
  // A block's names may be a list (`\IEEEauthorblockN{Clemens Paul Zengler\IEEEauthorrefmark{1}, Niels Troldborg, Mac
  // Gaunaa.}`, 2410.19527): read as an author block, one author each, and a `\\` line under a name its affiliation
  // (`\IEEEauthorblockN{Ann Able\\ University of X, City}`); the block's `\IEEEauthorblockA` goes to its last author
  // and the run of authors before it that have none (`annotate=new`; 63e).
  DefMacro!("\\IEEEauthorblockN{}", "\\lx@add@authors@append{#1}");
  // (an affiliation block led by an author reference mark is the affiliation of the authors showing that mark,
  // `\IEEEauthorblockA{\IEEEauthorrefmark{1}Univ A}`; 2406.11437, 2104.02493)
  DefMacro!("\\IEEEauthorblockA{}", "\\lx@add@affiliation@marked[annotate=new]{#1}");
  // Run an explicit author-block body: `\and` merely separates blocks here (each
  // block already emits its own creator), so neutralise it.
  // (the last `\\author` wins, as in IEEEtran.cls's `\\def\\@author` and Perl's `\\lx@add@authors`: this body adds its authors one by
  // one, so what an earlier call added goes first. 2606.16069's PDF prints its second call's four names. A plain call is such a body
  // too: `\\lx@add@authors` takes back a creator, not the contacts a marked call queued flat)
  DefPrimitive!(T_CS!("\\lx@IEEE@author@begin"), None, { begin_author_body(); });
  DefPrimitive!(T_CS!("\\lx@IEEE@author@end"), None, { end_author_body(); });
  DefMacro!("\\lx@IEEE@author@blocks{}",
    "\\lx@IEEE@author@begin\\def\\@author{#1}\\begingroup\\def\\and{}#1\\endgroup\\lx@IEEE@author@end");
  DefMacro!("\\lx@IEEE@author@plain{}",
    "\\lx@IEEE@author@begin\\def\\@shortauthor{}\\gdef\\shortauthor{}\\def\\@author{#1}\\lx@add@authors{#1}\\lx@IEEE@author@end");
  // Override `\author`: dispatch to explicit-block handling when the body uses
  // `\IEEEauthorblockN`, else to the standard name-splitter. `\author` is locked
  // in the kernel; re-lock so a user `\renewcommand` can't shadow this.
  // (the body as its document macros print it first, a block held in one, a wrapper's `\and` or `\\`: `blocks_as_printed`,
  // OXIDIZED_DESIGN_DIVERGENCES #128, P2-8a)
  DefMacro!("\\author[]{}", sub[(_short, body)] {
    // (a document's own `\IEEEmembership` keeps its meaning: `\def\IEEEmembership#1{\textit{#1}}` in 2408.00647)
    let ours = class_membership()?;
    // (the author structure a document macro holds, read as it means before the route: `blocks_as_printed`)
    let body = with_block_names(blocks_as_printed(body)?);
    // (a reference mark among other marks is the symbol it prints: 2312.10807, OXIDIZED_DESIGN_DIVERGENCES #968)
    let body = printed_refmarks(body);
    let (target, body) = if calls_names_block(&body) {
      (T_CS!("\\lx@IEEE@author@blocks"), author_block_body(body, ours)?)
    } else {
      (T_CS!("\\lx@IEEE@author@plain"), if ours { Tokens::new(name_memberships(body.unlist())) } else { body })
    };
    Ok(Invocation!(target, vec![Some(body)]))
  }, locked => true);
  // authblk's `\author` (a document's `\usepackage{authblk}`; the IEEEtaes and IEEEoj stubs load it) replaces the one
  // above, so its names reach the frontmatter with each grade still `\IEEEmembership`, dropped
  // (`\lx@IEEE@membership@drop`): every author's printed grade lost (1803.04687, 2202.10869, 2208.04191, 2307.05035,
  // 2505.19506, 2506.05637; Perl the same, IEEEtran.cls.ltxml:143). Once authblk is loaded, its `\author` reads the
  // grades of its names as this one does (`name_memberships`, `block_name_memberships`) and then adds the authors as
  // authblk does — conference mode still swallowing them (`\lx@IEEE@membership`; OXIDIZED_DESIGN_DIVERGENCES #950).
  // A body of author blocks (`\author{\IEEEauthorblockN{Ann Able,~\IEEEmembership{Member,~IEEE}, Bob Baker and Cy
  // Coe}\thanks{…}}`; 2402.16140, 1611.02001, 1708.05281, 1810.08035, 2310.18257, 2409.09417) is read as the class's
  // own `\author` reads it (`printed_refmarks`, `author_block_body`; `\IEEEauthorblockN` one author each), its authors
  // added after those before them as authblk adds every `\author`'s (authblk.sty:114-145; the class's
  // `\lx@IEEE@author@blocks` replaces them), then authblk's bookkeeping (`\lx@AB@author@state`, `\lx@author@after`,
  // authblk_sty.rs). Through authblk's own call the block was one creator around the nested `\IEEEauthorblockN` call,
  // which printed nothing there: every author lost without a Warning. Perl's `\IEEEauthorblockN` is `#1`
  // (IEEEtran.cls.ltxml:144), its authblk `\author` one creator holding every name (authblk.sty.ltxml:55); pdflatex
  // prints each name. A labelled call gives each name the label (`\lx@IEEE@authorblockN@labelled`), as authblk's
  // `\author[1]{Ann Able, Bob Baker}` does. OXIDIZED_DESIGN_DIVERGENCES #987.
  DefMacro!("\\lx@IEEE@authblk@blocks{}", "\\begingroup\\def\\and{}#1\\endgroup");
  DefMacro!("\\lx@IEEE@authorblockN@labelled{}{}", "\\lx@add@authors@append[annotations={#1}]{#2}");
  DefMacro!("\\lx@IEEE@authblk@author[]{}", sub[(label, body)] {
    let ours = class_membership()?;
    let body = with_block_names(blocks_as_printed(body)?);
    if calls_names_block(&body) {
      let label_tokens = label.clone().unwrap_or_default();
      let mut blocks = author_block_body(printed_refmarks(body.clone()), ours)?.unlist();
      if label.is_some() {
        let blockn = T_CS!("\\IEEEauthorblockN");
        blocks = blocks
          .into_iter()
          .flat_map(|t| {
            if t == blockn {
              let mut call = vec![T_CS!("\\lx@IEEE@authorblockN@labelled"), T_BEGIN!()];
              call.extend(label_tokens.unlist_ref().iter().copied());
              call.push(T_END!());
              call
            } else {
              vec![t]
            }
          })
          .collect();
      }
      let mut call = vec![T_CS!("\\lx@IEEE@authblk@blocks"), T_BEGIN!()];
      call.extend(blocks);
      call.extend([T_END!(), T_CS!("\\lx@AB@author@state"), T_BEGIN!()]);
      call.extend(label_tokens.unlist_ref().iter().copied());
      call.extend([T_END!(), T_BEGIN!()]);
      call.extend(body.unlist_ref().iter().copied());
      call.extend([T_END!(), T_CS!("\\lx@author@after"), T_BEGIN!()]);
      call.extend(body.unlist_ref().iter().copied());
      call.push(T_END!());
      return Ok(Tokens::new(call));
    }
    let body = if ours { Tokens::new(name_memberships(body.unlist())) } else { body };
    // (its optional label exactly as written, an empty one too: authblk's `\author[]{…}` is no unlabelled `\author`,
    // 2005.04149, 2105.12254)
    let mut call = vec![T_CS!("\\lx@IEEE@AB@author")];
    if let Some(label) = label {
      call.push(T_OTHER!("["));
      call.extend(label.unlist());
      call.push(T_OTHER!("]"));
    }
    call.push(T_BEGIN!());
    call.extend(body.unlist());
    call.push(T_END!());
    Ok(Tokens::new(call))
  });
  RawTeX!(r"\AddToHook{package/authblk/after}{\global\let\lx@IEEE@AB@author\author\global\let\author\lx@IEEE@authblk@author}");

  // IEEEkeywords environment (Perl PR #2767: digestion-based capture). Each block is printed where it stands
  // (IEEEtran.cls:5286-5292 sets "Index Terms—" and the body in place), so a paper's several blocks are all its keywords
  // (`accumulate`, as the VTeX `{keyword}` blocks, ims_support_sty.rs): Perl's `\lx@begin@keywords` clears the keywords
  // before it (Base_Utility.pool.ltxml:626-628), so only the last block was kept (repro
  // sectioning-frontmatter/ieeekeywords_blocks_are_all_kept; KNOWN_PERL_ERRORS #1020).
  DefMacro!(
    "\\IEEEkeywords",
    "\\lx@add@frontmatter@until{ltx:keywords}[accumulate=true,name={\\IEEEkeywordsname:~}]{\\lx@end@keywords}"
  );
  DefMacro!("\\endIEEEkeywords", "\\lx@end@keywords");
  // Perl IEEEtran.cls.ltxml L152-153 (pre-PR): explicit env-token aliases.
  // Kept so user `\def\endIEEEkeywords` can't break env routing. Drivers:
  // 2007.13436, 1812.09324.
  // With LaTeX's group, as `\begin`/`\end` give it: a document's own `\IEEEkeywords` setting the class's
  // `\@IEEEabskeysecsize\bfseries` kept its fonts to the end of the document (2609.12903, 13235, 16206). And with its
  // `\@currenvir` (latex.ltx `\begin`), which `\end{document}` checks: one left open is LaTeX's error naming it.
  DefMacro!(T_CS!("\\begin{IEEEkeywords}"), None, "\\begingroup\\lx@setcurrenvir{IEEEkeywords}\\IEEEkeywords");
  DefMacro!(T_CS!("\\end{IEEEkeywords}"),   None, "\\endIEEEkeywords\\endgroup");

  // IEEEtai (IEEE Trans. AI) IEEEImpStatement environment — author
  // content is the journal-mandated "Impact Statement" preceding the
  // main body. Render as a labeled section so the substantive prose
  // is preserved instead of bouncing as undefined. Witness 2305.09145.
  DefEnvironment!("{IEEEImpStatement}",
    "<ltx:note role='impactstatement' name='Impact Statement'>#body</ltx:note>",
    mode => "internal_vertical");

  DefMacro!("\\IEEEraisesectionheading{}", "#1");
  DefMacro!("\\IEEEPARstart{}{}", "#1#2");
  DefMacro!("\\IEEEcompsocitemizethanks{}", "\\thanks{#1}");
  def_macro_noop("\\IEEEcompsocthanksitem[]")?;
  // \IEEEauthorrefmark{n} — a superscript affiliation/footnote marker after an
  // author name. Perl binds `\IEEEauthorrefmark` with NO argument (''), so a
  // real `\IEEEauthorrefmark{2}` leaks its `{2}` as literal text ("Lei Chen2",
  // witness 2404.07847). Take the argument and print it as IEEEtran.cls:4540-4546 does (beyond-Perl): the n-th
  // footnote symbol, raised — `* † ‡ § ¶ ‖ ** †† ‡‡`, then lower-case roman numerals from 10 —, the number itself
  // only under `transmag` (:4541): "Lei Chen†" (2312.10807's `Oier Mees$^3$\IEEEauthorrefmark{1}` prints "Oier
  // Mees3∗"). The symbol is declared to the kernel's hook (`\lx@author@refmark@symbol`, author_macros.rs), whose
  // label readers key the mark by its number, one to one with the symbols the class's marks print, so a block marked
  // with them alone pairs as it did; a block mixing them with other marks is read by the symbols
  // (`printed_refmarks`, below; OXIDIZED_DESIGN_DIVERGENCES #968). Without IEEEtran's `\raisebox{0pt}[0pt][0pt]` (a
  // zero-size box in the name) and its `\footnotesize`; the symbols in their text forms, as `\@fnsymbol`'s
  // `\TextOrMath` sets them in text.
  RawTeX!(r"\def\lx@author@refmark@symbol#1{\ifCLASSOPTIONtransmag #1\else\ifcase#1\or
\textasteriskcentered\or\textdagger\or\textdaggerdbl\or\textsection\or\textparagraph\or\textbardbl\or
\textasteriskcentered\textasteriskcentered\or\textdagger\textdagger\or\textdaggerdbl\textdaggerdbl
\else\romannumeral#1\fi\fi}");
  DefMacro!("\\IEEEauthorrefmark{}", "\\textsuperscript{\\lx@author@refmark@printed{#1}}");
  def_macro_noop("\\IEEEtriggeratref{}")?;
  // IEEEtran.cls:6153 `\def\IEEEpubid#1{\def\@IEEEpubid{#1}…}`: the publisher's ID line at the first page's foot, the
  // argument alone — no label, and no "pubid: " before it (Perl's IEEEtran.cls.ltxml:157 content)
  // (FIDELITY_CHARTER §4, user ruling 2026-10-30; OXIDIZED_DESIGN_DIVERGENCES #921).
  DefMacro!("\\IEEEpubid{}", "\\lx@add@pubnote[role=pubid,name={}]{#1}");
  def_macro_noop("\\IEEEpubidadjcol")?;
  // \corresp{name} — IEEE Open Journal class IEEEoj.cls L4875 marks
  // a corresponding-author note (later rendered in titlepage). Preserve
  // as frontmatter note. Witness 2307.02076 (IEEEoj).
  DefMacro!("\\corresp{}",
    "\\lx@add@frontmatter{ltx:note}[role=corresponding]{#1}");

  // Section numbering (IEEEtran.cls:2545-2556, Perl IEEEtran.cls.ltxml:160-174, after the options): compsoc is all
  // arabic ("1.1", which its heading forms `\the<level>dis` print, :2570-2581), the other modes Roman ("I-A"). A
  // Roman-only port numbered compsoc "I.1" (rev78c probe ie_compsoc; 63 of 558 IEEEtran papers use compsoc). Guard
  // `perfect_kernel_batch78c::ieeetran_headings_print_their_heading_forms`.
  RawTeX!(r"\ifCLASSOPTIONcompsoc
\def\thesection{\arabic{section}}
\def\thesubsection{\thesection.\arabic{subsection}}
\def\thesubsubsection{\thesubsection.\arabic{subsubsection}}
\def\theparagraph{\thesubsubsection.\arabic{paragraph}}
\else
\def\thesection{\Roman{section}}
\def\thesubsection{\mbox{\thesection-\Alph{subsection}}}
\def\thesubsubsection{\thesubsection\arabic{subsubsection}}
\def\theparagraph{\thesubsubsection\alph{paragraph}}
\fi");

  // Font primitives (Perl L183-186)
  DefPrimitive!("\\ltx@ieeetran@it", None, font => { shape => "italic", family => "serif", series => "medium" }, locked => true);
  DefPrimitive!("\\ltx@ieeetran@sc", None, font => { shape => "smallcaps", family => "serif", series => "medium" }, locked => true);
  DefMacro!("\\format@title@font@section", "\\ltx@ieeetran@sc");
  DefMacro!("\\format@title@font@subsection", "\\ltx@ieeetran@it");
  // IEEEtran.cls:2607-2609 the names, compsoc conferences' figures "Figure"; :2826-2831 compsoc's tables arabic
  // ("TABLE 1"), the other modes' Roman ("TABLE I").
  DefMacro!("\\figurename", "Fig.");
  DefMacro!("\\tablename", "TABLE");
  RawTeX!(r"\ifCLASSOPTIONcompsoc\ifCLASSOPTIONconference\def\figurename{Figure}\fi\def\thetable{\arabic{table}}
\else\def\thetable{\Roman{table}}\fi");
  // IEEEtran.cls:2731-2797 `\@makecaption`, by mode: a table's label over its text — compsoc conferences "TABLE 1.
  // text" on one line — and every other caption "Fig. 1. text" (`\lx@makecaption`, latex_constructs_rust_only.rs: a
  // newfloat type's and a listing's too, "Listing 1. text"; the traditional form's two `\nobreakspace` one space here).
  // The names and separators only: the class's caption fonts — the table text's `\scshape`, compsoc journals' `\sffamily`
  // for every caption, the `\footnotesize` — are set under the class caption fonts knob alone (caption_sty.rs
  // `CLASS_CAPTION_FONTS`, off by default; OXIDIZED_DESIGN_DIVERGENCES #884). Installed unless the document loads
  // caption, which replaces it ("Fig. 1: text", "TABLE I: text": caption_sty.rs `class_makecaption`); a float.sty type
  // keeps float.sty's "Program 1: text" (float_sty.rs), as a `\restylefloat`ed table does ("TABLE I: text", 2006.08364),
  // longtable its "TABLE I: text" (longtable_sty.rs). The `captionsoff` option's caption-less floats are not ported: the
  // captions are kept. Perl's binding keeps the kernel's ": " for every type (IEEEtran.cls.ltxml; KNOWN_PERL_ERRORS
  // #1140). Guards `perfect_kernel_batch78y::ieeetran_*`.
  RawTeX!(r"\def\@IEEEtablestring{table}");
  let fonts = caption_sty::class_caption_fonts();
  let (table_font, compsoc_font) =
    if fonts { (r"\scshape ", r"\def\fnum@font@float{\sffamily}\def\format@title@font@float{\sffamily}%") } else { ("", "") };
  caption_sty::class_makecaption(&s!(
    r"\long\def\lx@makecaption##1##2{{\lx@tag[][. ]{{\lx@fnum@@{{##1}}}}##2}}%
\long\def\format@title@figure##1{{\lx@makecaption{{figure}}{{##1}}}}%
\ifCLASSOPTIONcompsoc
  \ifCLASSOPTIONconference
    \long\def\format@title@table##1{{\lx@tag[][. ]{{\lx@fnum@@{{table}}}}{table_font}##1}}%
  \else{compsoc_font}
    \long\def\format@title@table##1{{\lx@tag{{\lx@fnum@@{{table}}}}\\##1}}%
  \fi
\else
  \long\def\format@title@table##1{{\lx@tag{{\lx@fnum@@{{table}}}}\\{table_font}##1}}%
\fi"
  ))?;

  // The QED symbols and `{IEEEproof}` (Perl IEEEtran.cls.ltxml:194-222), shared with the IEEEtrantools binding, whose
  // package carries the same code (IEEEtrantools.sty:2726-2747).
  ieeetrantools_sty::define_ieeeproof()?;

  // IEEEbiography (Perl IEEEtran.cls.ltxml L238-247) — two-column
  // tabular-in-float: photo/placeholder on left, bolded author + body
  // on right. Matches Perl shape byte-for-byte.
  DefEnvironment!("{IEEEbiography}[]{}",
    "<ltx:float class='biography'>\
      <ltx:tabular>\
        <ltx:tr>\
          <ltx:td>#1</ltx:td>\
          <ltx:td><ltx:inline-block>\
            <ltx:text class='ltx_font_bold'>#2</ltx:text> #body\
          </ltx:inline-block></ltx:td>\
        </ltx:tr>\
      </ltx:tabular>\
    </ltx:float>");
  DefEnvironment!("{IEEEbiographynophoto}[]{}",
    "<ltx:float class='biography'>\
      <ltx:tabular>\
        <ltx:tr>\
          <ltx:td><ltx:inline-block>\
            <ltx:text class='ltx_font_bold'>#2</ltx:text> #body\
          </ltx:inline-block></ltx:td>\
        </ltx:tr>\
      </ltx:tabular>\
    </ltx:float>");

  // IEEEeqnarray, IEEEeqnarraybox, the (sub)numbering switches and the L/C/R column types (Perl IEEEtran.cls.ltxml
  // L242-332; IEEEtran.cls carries the code IEEEtrantools.sty ships), shared with the IEEEtrantools binding.
  ieeetrantools_sty::define_ieeeeqnarray()?;

  // IEEEtran.cls:5752-5775: `\appendices` numbers the appendices `\Roman` ("Appendix I") under the
  // `romanappendices` option and `\Alph` ("Appendix A") otherwise; `\appendix` gives `\Alph`. Perl lets
  // `\appendices` be `\appendix` (KNOWN_PERL_ERRORS #491). Guard
  // `perfect_kernel_batch61::captions_and_2609_class_commands_keep_their_text`.
  // IEEEtran's `\appendices` is its own code, not a call of `\appendix`, so it reaches the kernel's through a copy
  // taken here: a document's `\let\appendix\appendices` (2605.23316) then runs once instead of looping.
  Let!("\\lx@IEEE@appendix", "\\appendix");
  // IEEEtran.cls:5597-5627: a theorem numbered within sections prints the section through
  // `\@IEEEthmcounterinsection`, `\arabic` (so "Theorem 2.1" where `\thesection` is "II"), "A" after `\appendix`, `\Alph`
  // after `\appendices` (`A\arabic` under `romanappendices`), :5737/:5768/:5773. Perl's binding prints `\thesection`,
  // "II.1": KNOWN_PERL_ERRORS #1079. Witness 1911.01952. Guard `perfect_kernel_batch78n::ieeetran_theorem_within_section`.
  RawTeX!(r"\gdef\@IEEEthmcounterinsection#1{\arabic{#1}}\edef\@IEEEstringsection{section}");
  set_theorem_within_display("section", Some("\\@IEEEthmcounterinsection"));
  // IEEEtran V1.5-1.6c's switch for the same choice (V1.6b :4075-4091 `\newif\ifuseRomanappendices`, read by its
  // `\appendices`), which their documents set (`\useRomanappendicesfalse`; 67x, witnesses cs0504030, 0805.0012,
  // quant-ph0703150, 0705.3677): false here, as V1.8b numbers `\Alph` without the option (V1.6's default was true).
  // The heading form `\thesectiondis` follows (:5765-5774; `\renewcommand{\thesubsection}{\thesectiondis-…}` in
  // 1605.01233, 1702.07429).
  // `\appendix[<title>]` (:5719-5745) is one appendix: it resets the counters, clears the heading form `\thesectiondis`,
  // numbers the subsections `\Alph` ("A. Sub") and prints its heading, "Appendix" or "Appendix \\* <title>" through the
  // kernel's `\section*`, with `\refstepcounter{section}` making `\ref` "A". Here the heading is the appendix (refnum
  // "A", tag "Appendix", the `\appendixname` alone as the class prints it), its title the optional argument (rev78c2
  // probe i2, `[Proof]` had been a paragraph and the next subsection "B."/".2"; KNOWN_PERL_ERRORS #1022). The class's
  // `\section` after it throws its argument away (`\@IEEEdestroythesectionargument`); here the authored title is kept
  // (OXIDIZED_DESIGN_DIVERGENCES #793): the first unstarred `\section` after an argument-less `\appendix` — past blank
  // lines, spaces, empty groups (2002.07641's `\appendix{}`) and `\label`s, which then label the appendix — is the
  // appendix's heading (pdflatex prints "APPENDIX" alone); anything else next (`\section*`, `\subsection`, text, and —
  // a residual — a non-printing command or a float before the `\section`: 2404.01875, 2007.07179) follows the "Appendix"
  // heading the class prints
  // (rev78d probe ie_blank; witnesses 1903.01287, 1103.2068, 2002.03484, 2410.15995, 2401.07123, whose blank line had
  // made the titled section a second appendix "B"). A later `\section` is another appendix, tagged "Appendix B", "C"
  // as its refnum (pdflatex drops its title, and a `\ref` to it prints "A"). `\appendices` after `\appendix` numbers
  // its appendices from "A" again with the kernel's tag (probe ie_appendices: "Appendix A", "Appendix B"). Guard
  // `perfect_kernel_batch78d::ieeetran_appendix_with_a_title_is_one_appendix`.
  // Both save the kernel's `\section` once as `\@IEEEappendixsavesection` (:5695-5696 `\@IEEEsaveoriginalsectiononce`),
  // which documents' own appendix-like commands copied from the class's `\appendix` call, with its
  // `\@IEEEprocessthesectionargument`/`\@IEEEdestroythesectionargument` (:5701-5713; 1907.08915's `\suppl`).
  RawTeX!(r"\def\@IEEEsaveoriginalsectiononce{\let\@IEEEappendixsavesection\section
\let\@IEEEsaveoriginalsectiononce\relax}
\def\@IEEEprocessthesectionargument#1{\@ifmtarg{#1}{\@IEEEappendixsavesection*{\appendixname\nobreakspace\thesectiondis}%
\addcontentsline{toc}{section}{\appendixname\nobreakspace\thesection}}{%
\@IEEEappendixsavesection*{\appendixname\nobreakspace\thesectiondis\\* #1}%
\addcontentsline{toc}{section}{\appendixname\nobreakspace\thesection: #1}}}
\def\@IEEEdestroythesectionargument#1{\typeout{** WARNING: Ignoring useless
\protect\section\space in Appendix (line \the\inputlineno).}}
\renewcommand{\appendix}[1][]{\@IEEEsaveoriginalsectiononce\par\lx@IEEE@appendix
\setcounter{subsection}{0}\setcounter{subsubsection}{0}\setcounter{paragraph}{0}%
\gdef\thesectiondis{}\gdef\thesubsection{\Alph{subsection}}%
\gdef\fnum@appendix{\ifnum\c@section>\@ne\lx@@fnum@@{appendix}\else\appendixname\fi}%
\gdef\@IEEEthmcounterinsection##1{A}%
\gdef\lx@IEEE@appendix@labels{}\@ifmtarg{#1}{\lx@IEEE@appendix@next}{\section{#1}}}
\def\lx@IEEE@appendix@next{\@ifnextchar\par{\expandafter\lx@IEEE@appendix@next\@gobble}{%
\@ifnextchar\label{\expandafter\lx@IEEE@appendix@label\@gobble}{%
\@ifnextchar\bgroup{\lx@IEEE@appendix@group}{%
\@ifnextchar\section{\expandafter\lx@IEEE@appendix@section\@gobble}{\lx@IEEE@appendix@heading}}}}}
\def\lx@IEEE@appendix@label#1{\g@addto@macro\lx@IEEE@appendix@labels{\label{#1}}\lx@IEEE@appendix@next}
\long\def\lx@IEEE@appendix@group#1{\@ifmtarg{#1}{\lx@IEEE@appendix@next}{\lx@IEEE@appendix@heading{#1}}}
\def\lx@IEEE@appendix@heading{\section{}\lx@IEEE@appendix@labels}
\def\lx@IEEE@appendix@section{\@ifstar{\lx@IEEE@appendix@heading\section*}{\@ifnextchar[%
{\lx@IEEE@appendix@titled}{\lx@IEEE@appendix@titled@}}}
\def\lx@IEEE@appendix@titled[#1]#2{\section[#1]{#2}\lx@IEEE@appendix@labels}
\def\lx@IEEE@appendix@titled@#1{\section{#1}\lx@IEEE@appendix@labels}
\newif\ifuseRomanappendices
\def\appendices{\@IEEEsaveoriginalsectiononce\lx@IEEE@appendix\global\let\fnum@appendix\@undefined\setcounter{section}{0}%
\ifCLASSOPTIONromanappendices\lx@IEEE@roman@appendices\else\ifuseRomanappendices\lx@IEEE@roman@appendices\else
\gdef\thesectiondis{\Alph{section}}\@IEEEcompsocconfonly{\gdef\thesectiondis{\Alph{section}.}}%
\gdef\@IEEEthmcounterinsection##1{\Alph{##1}}\fi\fi}
\def\lx@IEEE@roman@appendices{\gdef\thesection{\Roman{section}}\gdef\thesectiondis{\Roman{section}}%
\gdef\@IEEEthmcounterinsection##1{A\arabic{##1}}%
\@IEEEcompsocconfonly{\gdef\thesectiondis{\Roman{section}.}}}");

  // Bibliography style — Perl IEEEtran doesn't touch bibliography
  // beyond the (commented-out) bstctlcite documentation at L442. Stale
  // "AssignMapping not yet ported" note removed; nothing to port here.

  // The IED lists `{IEEEitemize}`/`{IEEEenumerate}`/`{IEEEdescription}` and their parameters (Perl IEEEtran.cls.ltxml
  // L340-366), shared with the IEEEtrantools binding (IEEEtrantools.sty:1000-1140).
  ieeetrantools_sty::define_ied_lists()?;

  // Override LaTeX's default IED lists with the IEEE versions defined
  // above. Per Perl L369-380: a `\let` of the bare CSes AND a `\let` on
  // the `\begin{itemize}` / `\end{itemize}` env-tokens, so user code that
  // writes `\begin{itemize}` (the LaTeXish form) hits the IEEE variant
  // — without these the standard latex-core itemize wins, bypassing
  // IEEE's locked spec. 12 aliases total.
  // IEEEtran.cls:2219-2222 (V1.6b on) first saves LaTeX's own lists as `{LaTeXitemize}`/`{LaTeXenumerate}`, beside
  // the `{LaTeXdescription}` below (67x; witnesses cs0701047, 2312.13891, 1502.01199).
  RawTeX!(r"\let\LaTeXitemize\itemize \let\endLaTeXitemize\enditemize
\let\LaTeXenumerate\enumerate \let\endLaTeXenumerate\endenumerate");
  Let!("\\itemize",      "\\IEEEitemize");
  Let!("\\enditemize",   "\\endIEEEitemize");
  Let!("\\enumerate",    "\\IEEEenumerate");
  Let!("\\endenumerate", "\\endIEEEenumerate");
  Let!("\\description",  "\\IEEEdescription");
  Let!("\\enddescription","\\endIEEEdescription");
  Let!("\\begin{itemize}",      "\\IEEEitemize");
  Let!("\\end{itemize}",        "\\endIEEEitemize");
  Let!("\\begin{enumerate}",    "\\IEEEenumerate");
  Let!("\\end{enumerate}",      "\\endIEEEenumerate");
  Let!("\\begin{description}",  "\\IEEEdescription");
  Let!("\\end{description}",    "\\endIEEEdescription");

  // String macros (Perl L383-395)
  DefMacro!("\\contentsname", "Contents");
  DefMacro!("\\listfigurename", "List of Figures");
  DefMacro!("\\listtablename", "List of Tables");
  DefMacro!("\\refname", "References");
  DefMacro!("\\indexname", "Index");
  DefMacro!("\\partname", "Part");
  DefMacro!("\\appendixname", "Appendix");
  DefMacro!("\\abstractname", "Abstract");
  DefMacro!("\\IEEEkeywordsname", "Index Terms");
  DefMacro!("\\IEEEproofname", "Proof");
  // IEEEtran's own `\…name`s are text names a copy may spell otherwise (OXIDIZED_DESIGN_DIVERGENCES #731 rule 14,
  // `state::declare_copy_text_names`): 1803.07915's IEEEURAI.cls (`\ProvidesClass{IEEEtran}`) heads its keywords
  // "Keywords", as its PDF.
  declare_copy_text_names(&["\\IEEEkeywordsname", "\\IEEEproofname"]);

  // Legacy aliases (Perl L398-439)
  Let!("\\authorblockA", "\\IEEEauthorblockA");
  Let!("\\authorblockN", "\\IEEEauthorblockN");
  Let!("\\authorrefmark", "\\IEEEauthorrefmark");
  Let!("\\PARstart", "\\IEEEPARstart");
  Let!("\\pubid", "\\IEEEpubid");
  Let!("\\pubidadjcol", "\\IEEEpubidadjcol");
  Let!("\\specialpapernotice", "\\IEEEspecialpapernotice");

  // Keywords environment aliases — Perl L406-414
  // Perl dispatches on whether the next token is a brace:
  //   \keywords{foo}  → \keywords@onearg{foo}
  //   \keywords … \endkeywords (env form) → \@IEEEkeywords
  // Rust was hardcoding the env-start path, so braced `\keywords{foo}`
  // never reached the one-arg expansion.
  // IEEEtran.cls (1.8b) defines no `\keywords`: the alias is a convenience, so it is a fallback a
  // document's own `\newcommand{\keywords}{M-estimators, …}` replaces, as in TeX — Perl locks it
  // (IEEEtran.cls.ltxml:398-403), which refused the definition, and `\keywords` in
  // `{IEEEkeywords}` opened a second keywords block whose end met the environment's `\endgroup`
  // (witnesses 2006.10005, 2008.05854, 2011.04315; KNOWN_PERL_ERRORS #664,
  // `DefinitionOrigin::Fallback`, `is_definable_latex`).
  {
    use latexml_core::definition::origin::{DefinitionOrigin, OriginGuard};
    let _fallback = OriginGuard::new(DefinitionOrigin::Fallback);
    DefMacro!("\\keywords", sub[_args] {
      let next = read_token()?;
      if let Some(t) = next {
        unread(Tokens!(t));
        if t.get_catcode() == Catcode::BEGIN {
          return Ok(Tokens!(T_CS!("\\keywords@onearg")));
        }
      }
      Ok(Tokens!(T_CS!("\\@IEEEkeywords")))
    });
  }
  DefMacro!("\\keywords@onearg{}",
    "\\@IEEEkeywords #1 \\@endIEEEkeywords");
  DefMacro!("\\endkeywords", "\\@endIEEEkeywords");
  // Perl L406-407: explicit `\begin{keywords}` / `\end{keywords}` env-token
  // aliases so user `\def\keywords` / `\def\endkeywords` redefinitions
  // don't break the env semantics. Drivers: 2007.06704
  // (`\def\keywords{...}\def\endkeywords{\par}` followed by
  // `\begin{keywords}...\end{keywords}` — without these aliases, our
  // standard `\begin{X} → \begingroup\X` expansion routed through user's
  // `\par`-redef'd `\endkeywords`, leaving `\@IEEEkeywords`'s
  // `XUntil:\@endIEEEkeywords` reading past EOF).
  DefMacro!(T_CS!("\\begin{keywords}"), None, "\\begingroup\\@IEEEkeywords");
  DefMacro!(T_CS!("\\end{keywords}"),   None, "\\@endIEEEkeywords\\endgroup");
  // The `\@`-prefixed internal indirection the routing above targets. It is deliberately separate
  // from `\IEEEkeywords`/`\endIEEEkeywords` so a user `\def\keywords`/`\def\endkeywords` can't break
  // env routing — but it was REFERENCED above (L654-676) and never DEFINED, so every legacy
  // `\begin{keywords}` / `\keywords` bounced as `Error:undefined:\@IEEEkeywords` (the 27-doc cluster;
  // witness 0712.0271). Point the internals at the real begin/end keywords macros (Perl
  // IEEEtran.cls.ltxml L147-148: `\IEEEkeywords → \lx@begin@keywords`, `\endIEEEkeywords → \lx@end@keywords`).
  Let!("\\@IEEEkeywords", "\\IEEEkeywords");
  Let!("\\@endIEEEkeywords", "\\endIEEEkeywords");

  // Legacy IED list aliases — Perl IEEEtran.cls.ltxml L417-423
  Let!("\\labelindent", "\\IEEElabelindent");
  Let!("\\calcleftmargin", "\\IEEEcalcleftmargin");
  Let!("\\setlabelwidth", "\\IEEEsetlabelwidth");
  Let!("\\usemathlabelsep", "\\IEEEusemathlabelsep");
  Let!("\\iedlabeljustifyc", "\\IEEEiedlabeljustifyc");
  Let!("\\iedlabeljustifyl", "\\IEEEiedlabeljustifyl");
  Let!("\\iedlabeljustifyr", "\\IEEEiedlabeljustifyr");

  // QED/proof aliases
  Let!("\\QED", "\\IEEEQED");
  Let!("\\QEDclosed", "\\IEEEQEDclosed");
  Let!("\\QEDopen", "\\IEEEQEDopen");
  DefMacro!("\\qed", "\\ltx@qed");
  DefConstructor!("\\ltx@qed",
    "?#isMath(<ltx:XMTok role='PUNCT'>\u{220E}</ltx:XMTok>)(\u{220E})",
    enter_horizontal => true, reversion => "\\qed",
    sizer => sub[whatsit] { Ok(symbol_box_size(whatsit, 1.3, 1.3, FontUnit::Ex)) });
  // Perl L423-424 `\proof` → `\IEEEproof`: IEEEtran 1.8b defines no `\proof` (IEEEtran.cls:6332, commented
  // out with the V1.7 aliases), so the alias is a fallback a document's own `\newenvironment{proof}` replaces
  // (1002.0117, 1010.1899; `DefinitionOrigin::Fallback`, `is_definable_latex`).
  {
    use latexml_core::definition::origin::{DefinitionOrigin, OriginGuard};
    let _fallback = OriginGuard::new(DefinitionOrigin::Fallback);
    Let!("\\proof", "\\IEEEproof");
    Let!("\\endproof", "\\endIEEEproof");
  }
  // IEEEtran proofs route through amsthm's `\@proof` / `\end@proof`
  // machinery (the magic `\begin{proof}` CS from amsthm_sty.rs:220 —
  // `\begin{proof}` → `\begin{@proof}`). We re-override its head style (`\lx@amsthm@proof@style`)
  // here so the amsthm header-font is bold-italic (Perl ships
  // `\textbf{\textit{Proof:}}` under IEEEtran, producing
  // font="bold italic"). Keeping amsthm's env path also gives us the
  // QED symbol emission at proof-end for free — it's already wired
  // through amsthm's `\end@proof` before_digest stack-pop.
  RawTeX!(r"\def\lx@amsthm@proof@style{\def\thm@headfont{\bfseries\itshape}\def\thm@bodyfont{\normalfont}}");

  // Biography aliases
  Let!("\\biography", "\\IEEEbiography");
  Let!("\\biographynophoto", "\\IEEEbiographynophoto");
  Let!("\\endbiography", "\\endIEEEbiography");
  Let!("\\endbiographynophoto", "\\endIEEEbiographynophoto");

  // bstctlcite stub (Perl L445)
  def_macro_noop("\\bstctlcite[]{}")?;

  // Disable internal alignment env (Perl IEEEtran.cls.ltxml:443-444): the `\begin{@IEEEauthorhalign}` and
  // `\end{@IEEEauthorhalign}` control sequences themselves, which `\begin`/`\end` look up first, so a document's
  // `\authorrowbreak` (`\end{@IEEEauthorhalign}\hfill\mbox{}\par\mbox{}\hfill\begin{@IEEEauthorhalign}`) opens and
  // closes no group in the frontmatter digest (2609.02116; the `\@IEEEauthorhalign`/`\end@IEEEauthorhalign` pair
  // this defined left `\end`'s `\endgroup` live).
  DefMacro!(T_CS!("\\begin{@IEEEauthorhalign}"), None, "\\relax");
  DefMacro!(T_CS!("\\end{@IEEEauthorhalign}"), None, "\\relax");

  // \linebreakand — IEEEtran tip-jar macro that papers redefine to break
  // a multi-row author halign across visual lines. The canonical
  // tex.stackexchange recipe is:
  //   \newcommand{\linebreakand}{
  //     \end{@IEEEauthorhalign}\hfill\mbox{}\par\mbox{}\hfill
  //     \begin{@IEEEauthorhalign}}
  // The unbalanced `\end{...}\begin{...}` pair pops + pushes a frame
  // on the live stack, which is fine in real IEEEtran (where the
  // \author{...} body is wrapped in `\begin{@IEEEauthorhalign}...\end{@IEEEauthorhalign}`)
  // but breaks our `\@personname`-wrapped frontmatter digest where
  // there's no matching outer halign frame. Pre-define `\linebreakand`
  // as a paragraph break so user `\newcommand{\linebreakand}{...}`
  // can't override (it's already-defined → silently ignored), matching
  // Perl's behavior where the user redefinition triggers
  // ``\linebreakand:locked`` and gets dropped on the floor. Driver
  // cluster: 2211.12981, 2403.11083, 2405.03537, 2405.04387 (IEEEtran
  // multi-author papers using the linebreakand recipe).
  DefMacro!("\\linebreakand", "\\par");
  // \newlineauthors — sibling-recipe of \linebreakand commonly defined
  // in IEEE conference-template papers (driver cluster: 2601.15292,
  // 2601.16670, 2602.05977). User \newcommand body is the same
  // \end{@IEEEauthorhalign}\hfill\mbox{}\par\mbox{}\hfill\begin{@IEEEauthorhalign}
  // pattern that breaks our frontmatter digest. Pre-define so the
  // \newcommand is silently ignored (Perl-parity: already-defined →
  // dropped on the floor).
  DefMacro!("\\newlineauthors", "\\par");
  // IEEEtran.cls (TL 2025 v1.8b, as nearly every sampled copy, 42-58 of 58), what papers call that the binding lacked
  // (run 340: `{LaTeXdescription}` 9 of 58 sampled papers that ship IEEEtran.cls, `\IEEEstrut` 5,
  // `\IEEEsettopmargin` 4, `\yesnumber` 2, `\IEEEeqnarraystrutmode` 2; ~100 first errors; witnesses 2009.01542,
  // 2403.19006, 2001.00914, 2510.10761, 2001.00708):
  // - `{LaTeXdescription}` (:2226-2229), LaTeX's own description list, which the class keeps beside its IEEE one;
  // - `\yesnumber` (:3068), the opposite of `\nonumber`: `\IEEEyesnumber`;
  // - `\IEEEstrut[<height>][<depth>][<font>]` (:3585-3590), `\IEEEsettopmargin[<sample>]{<mode>}{<margin>}` (:1394),
  //   `\IEEEeqnarraystrutmode` and `\IEEEeqnarraystrutsize{<height>}{<depth>}[<font>]` (:3600-3613): struts and
  //   page geometry, which set nothing here.
  RawTeX!(r"\newenvironment{LaTeXdescription}{\begin{description}}{\end{description}}
\providecommand{\yesnumber}{\IEEEyesnumber\relax}");
  DefMacro!("\\IEEEstrut[][][]", "");
  DefMacro!("\\IEEEsettopmargin[]{}{}", "");
  // :1190 `\IEEEsettextheight[<sample>]{<top margin>}{<bottom margin>}`, page geometry too (65z; witnesses 2605.04966, 1905.10846).
  DefMacro!("\\IEEEsettextheight[]{}{}", "");
  // The rest of that page-geometry family (65z): :1354 `\IEEEquantizetextheight[<unit>]{<mode>}`, :1113
  // `\IEEEsettextwidth{<inner>}{<outer>}`, :1133 `\IEEEsetsidemargin{<mode>}{<margin>}`, :1478/:1570
  // `\IEEEsetheadermargin`/`\IEEEsetfootermargin[<sample>][<text sample>]{<mode>}{<margin>}`; :3506
  // `\IEEEeqnarraystrutsizeadd{<height>}{<depth>}[<font>]` beside the strut size below.
  DefMacro!("\\IEEEquantizetextheight[]{}", "");
  DefMacro!("\\IEEEsettextwidth{}{}", "");
  DefMacro!("\\IEEEsetsidemargin{}{}", "");
  DefMacro!("\\IEEEsetheadermargin[][]{}{}", "");
  DefMacro!("\\IEEEsetfootermargin[][]{}{}", "");
  DefMacro!("\\IEEEeqnarraystrutsizeadd{}{}[]", "");
  if lookup_definition(&T_CS!("\\IEEEeqnarraystrutsize"))?.is_none() {
    DefMacro!("\\IEEEeqnarraystrutsize{}{}[]", "");
  }
  def_macro_noop("\\IEEEeqnarraystrutmode")?;

  // 67x: the IEEEtran.cls internals documents call — mostly in their own copies of the class's `\maketitle`,
  // `\IEEEkeywords` or `\confheader` code — and the names of older and variant versions their papers still use
  // (run 340 undefined-macro inventory, 692 papers; Perl's binding lacks them all, KNOWN_PERL_ERRORS #710/#711).
  // IEEEtran.cls (TL 2025, V1.8b) as written:
  // - :456-462 the compsoc-only executors `\@IEEEcompsoconly`, `\@IEEEnotcompsoconly`, `\@IEEEcompsocconfonly`,
  //   `\@IEEEcompsocnotconfonly` (an URSI-style `\newmaketitle`; 2202.10460, 2202.12377, 2203.08603, 2204.11491);
  // - :606-697 `\@IEEEnormalsizeunitybaselineskip`, the size option's unity baselineskip, and :976
  //   `\CLASSINFOnormalsizeunitybaselineskip`; :639-708 `\sublargesize`, the size between `\normalsize` and `\large`
  //   (1208.3665, 2103.13318); :4748-4750 `\IEEEtitletopspace`/`\IEEEtitletopspaceextra`; :4969-4978 the title
  //   spacer's `\@IEEENORMtitlevspace`/`\@IEEEMINtitlevspace`; :5208 `\IEEEquantizedisabletitlecmds` (2601.10876);
  // - :1074 `\@IEEEtunefonts` and its `\@IEEEsetfontdimens`, which retune the interword spacing (`\fontdimen`s, page
  //   layout only, so nothing here; 0705.1682, 0705.3677, 0705.1922 in V1.7a);
  // - :2085-2153 the IED list parameters (`define_ied_lists`, above);
  // - :2571-2587 the heading forms of the section numbers, `\thesectiondis` … `\theparagraphdis` (1605.01233,
  //   1702.07429, 1610.00529), which :5397/:5402 `\@seccntformat` prints in the heading ("I.", "A.", "1)", where a
  //   `\ref` reads "I-A"): the heading tags' `\fnum@<level>` (base_utilities.rs `\lx@fnum@@`; Perl's binding has no
  //   heading forms, KNOWN_PERL_ERRORS #1022; about 178,000 IEEEtran papers with a `\subsection`). An appendix keeps
  //   its "Appendix A" (the `appendix` type's own tag), its subsections "A.". Guard
  //   `perfect_kernel_batch78c::ieeetran_headings_print_their_heading_forms`;
  // - :4228-4347 the page styles `\ps@headings` (which documents' `\confheader` recipes call; 1805.00151,
  //   1805.02797, 1811.10132, 1908.05569, 1910.06140), `\ps@IEEEtitlepagestyle`, `\ps@IEEEpeerreviewcoverpagestyle`,
  //   with their `\@IEEEheaderstyle`/`\@IEEEfooterstyle` and :6127-6150 `\@IEEEpubidpullup`, `\@IEEEpubid`,
  //   `\if@IEEEusingpubid` (running heads and feet: page furniture, set nowhere here);
  // - :4448 `\IEEEnoauxwrite{<citations>}` (2006.13598); :4484-4486 `\IEEEbibitemsep` (2007.01482, 2106.14760);
  // - :4552-4561 the conference author blocks' `\@IEEEauthorblockNstyle`, `\@IEEEauthorblockAstyle`,
  //   `\@IEEEauthordefaulttextstyle`, `\@IEEEauthorblockconfadjspace` (2507.18515, 1811.01547);
  // - :5219 `\@IEEEaftertitletext` and :6101 `\@IEEEspecialpapernotice`, what `\IEEEaftertitletext` and
  //   `\IEEEspecialpapernotice` store for the title: empty, as those two make frontmatter notes here (above);
  // - :5668-5669 the `\@ifmtarg` the class carries from ifmtarg.sty (1510.02032).
  RawTeX!(r"\long\def\@IEEEcompsoconly#1{\relax\ifCLASSOPTIONcompsoc\relax#1\relax\fi\relax}
\long\def\@IEEEnotcompsoconly#1{\relax\ifCLASSOPTIONcompsoc\else\relax#1\relax\fi\relax}
\long\def\@IEEEcompsocconfonly#1{\relax\ifCLASSOPTIONcompsoc\ifCLASSOPTIONconference\relax#1\relax\fi\fi\relax}
\long\def\@IEEEcompsocnotconfonly#1{\relax\ifCLASSOPTIONcompsoc\ifCLASSOPTIONconference\else\relax#1\relax\fi\fi\relax}
\newdimen\@IEEEnormalsizeunitybaselineskip
\@IEEEnormalsizeunitybaselineskip 12pt
\def\sublargesize{\@setfontsize{\sublargesize}{11}{13.4pt}}
\ifnum\CLASSOPTIONpt=9 \@IEEEnormalsizeunitybaselineskip 11.0476pt
  \def\sublargesize{\@setfontsize{\sublargesize}{10}{12pt}}\fi
\ifnum\CLASSOPTIONpt=11 \@IEEEnormalsizeunitybaselineskip 13.3846pt
  \def\sublargesize{\@setfontsize{\sublargesize}{12}{14pt}}\fi
\ifnum\CLASSOPTIONpt=12 \@IEEEnormalsizeunitybaselineskip 13.92pt
  \def\sublargesize{\@setfontsize{\sublargesize}{14}{17pt}}\fi
\newdimen\CLASSINFOnormalsizeunitybaselineskip
\CLASSINFOnormalsizeunitybaselineskip\@IEEEnormalsizeunitybaselineskip
\def\@IEEEsetfontdimens{}
\def\@IEEEtunefonts{}
\def\IEEEtitletopspace{0.5\baselineskip}
\def\IEEEtitletopspaceextra{0pt}
\def\@IEEENORMtitlevspace{2.5\baselineskip}
\def\@IEEEMINtitlevspace{2\baselineskip}
\ifCLASSOPTIONconference\def\@IEEENORMtitlevspace{1\baselineskip}\def\@IEEEMINtitlevspace{0.75\baselineskip}\fi
\ifCLASSOPTIONtechnote\def\@IEEENORMtitlevspace{1\baselineskip}\def\@IEEEMINtitlevspace{0.75\baselineskip}\fi
\def\IEEEquantizedisabletitlecmds{\long\def\thanks##1{\relax}\relax
\long\def\IEEEcompsocitemizethanks##1{\relax}\def\newpage{\relax}}
\ifCLASSOPTIONcompsoc
  \ifCLASSOPTIONconference
    \def\thesectiondis{\thesection.}
    \def\thesubsectiondis{\thesectiondis\arabic{subsection}.}
    \def\thesubsubsectiondis{\thesubsectiondis\arabic{subsubsection}.}
    \def\theparagraphdis{\thesubsubsectiondis\arabic{paragraph}.}
  \else
    \def\thesectiondis{\thesection}
    \def\thesubsectiondis{\thesectiondis.\arabic{subsection}}
    \def\thesubsubsectiondis{\thesubsectiondis.\arabic{subsubsection}}
    \def\theparagraphdis{\thesubsubsectiondis.\arabic{paragraph}}
  \fi
\else
  \def\thesectiondis{\thesection.}
  \def\thesubsectiondis{\Alph{subsection}.}
  \def\thesubsubsectiondis{\arabic{subsubsection})}
  \def\theparagraphdis{\alph{paragraph})}
\fi
\def\fnum@section{\thesectiondis}\def\fnum@subsection{\thesubsectiondis}
\def\fnum@subsubsection{\thesubsubsectiondis}\def\fnum@paragraph{\theparagraphdis}
\def\@IEEEheaderstyle{\normalfont\scriptsize}
\def\@IEEEfooterstyle{\normalfont\scriptsize}
\ifCLASSOPTIONcompsoc
  \def\@IEEEheaderstyle{\normalfont\sffamily\scriptsize}
  \def\@IEEEfooterstyle{\normalfont\sffamily\scriptsize}
\fi
\def\@IEEEpubidpullup{1.3\baselineskip}
\ifCLASSOPTIONtechnote\def\@IEEEpubidpullup{2\baselineskip}\fi
\ifCLASSOPTIONcompsoc\def\@IEEEpubidpullup{0pt}\fi
\def\@IEEEpubid{\relax}
\newif\if@IEEEusingpubid
\global\@IEEEusingpubidfalse
\def\ps@headings{%
\def\@oddhead{\hbox{}\@IEEEheaderstyle\rightmark\hfil\thepage}\relax
\def\@evenhead{\@IEEEheaderstyle\thepage\hfil\leftmark\hbox{}}\relax
\let\@oddfoot\@empty
\let\@evenfoot\@empty
\ifCLASSOPTIONtechnote
  \def\@oddhead{\hbox{}\@IEEEheaderstyle\leftmark\hfil\thepage}\relax
  \def\@evenhead{\@IEEEheaderstyle\thepage\hfil\leftmark\hbox{}}\relax
\fi
\ifCLASSOPTIONdraftcls
  \def\@oddfoot{\@IEEEfooterstyle\@date\hfil DRAFT}\relax
  \def\@evenfoot{\@IEEEfooterstyle DRAFT\hfil\@date}\relax
\fi
\if@twoside\else
  \def\@oddhead{\hbox{}\@IEEEheaderstyle\leftmark\hfil\thepage}\relax
  \let\@evenhead\@empty
  \ifCLASSOPTIONdraftcls
    \def\@oddfoot{\@IEEEfooterstyle\@date\hfil DRAFT}\relax
    \let\@evenfoot\@empty
  \fi
\fi
\ifCLASSOPTIONconference
  \let\@oddhead\@empty
  \let\@evenhead\@empty
\fi
\ifCLASSOPTIONdraftclsnofoot
  \let\@oddfoot\@empty
  \let\@evenfoot\@empty
\fi}
\def\ps@IEEEtitlepagestyle{%
\def\@oddhead{\hbox{}\@IEEEheaderstyle\leftmark\hfil\thepage}\relax
\def\@evenhead{\@IEEEheaderstyle\thepage\hfil\leftmark\hbox{}}\relax
\let\@oddfoot\@empty
\let\@evenfoot\@empty
\ifCLASSOPTIONdraftcls
  \ifCLASSOPTIONdraftclsnofoot\else
    \def\@oddfoot{\@IEEEfooterstyle\@date\hfil DRAFT}\relax
    \def\@evenfoot{\@IEEEfooterstyle DRAFT\hfil\@date}\relax
  \fi
\else
  \if@IEEEusingpubid
    \ifCLASSOPTIONpeerreview\else
      \def\@oddfoot{\hbox{}\hss\@IEEEfooterstyle\footnotesize\raisebox{\footskip}[0pt][0pt]{\@IEEEpubid}\hss\hbox{}}\relax
      \def\@evenfoot{\hbox{}\hss\@IEEEfooterstyle\footnotesize\raisebox{\footskip}[0pt][0pt]{\@IEEEpubid}\hss\hbox{}}\relax
      \ifCLASSOPTIONcompsoc
        \def\@oddfoot{\hbox{}\hss\@IEEEfooterstyle\raisebox{0pt}[0pt][0pt]{\@IEEEpubid}\hss\hbox{}}\relax
        \def\@evenfoot{\hbox{}\hss\@IEEEfooterstyle\raisebox{0pt}[0pt][0pt]{\@IEEEpubid}\hss\hbox{}}\relax
      \fi
    \fi
  \fi
\fi
\ifCLASSOPTIONconference
  \let\@oddhead\@empty
  \let\@evenhead\@empty
\fi}
\def\ps@IEEEpeerreviewcoverpagestyle{%
\let\@oddhead\@empty
\let\@evenhead\@empty
\let\@oddfoot\@empty
\let\@evenfoot\@empty
\ifCLASSOPTIONdraftcls
  \ifCLASSOPTIONdraftclsnofoot\else
    \def\@oddfoot{\@IEEEfooterstyle\@date\hfil DRAFT}\relax
    \def\@evenfoot{\@IEEEfooterstyle DRAFT\hfil\@date}\relax
  \fi
\else
  \if@IEEEusingpubid
    \def\@oddfoot{\hbox{}\hss\@IEEEfooterstyle\footnotesize\raisebox{\footskip}[0pt][0pt]{\@IEEEpubid}\hss\hbox{}}\relax
    \def\@evenfoot{\hbox{}\hss\@IEEEfooterstyle\footnotesize\raisebox{\footskip}[0pt][0pt]{\@IEEEpubid}\hss\hbox{}}\relax
    \ifCLASSOPTIONcompsoc
      \def\@oddfoot{\hbox{}\hss\@IEEEfooterstyle\raisebox{0pt}[0pt][0pt]{\@IEEEpubid}\hss\hbox{}}\relax
      \def\@evenfoot{\hbox{}\hss\@IEEEfooterstyle\raisebox{0pt}[0pt][0pt]{\@IEEEpubid}\hss\hbox{}}\relax
    \fi
  \fi
\fi}
\DeclareRobustCommand{\IEEEnoauxwrite}[1]{\relax\if@filesw\@fileswfalse#1\relax\relax\relax\relax\relax\@fileswtrue
\else#1\relax\relax\relax\relax\relax\fi}
\def\IEEEbibitemsep{0pt plus .5pt}
\@IEEEcompsocconfonly{\def\IEEEbibitemsep{0.5\baselineskip plus 0.25\baselineskip minus 0.25\baselineskip}}
\def\@IEEEauthorblockNstyle{\normalfont\@IEEEcompsocnotconfonly{\sffamily}\sublargesize}
\def\@IEEEauthorblockAstyle{\normalfont\@IEEEcompsocnotconfonly{\sffamily}\@IEEEcompsocconfonly{\itshape}\normalsize}
\def\@IEEEauthordefaulttextstyle{\normalfont\@IEEEcompsocnotconfonly{\sffamily}\sublargesize}
\def\@IEEEauthorblockconfadjspace{-0.25em}
\@IEEEcompsocconfonly{\def\@IEEEauthorblockconfadjspace{0.75\@IEEEnormalsizeunitybaselineskip}}
\let\@IEEEaftertitletext=\relax
\def\@IEEEspecialpapernotice{\relax}
\begingroup
\catcode`\Q=3
\long\gdef\@ifmtarg#1{\@xifmtarg#1QQ\@secondoftwo\@firstoftwo\@nil}
\long\gdef\@xifmtarg#1#2Q#3#4#5\@nil{#4}
\endgroup");
  // :4983-5203 `\IEEEquantizevspace*[<out dimen>]{<object>}[<decl>]{<top baselineskip>}[<offset>][<prevdepth>]
  // [<lineskiplimit>][<lineskip>]{<unit>}{<nominal space>}{<minimum space>}`, which the class's `\maketitle` (:4772)
  // and documents' copies of it (2601.10876) set the title with: it sets `<object>` in two measuring boxes and then
  // once more, followed by the space that brings the text below it to a multiple of `<unit>`, or puts that space in
  // `<out dimen>` instead. The arguments are read as written; the measuring and the space, page layout, are not
  // modelled — measuring the title would make its frontmatter twice — so `<object>` is set once, the out dimen 0pt.
  RawTeX!(r"\newtoks\@IEEEtrantmptoksA
\newif\if@IEEEquantizevspacestarform
\newif\if@IEEEquantizevspaceuseoutdimenreg
\def\IEEEquantizevspace{\begingroup\@ifstar{\@IEEEquantizevspacestarformtrue\@IEEEquantizevspace}{\@IEEEquantizevspacestarformfalse\@IEEEquantizevspace}}
\def\@IEEEquantizevspace{\@ifnextchar [{\@IEEEquantizevspaceuseoutdimenregtrue\@@IEEEquantizevspace}{\@IEEEquantizevspaceuseoutdimenregfalse\@@IEEEquantizevspace[]}}
\long\def\@@IEEEquantizevspace[#1]#2{\relax
\edef\@IEEEquantizeoutdimenreg{#1}\relax
\@IEEEtrantmptoksA={#2}\relax
\long\edef\@IEEEquantizeobject{\the\@IEEEtrantmptoksA}\relax
\@ifnextchar [{\@@@IEEEquantizevspace}{\@@@IEEEquantizevspace[\relax]}}
\long\def\@@@IEEEquantizevspace[#1]#2{\relax
\@IEEEtrantmptoksA={#1}\relax
\long\edef\@IEEEquantizeobjectdecl{\the\@IEEEtrantmptoksA}\relax
\edef\@IEEEquantizetopbaselineskip{#2}\ivIEEEquantizevspace}
\def\ivIEEEquantizevspace{\@ifnextchar [{\@vIEEEquantizevspace}{\@vIEEEquantizevspace[0pt]}}
\def\@vIEEEquantizevspace[#1]{\edef\@IEEEquantizeoffset{#1}\@ifnextchar [{\@viIEEEquantizevspace}{\@viIEEEquantizevspace[\prevdepth]}}
\def\@viIEEEquantizevspace[#1]{\@ifnextchar [{\@viiIEEEquantizevspace}{\@viiIEEEquantizevspace[\lineskiplimit]}}
\def\@viiIEEEquantizevspace[#1]{\edef\@IEEEquantizelineskiplimit{#1}\@ifnextchar [{\@viiiIEEEquantizevspace}{\@viiiIEEEquantizevspace[\lineskip]}}
\def\@viiiIEEEquantizevspace[#1]{\edef\@IEEEquantizelineskip{#1}\@ixIEEEquantizevspace}
\def\@ixIEEEquantizevspace#1#2#3{\global\let\@IEEEquantizeobjectout\@IEEEquantizeobject\relax\endgroup
\if@IEEEquantizevspaceuseoutdimenreg\@IEEEquantizeoutdimenreg=\z@\else\@IEEEquantizeobjectout\relax\fi}");
  // The names the IEEEtran versions before V1.7 had for the same things, and the variant classes papers ship (each
  // a fallback a document's own definition replaces, as for `\keywords` above):
  // - V1.5-1.6c `\ifcenterfigcaptions` (V1.6b :1966-1972, true in conference mode; the caption centring is layout;
  //   cs0501064, 0704.1833, cs0610078, cond-mat0207222, 1807.11264 in ieeeconf.cls), `\ifnocalcleftmargin` (V1.6b
  //   :1512; 2009.04715), `\@specialpapernotice` (V1.6b :4379, `\IEEEspecialpapernotice`'s store; 1811.05306,
  //   1811.01547); V1.6b's `\specialpapernotice` is the `\IEEEspecialpapernotice` above.
  // - V1.7a `\proof` (V1.7a :4690, `\AtBeginDocument{\def\proof{…\IEEEproof}…}`): one a document left undefined
  //   or `\relax` at `\begin{document}`, where `\begin{proof}` is an error under V1.8b, which defines none, is IEEEproof
  //   there (2106.12111's `\let\proof\@undefined`; 2208.13860, 2310.09933, 2310.09935, 2509.09277's `\let\proof\relax`
  //   after amsthm).
  // - IEEEtran4PSCC.cls :4793-4804 (PSCC's IEEEtran V1.8a; its `\thanks` commented out), `\thanksto{<note>}`, the
  //   paper's thanks note (funding, sponsorship), set as an unmarked footnote after `\maketitle` has let `\thanks` go:
  //   the frontmatter's thanks note, as a `\thanks` outside the author block (185 papers; 1601.01675, 1603.05188,
  //   1710.09738).
  // - The Turkish SIU template's IEEEtran.cls (2011/11/03 "V1.8 based on V1.7a" :1659-1662, :3745-3790): `{ozet}`,
  //   the abstract in Turkish under "Özetçe", and `{IEEEanahtar}`, its keywords under "Anahtar Kelimeler" — written
  //   where they stand, as the paper's English `{abstract}`/`{IEEEkeywords}` are the frontmatter's (21 papers;
  //   1504.02975, 1807.04577, 1807.04578, 1910.03090); in the two-column form (:3746, :3765), the whole block bold
  //   (`\bfseries\leavevmode…\textit{\ozetname}---`), the keywords bold italic; the paragraph begins bold, so the
  //   heading is a text of its own and what follows it is not set inside it.
  // The begin-document `\let`s run after this load, outside the guard below, so the hook calls a primitive that makes
  // them as the fallback guess themselves: a document's own `\AtBeginDocument{\newenvironment{proof}…}`, which runs
  // after the class's hook, replaces them (pdflatex with V1.8b: its own proof; 67x review probe g).
  DefPrimitive!(T_CS!("\\lx@IEEE@begindocument@proof"), None, {
    use latexml_core::definition::origin::{DefinitionOrigin, OriginGuard};
    let _fallback = OriginGuard::new(DefinitionOrigin::Fallback);
    Let!("\\proof", "\\IEEEproof");
    Let!("\\endproof", "\\endIEEEproof");
  });
  {
    use latexml_core::definition::origin::{DefinitionOrigin, OriginGuard};
    let _fallback = OriginGuard::new(DefinitionOrigin::Fallback);
    RawTeX!(r#"\newif\ifcenterfigcaptions
\ifCLASSOPTIONconference\centerfigcaptionstrue\else\centerfigcaptionsfalse\fi
\newif\ifnocalcleftmargin \nocalcleftmarginfalse
\def\@specialpapernotice{\relax}
\AtBeginDocument{\@ifundefined{proof}{\lx@IEEE@begindocument@proof}{}}
\def\thanksto#1{\lx@add@pubnote@thanks{#1}}
\def\ozetname{\"Ozet\c{c}e }
\def\IEEEanahtaradi{Anahtar Kelimeler}
\newenvironment{ozet}{\par\bfseries\noindent\textit{\ozetname}---\ignorespaces}{\par}
\newenvironment{IEEEanahtar}{\par\bfseries\noindent\textit{\IEEEanahtaradi}---\itshape\ignorespaces}{\par}"#);
  }
});

/// The brace level after `t`, from `level` before it: a begin-group opens one, an end-group closes one (the one reader
/// of a body's groups for [`split_top_level`], [`blocks_as_printed`] and [`grid_columns`]).
fn brace_level(level: i32, t: &Token) -> i32 {
  match t.get_catcode() {
    Catcode::BEGIN => level + 1,
    Catcode::END => level - 1,
    _ => level,
  }
}

/// Split `tokens` at each TOP-LEVEL (brace-depth 0) occurrence of `delim`, or of a `\let` copy of it (`\let\nx\and`:
/// a column in pdflatex as the command is; `Token::defined_as`).
fn split_top_level(tokens: &[Token], delim: &Token) -> Vec<Vec<Token>> {
  let mut parts: Vec<Vec<Token>> = vec![Vec::new()];
  let mut depth: i32 = 0;
  for t in tokens {
    depth = brace_level(depth, t);
    if depth == 0 && (t == delim || (t.is_executable() && t.defined_as(delim))) {
      parts.push(Vec::new());
    } else {
      parts.last_mut().unwrap().push(*t);
    }
  }
  parts
}

/// Whether an `\author` body calls a names block — by meaning: read after `blocks_as_printed` and `with_block_names`,
/// which write each document macro holding one as it prints and each `\let` copy as `\IEEEauthorblockN`, so the command
/// anywhere in the body (a block's group, a conditional's branch) is the route's fact. It was the body's spelling
/// (`contains("authorblockN")`), which missed a block in a macro or a copy and took any name spelling it (a document's
/// own `\authorblockN`) for the class's (P2-8a).
fn calls_names_block(body: &Tokens) -> bool {
  body.unlist_ref().contains(&T_CS!("\\IEEEauthorblockN"))
}

/// How many of the document's own macros [`holds_author_structure`] reads into from one call, and how many calls
/// [`blocks_as_printed`] expands in one body, at most.
const MEANING_READS: usize = 64;

/// Whether `token`, a macro of the document's own (`\newcommand`, `\def` in the document or its own files), holds author
/// structure by meaning — its definition, or a document macro it calls (recursively), spells a names block (a token
/// meaning `\IEEEauthorblockN`: the class's, `\authorblockN`, a `\let` copy), an `\and`-family separator (`\and`,
/// `\And`, `\AND`, or a token `\let` to one) or the grid's row break `\\` (or a `\let` copy). Read as the rehearsal reads a block's macros for `\let` copies of its
/// separators (`rehearsal::let_copies_of_bound_commands`): the definitions' tokens, no expansion.
fn holds_author_structure(token: &Token) -> bool {
  // (a copy means the command: `\authorblockN`, the kernel's `\And` and `\AND`, a document's `\let`)
  let structure = [T_CS!("\\IEEEauthorblockN"), T_CS!("\\and"), T_CS!("\\\\")];
  let document_macro = |t: &Token| -> Option<Tokens> {
    let defn = lookup_definition(t).ok().flatten()?;
    if !defn.is_expandable() || defn.get_origin() != definition::origin::DefinitionOrigin::Document
    {
      return None;
    }
    match defn.get_expansion() {
      Some(ExpansionBody::Tokens(body)) => Some(body.clone()),
      _ => None,
    }
  };
  let Some(body) = document_macro(token) else {
    return false;
  };
  let mut seen = rustc_hash::FxHashSet::<Token>::default();
  let mut pending: Vec<Token> = body.unlist();
  let mut macros = 1;
  while let Some(t) = pending.pop() {
    if !t.is_executable() || !seen.insert(t) {
      continue;
    }
    if structure.iter().any(|s| t == *s || t.defined_as(s)) {
      return true;
    }
    if macros < MEANING_READS
      && let Some(inner) = document_macro(&t)
    {
      macros += 1;
      pending.extend(inner.unlist());
    }
  }
  false
}

/// The block-call fact, by meaning: an `\author` body with each top-level call of a document macro that holds author
/// structure ([`holds_author_structure`]: names blocks, `\and`, `\\`) expanded, one step at a time, its expansion read again —
/// the body's blocks and separators as its macros print them, before the route is chosen and the block reader's token
/// steps (`author_block_body`) read it. A body held whole in a macro (`\newcommand\aut{\IEEEauthorblockN{Ann
/// Able}\IEEEauthorblockA{Univ A}\and…}`, `\author{\aut}`) took the plain route by spelling (`\lx@add@authors`): one
/// nameless creator, the names lost. A wrapper printing the separator (`\newcommand\pair[2]{#1\and#2}`) or a block
/// (`\newcommand\cell[2]{\IEEEauthorblockN{#1}\IEEEauthorblockA{#2}}`) took the block route with its call unread: the
/// grid's `\and`/`\\` and the bare-text wrap read `\pair`, `\cell` and their arguments' groups as text, the creators
/// scrambled and duplicated. A body no such macro reaches is returned as it is (a document macro printing layout only,
/// `\linebreakand`, is left to digestion); a macro inside a group (a block's argument) is the block's.
/// OXIDIZED_DESIGN_DIVERGENCES #128 (P2-8a).
fn blocks_as_printed(body: Tokens) -> Result<Tokens> {
  // (a quick read first: the top level's macros, without a mouth)
  let mut depth = 0i32;
  let reached = body.unlist_ref().iter().any(|t| {
    depth = brace_level(depth, t);
    depth == 0 && t.is_executable() && holds_author_structure(t)
  });
  if !reached {
    return Ok(body);
  }
  reading_from_mouth(Mouth::new("", None)?, move || {
    unread(body);
    let mut out: Vec<Token> = Vec::new();
    let mut depth = 0i32;
    let mut expansions = 0;
    while let Some(token) = read_token()? {
      depth = brace_level(depth, &token);
      if depth == 0
        && expansions < MEANING_READS
        && token.is_executable()
        && holds_author_structure(&token)
        && expand_once_partial(token)?
      {
        expansions += 1;
        continue;
      }
      out.push(token);
    }
    Ok(Tokens::new(out))
  })
}

/// An author-block body (`\IEEEauthorblockN`) as IEEEtran's `\author` runs it, the class's own and authblk's alike: a
/// genuine `\and`×`\\` grid transposed to row-major reading order first (arXiv:2403.16405; a single-row `\and` list is
/// returned unchanged), then any trailing bare-text run (a loose email after `\IEEEauthorblockA`) wrapped in
/// `\IEEEauthorblockA{}` so it attaches to the creator instead of leaking into the body (arXiv 1901.07768), then, when
/// `\IEEEmembership` is the class's (`ours`), each name's grade made its contact (`block_name_memberships`). The old
/// block names are read as the ones they mean first (`with_block_names`), so every step below reads only those.
fn author_block_body(body: Tokens, ours: bool) -> Result<Tokens> {
  let body = wrap_bare_author_block_text(transpose_ieee_author_grid(unnested_name_blocks(
    with_block_names(body),
  ))?);
  Ok(if ours {
    block_name_memberships(body)
  } else {
    body
  })
}

/// An author-block body with each old block name (`\authorblockN`, `\authorblockA`, IEEEtran.cls's before 1.8, the
/// binding's `\let`s of `\IEEEauthorblockN`, `\IEEEauthorblockA`), and any other `\let` copy of a block command
/// (`\let\N\IEEEauthorblockN`, P2-8a), written as the one it means, while it still means it (a document may define
/// either old name its own way, as `\authorrefmark` in `printed_refmarks`): read by name, the labelled path of authblk's
/// `\author` left `\authorblockN` its own (`\author[1]{\authorblockN{Ann Able}}` with `\affil[1]{…}`: a nameless
/// creator holding the affiliation, the names lost; OXIDIZED_DESIGN_DIVERGENCES #987), and the route by spelling took
/// a copy's body for the plain route or its calls for bare text (Bob Baker lost, Ann Able twice). Read before the route
/// (`\author`, `\lx@IEEE@authblk@author`) and again by `author_block_body`.
fn with_block_names(body: Tokens) -> Tokens {
  let names = [T_CS!("\\IEEEauthorblockN"), T_CS!("\\IEEEauthorblockA")];
  let means = |t: Token| {
    if !t.is_executable() || names.contains(&t) {
      return t;
    }
    names
      .iter()
      .find(|name| t.defined_as(name))
      .copied()
      .unwrap_or(t)
  };
  Tokens::new(body.unlist().into_iter().map(means).collect())
}

/// An author-block body with every names block written inside another's names (`\authorblockN{\authorblockN{Ann
/// Able, \IEEEmembership{Member, IEEE}, Bob Baker \\ and Cy Coe.} \\ \thanks{…}}`, 1511.03982) spliced into the outer
/// one, its command and braces dropped: one names block, read as any other. In IEEEtran.cls a names block only sets
/// its style (`\@IEEEauthorblockNstyle`), so pdflatex prints each name; Perl's `\IEEEauthorblockN` is `#1`
/// (IEEEtran.cls.ltxml:144), every name in one creator. Each block being an author call (`\lx@add@authors@append`),
/// the inner call ran inside the outer's argument and printed nothing there: every author lost without a Warning
/// (OXIDIZED_DESIGN_DIVERGENCES #987). An outer block holding its own `\IEEEauthorblockA` before an inner names
/// block is one left unclosed, not a wrapper (2605.20445): it is closed before that affiliation block instead
/// (`affiliation_before_names`).
fn unnested_name_blocks(body: Tokens) -> Tokens {
  fn is_names(t: &Token) -> bool { *t == T_CS!("\\IEEEauthorblockN") }
  // the argument group of the command just before `at`, past spaces: its open and close
  fn argument(tokens: &[Token], at: usize) -> Option<(usize, usize)> {
    let open = at
      + tokens[at..]
        .iter()
        .take_while(|t| **t == T_SPACE!())
        .count();
    if tokens.get(open)?.code != Catcode::BEGIN {
      return None;
    }
    let mut level = 0usize;
    let close = open
      + tokens[open..].iter().position(|t| {
        match t.code {
          Catcode::BEGIN => level += 1,
          Catcode::END => level -= 1,
          _ => {},
        }
        level == 0
      })?;
    Some((open, close))
  }
  // the first affiliation block at the top level of a names block's argument that a names block comes after
  fn affiliation_before_names(arg: &[Token]) -> Option<usize> {
    let mut j = 0;
    while j < arg.len() {
      if arg[j] == T_CS!("\\IEEEauthorblockA")
        && let Some((_, close)) = argument(arg, j + 1)
        && arg[close + 1..].iter().any(is_names)
      {
        return Some(j);
      }
      // (a group skipped whole)
      j = argument(arg, j).map_or(j + 1, |(_, close)| close + 1);
    }
    None
  }
  // `tokens` with each names command's argument spliced in (`inner`: its command and braces dropped), at any depth
  fn spliced(tokens: &[Token], inner: bool, out: &mut Vec<Token>) {
    let mut i = 0;
    while i < tokens.len() {
      if is_names(&tokens[i])
        && let Some((open, close)) = argument(tokens, i + 1)
      {
        if inner {
          spliced(&tokens[open + 1..close], true, out);
        } else if let Some(at) = affiliation_before_names(&tokens[open + 1..close]) {
          // An outer block left unclosed, its own affiliation block then the next author's blocks inside it
          // (`\IEEEauthorblockN{Ann Able\\ \IEEEauthorblockA{…}\\ \IEEEauthorblockN{Bob Baker} \IEEEauthorblockA{…}}`,
          // 2605.20445): closed before that affiliation block, the rest read at the top level, stacked blocks as
          // pdflatex prints them; spliced in, the next author's names were the first's affiliation.
          let at = open + 1 + at;
          out.extend_from_slice(&tokens[i..=open]);
          spliced(&tokens[open + 1..at], true, out);
          out.push(tokens[close]);
          spliced(&tokens[at..close], false, out);
        } else {
          out.extend_from_slice(&tokens[i..=open]);
          spliced(&tokens[open + 1..close], true, out);
          out.push(tokens[close]);
        }
        i = close + 1;
        continue;
      }
      out.push(tokens[i]);
      i += 1;
    }
  }
  let tokens = body.unlist();
  if tokens.iter().filter(|t| is_names(t)).count() < 2 {
    return Tokens::new(tokens);
  }
  let mut out = Vec::with_capacity(tokens.len());
  spliced(&tokens, false, &mut out);
  Tokens::new(out)
}

/// The grid's columns by meaning: `toks` cut at each separator the separator query reads at its own level
/// (`argument_separators`, OXIDIZED_DESIGN_DIVERGENCES #995: a written `\and`, a `\let` copy, `\And`/`\AND`, whatever it
/// means — IEEEtran's `\and` is its column whatever a document made it, as the spelling read it) whose own tokens stand
/// at the body's brace level (one inside a brace group, which the query counts at its level, stays in its cell, as
/// before). `None` where the query could not rehearse the body or a separator's tokens print something else too: the
/// caller cuts by `split_top_level`.
fn grid_columns(toks: &[Token]) -> Result<Option<Vec<Vec<Token>>>> {
  let Some(separators) = argument_separators(&Tokens::new(toks.to_vec()))? else {
    return Ok(None);
  };
  let mut level = Vec::with_capacity(toks.len());
  let mut depth = 0i32;
  for t in toks {
    level.push(depth);
    depth = brace_level(depth, t);
  }
  let mut cuts: Vec<(usize, usize)> = Vec::new();
  for separator in separators {
    let Some((start, end)) = separator.span else {
      return Ok(None);
    };
    if level.get(start) == Some(&0) {
      cuts.push((start, end));
    }
  }
  let mut columns = Vec::with_capacity(cuts.len() + 1);
  let mut from = 0;
  for (start, end) in cuts {
    columns.push(toks[from..start].to_vec());
    from = end;
  }
  columns.push(toks[from..].to_vec());
  Ok(Some(columns))
}

/// Transpose a genuine IEEEtran `\and`×`\\` author grid from column-major
/// (declaration order) to row-major (reading order).
///
/// html_feedback#6242-adjacent (arXiv:2403.16405): a conference `\author{}` lays
/// authors out as a 2-D grid — `\and` starts a new COLUMN, top-level `\\` a new ROW
/// within a column. Each `\IEEEauthorblockN` emits its creator in token order, so
/// LaTeXML (and Perl) linearize down each column = column-major, scrambling the
/// reading (row-major) order shown in the PDF and the arXiv metadata. Transpose so
/// creators come out row-major. Guarded TIGHTLY: only a REGULAR grid (≥2 columns,
/// every column the SAME number of rows ≥2) is reordered. A single-row `\and` list
/// (no top-level `\\`) and `\\` used only INSIDE `\IEEEauthorblockA{…\\…}` (nested,
/// depth>0) are left exactly as declared, and so is a grid with a cell naming no author (a column's affiliation rows
/// under its name, 2605.15804). Surpass-Perl divergence: OXIDIZED_DESIGN
/// #128 (shared upstream bug KNOWN_PERL_ERRORS #94). The grid's tokens are the body as its document macros print it
/// (`blocks_as_printed`), its columns the separators the query reads (`grid_columns`) and its rows its `\\`
/// (`split_top_level`, by meaning); the transposition stays here, on
/// tokens, until the reading holds the grid's events (each `\IEEEauthorblockN` is a ledger block of its own, the
/// separators between blocks unrecorded; P2-8a).
fn transpose_ieee_author_grid(body: Tokens) -> Result<Tokens> {
  let toks = body.unlist();
  let columns = match grid_columns(&toks)? {
    Some(columns) => columns,
    None => split_top_level(&toks, &T_CS!("\\and")),
  };
  if columns.len() < 2 {
    return Ok(Tokens::new(toks)); // not a multi-column grid
  }
  let grid: Vec<Vec<Vec<Token>>> = columns
    .iter()
    .map(|c| split_top_level(c, &T_CS!("\\\\")))
    .collect();
  let rows = grid[0].len();
  // Only a REGULAR grid transposes: same row count in every column, and ≥2 rows.
  if rows < 2 || grid.iter().any(|c| c.len() != rows) {
    return Ok(Tokens::new(toks));
  }
  // And a grid of authors: every cell that prints something (`prints_nothing`) names one. A row of a column's blocks
  // or text without a name is its author's, below the name as printed (`\IEEEauthorblockN{Ann Able} \textit{School
  // X}\\ \IEEEauthorblockA{Dept Y\\ City} \and …`, 2605.15804: transposed, every `\IEEEauthorblockA` of that row went
  // to the last author of the row above).
  let names = |cell: &Vec<Token>| {
    prints_nothing(&Tokens::new(cell.clone()))
      || cell.iter().any(|t| *t == T_CS!("\\IEEEauthorblockN"))
  };
  if !grid.iter().flatten().all(names) {
    return Ok(Tokens::new(toks));
  }
  let and = T_CS!("\\and");
  let mut out: Vec<Token> = Vec::new();
  for r in 0..rows {
    for col in &grid {
      let cell = &col[r];
      if cell.iter().all(|t| *t == T_SPACE!()) {
        continue;
      }
      if !out.is_empty() {
        out.push(and);
      }
      out.extend(cell.iter().copied());
    }
  }
  Ok(Tokens::new(out))
}

/// Wrap each top-level bare-text run in an IEEE author BLOCK body in `\IEEEauthorblockA{...}` so it attaches to the
/// current creator as an affiliation instead of digesting into the document body. A block macro
/// (`\IEEEauthorblockN/A`, `\authorblockN/A`) plus its balanced `{...}` argument is copied verbatim; `\and` passes
/// through untouched, a run that prints nothing (`prints_nothing`: spaces, line and paragraph breaks, `\vspace`) as its
/// spaces and braces, and a run of notes before an `\IEEEauthorblockA` as the document's notes. Without this, the
/// trailing bare email in `\author{\IEEEauthorblockN{…}\IEEEauthorblockA{…} \{…\}@host}` (the `\{`/`\}` are
/// catcode-12 literal-brace control symbols, so the email sits at brace-depth 0 = top level) is ordinary horizontal
/// text and `\lx@IEEE@author@blocks` digests it into the document body as the first paragraph. Real Perl LaTeXML
/// bundles the WHOLE author argument into `<personname>` (`\lx@add@authors`), so the tail never leaks —
/// GENUINE-RUST-ONLY. Witness arXiv 1901.07768 (`\{anuja, seth.gilbert, tankl\}@comp.nus.edu.sg`).
fn wrap_bare_author_block_text(body: Tokens) -> Tokens {
  let toks = body.unlist();
  let is_block = |t: &Token| *t == T_CS!("\\IEEEauthorblockN") || *t == T_CS!("\\IEEEauthorblockA");
  fn wrap(run: &mut Vec<Token>, out: &mut Vec<Token>, before_affiliation: bool) {
    // the spans of the run's line breaks: `\\` with its `*` and `[…]`, `\newline`
    let mut breaks: Vec<(usize, usize)> = Vec::new();
    let mut i = 0;
    while i < run.len() {
      if run[i] == T_CS!("\\\\") || run[i] == T_CS!("\\newline") {
        let double_backslash = run[i] == T_CS!("\\\\");
        let past_spaces = |mut at: usize| {
          while run.get(at) == Some(&T_SPACE!()) {
            at += 1;
          }
          at
        };
        let mut end = i + 1;
        if double_backslash && run.get(past_spaces(end)) == Some(&T_OTHER!("*")) {
          end = past_spaces(end) + 1;
        }
        if double_backslash && run.get(past_spaces(end)) == Some(&T_OTHER!("[")) {
          let open = past_spaces(end);
          if let Some(close) = run[open..].iter().position(|t| *t == T_OTHER!("]")) {
            end = open + close + 1;
          }
        }
        breaks.push((i, end));
        i = end;
      } else {
        i += 1;
      }
    }
    let in_break = |at: usize| breaks.iter().any(|&(from, to)| (from..to).contains(&at));
    let printed: Vec<Token> = run
      .iter()
      .enumerate()
      .filter(|(at, _)| !in_break(*at))
      .map(|(_, t)| *t)
      .collect();
    if prints_nothing(&Tokens::new(printed.clone())) {
      // Layout, not a block: the line break between two blocks (`\IEEEauthorblockN{…}\\ \IEEEauthorblockA{…}`,
      // 2410.19527), a paragraph break or a `\vspace` (1401.2118, 1705.03529). Wrapped, it was an affiliation printing
      // nothing that took the block's authors before the shared `\IEEEauthorblockA` (`annotate=new`), which then
      // reached the last only. Its spaces and its braces pass through: a group it opens or closes around a block
      // (`\authorblockN{…}\\ {\small \authorblockA{…}}\\ \and`, cs0507001) stays balanced.
      out.extend(run.iter().filter(|t| {
        matches!(
          t.get_catcode(),
          Catcode::SPACE | Catcode::BEGIN | Catcode::END
        )
      }));
    } else if let Some(notes) =
      document_notes(&printed).filter(|notes| before_affiliation && !notes.is_empty())
    {
      // The authors' notes between the names and their shared block (`\thanks{…}`, 1508.00691) stay the document's
      // notes, as in a block: wrapped, they were such an affiliation printing nothing. Notes no `\IEEEauthorblockA`
      // follows are wrapped as before, so the block reader still reads them (the e-mails of
      // `\thanks{E-mail: \{lins, shahmohm, helgamal\}@ece.osu.edu.}` after the names, 0710.2705; the note a name's mark
      // requests, 1504.01072).
      out.extend(notes);
    } else {
      // Printed text → route to the creator as an affiliation, without the line break it ends with
      // (`\IEEEauthorblockN{Ann Able} \textit{School X}\\ \IEEEauthorblockA{…}`, 2605.15804: kept, it sent every
      // later block to the last creator).
      let mut end = run.len();
      loop {
        while end > 0 && run[end - 1] == T_SPACE!() {
          end -= 1;
        }
        match breaks.iter().find(|&&(_, to)| to == end) {
          Some(&(from, _)) => end = from,
          None => break,
        }
      }
      run.truncate(end);
      out.push(T_CS!("\\IEEEauthorblockA"));
      out.push(T_BEGIN!());
      out.append(run);
      out.push(T_END!());
    }
    run.clear();
  }
  // A conditional around the blocks (`\ifdefined\hpcacameraready \IEEEauthorblockN{…}… \else … \fi`, 2407.02944)
  // keeps its own tokens as written — an `\if…` with its test, each `\else`/`\or`/`\fi` — and the bare text between
  // them is wrapped piece by piece: wrapped whole, the `\if` and its `\else`/`\fi` landed in different arguments. A
  // conditional the run holds whole (`\ifdefined\x a@b.c\fi` after a block) is wrapped with it, as any bare text.
  // `open_in_blocks` counts the conditionals a block's argument opened and left open (`\IEEEauthorblockA{… \if\x1
  // a@b.c} \vspace{-5mm} \else x@y.z \fi`, 2606.17718): the bare run's `\else`/`\fi` are theirs, so the run is wrapped
  // whole, as before the piecewise split (63x), and its own net depth closes them.
  fn flush(
    run: &mut Vec<Token>,
    out: &mut Vec<Token>,
    open_in_blocks: &mut i32,
    before_affiliation: bool,
  ) {
    let mut depth = 0i32;
    let mut whole = true;
    for t in run.iter() {
      match lookup_conditional(t) {
        Some(ConditionalType::If) => depth += 1,
        Some(ConditionalType::Fi) => depth -= 1,
        Some(ConditionalType::Else | ConditionalType::Or) if depth == 0 => whole = false,
        _ => {},
      }
      whole &= depth >= 0;
    }
    if *open_in_blocks > 0 {
      *open_in_blocks = (*open_in_blocks + depth).max(0);
      wrap(run, out, before_affiliation);
      return;
    }
    if whole && depth == 0 {
      wrap(run, out, before_affiliation);
      return;
    }
    let mut piece: Vec<Token> = Vec::new();
    let mut tokens = run.drain(..).peekable();
    while let Some(t) = tokens.next() {
      let Some(kind) = lookup_conditional(&t) else {
        piece.push(t);
        continue;
      };
      wrap(&mut piece, out, false);
      out.push(t);
      if kind != ConditionalType::If {
        continue;
      }
      // the test the primitive reads (tex.web §498-§513): `\ifdefined` one token, `\ifx`/`\if`/`\ifcat` two,
      // `\ifcsname` through `\endcsname`, a number's or dimension's up to the space ending it; a `\newif` one none.
      // The common shapes only: a relation spaced from its operands (`\ifnum\value{x} > 0`) or an `\expandafter` before
      // the `\if` leaves part of the test to be wrapped.
      let name = t.with_str(|s| s.to_string());
      match name.as_str() {
        "\\ifdefined" | "\\ifx" | "\\if" | "\\ifcat" => {
          let n = if name == "\\ifdefined" { 1 } else { 2 };
          out.extend(tokens.by_ref().take(n));
        },
        "\\ifcsname" => {
          for u in tokens.by_ref() {
            out.push(u);
            if u == T_CS!("\\endcsname") {
              break;
            }
          }
        },
        "\\ifnum" | "\\ifdim" | "\\ifodd" | "\\ifcase" | "\\ifeof" | "\\ifvoid" | "\\ifhbox"
        | "\\ifvbox" => {
          while let Some(u) = tokens.next_if(|u| lookup_conditional(u).is_none()) {
            out.push(u);
            if u == T_SPACE!() {
              break;
            }
          }
        },
        _ => {},
      }
    }
    drop(tokens);
    wrap(&mut piece, out, before_affiliation);
  }
  let mut out: Vec<Token> = Vec::new();
  let mut run: Vec<Token> = Vec::new();
  let mut open_in_blocks = 0i32;
  // An `\and` at the top level of an affiliation block's argument that a block macro follows starts the next author,
  // as pdflatex prints it (a brace missing after the first author's block: `\authorblockA{…\\ \and
  // \authorblockN{Hadas Kress-Gazit} \authorblockA{…}}}`, 2105.06537). The argument is closed before the `\and`, the
  // rest read as top-level blocks, and the argument's own closing brace — the first top-level `}` no run opened —
  // dropped (`unclosed`, `run_depth`): copied whole, the second author's blocks were the first author's too.
  let mut unclosed = 0usize;
  let mut run_depth = 0usize;
  let block_follows = |from: usize| {
    toks[from..]
      .iter()
      .find(|v| **v != T_SPACE!() && **v != T_CS!("\\par"))
      .is_some_and(is_block)
  };
  let mut i = 0;
  while i < toks.len() {
    let t = toks[i];
    if is_block(&t) {
      let affiliation = t == T_CS!("\\IEEEauthorblockA");
      flush(&mut run, &mut out, &mut open_in_blocks, affiliation);
      out.push(t);
      i += 1;
      // Copy the block's balanced `{...}` argument verbatim.
      if i < toks.len() && toks[i].get_catcode() == Catcode::BEGIN {
        let mut depth = 0;
        while i < toks.len() {
          let u = toks[i];
          if affiliation && depth == 1 && u == T_CS!("\\and") && block_follows(i + 1) {
            out.push(T_END!());
            unclosed += 1;
            break;
          }
          match u.get_catcode() {
            Catcode::BEGIN => depth += 1,
            Catcode::END => depth -= 1,
            _ => {},
          }
          match lookup_conditional(&u) {
            Some(ConditionalType::If) => open_in_blocks += 1,
            Some(ConditionalType::Fi) => open_in_blocks = (open_in_blocks - 1).max(0),
            _ => {},
          }
          out.push(u);
          i += 1;
          if depth == 0 {
            break;
          }
        }
      }
    } else if t == T_CS!("\\and") {
      flush(&mut run, &mut out, &mut open_in_blocks, false);
      out.push(t);
      i += 1;
    } else {
      match t.get_catcode() {
        Catcode::BEGIN => run_depth += 1,
        Catcode::END if run_depth > 0 => run_depth -= 1,
        Catcode::END if unclosed > 0 => {
          unclosed -= 1;
          i += 1;
          continue;
        },
        _ => {},
      }
      run.push(t);
      i += 1;
    }
  }
  flush(&mut run, &mut out, &mut open_in_blocks, false);
  Tokens::new(out)
}

/// An author block whose reference marks (`\IEEEauthorrefmark{n}`, `\authorrefmark{n}`) share a number with a mark of
/// another form that labels a line (`\thanks{$^1$TUM}`, `\IEEEauthorblockA{$^{1}$Univ A}`) with each reference mark
/// written as the superscript symbol it prints (IEEEtran.cls:4543-4546: `*`, `\dagger`, `\ddagger`, `\S`, `\P`, `\|`,
/// then doubled, then roman numerals): there its number is no label of that line — 2312.10807's `Oier
/// Mees$^3$\IEEEauthorrefmark{1}` prints "Mees3∗" under `\thanks{$^1$TUM}` and `\thanks{$^{*}$Equal contribution}`, its
/// `∗` that legend's and not TUM's (2409.20550's `Yanlin Wang$^{1}$\IEEEauthorrefmark{1}` over
/// `\IEEEauthorblockA{$^{1}$Sun Yat-sen University}` and `\thanks{\IEEEauthorrefmark{1} Corresponding author.}`) —, so
/// the block reads as one labelled with numbers and annotated with footnote symbols. Any other block is left as it is:
/// the numbers key its reference marks one to one with the symbols they print (`\lx@author@refmark@symbol`), as labels
/// — those that label its lines (1910.10075's `\IEEEauthorblockA{\IEEEauthorrefmark{1}University of Cambridge …}` under
/// names marked `\IEEEauthorrefmark{1}\textsuperscript{1}`, whose `1` labels no line) or none (2411.01611); under
/// `transmag` (:4541), IEEEoj (IEEEoj.cls:3186) and ieeeaccess's `\authorrefmark` (ieeeaccess.cls:453) they print
/// their numbers.
fn printed_refmarks(body: Tokens) -> Tokens {
  // (a class printing the number — `transmag`, IEEEoj's `\textsuperscript{#1}` — prints no symbol to read)
  if T_CS!("\\ifCLASSOPTIONtransmag").defined_as(&T_CS!("\\iftrue"))
    || T_CS!("\\lx@author@refmark@symbol").defined_as(&T_CS!("\\lx@author@refmark@arabic"))
  {
    return body;
  }
  // (`\authorrefmark` only as IEEEtran lets it, :6312; ieeeaccess.cls:453 prints its number)
  let legacy = T_CS!("\\authorrefmark").defined_as(&T_CS!("\\IEEEauthorrefmark"));
  let refmark =
    |t: &Token| *t == T_CS!("\\IEEEauthorrefmark") || (legacy && *t == T_CS!("\\authorrefmark"));
  let v = body.unlist_ref();
  // the end of the group opening at `at` (one past its close), or of the one token there
  let operand_end = |at: usize| -> usize {
    if v.get(at).is_some_and(|t| t.get_catcode() == Catcode::BEGIN) {
      let mut depth = 0usize;
      for (j, t) in v.iter().enumerate().skip(at) {
        match t.get_catcode() {
          Catcode::BEGIN => depth += 1,
          Catcode::END => {
            depth -= 1;
            if depth == 0 {
              return j + 1;
            }
          },
          _ => {},
        }
      }
      v.len()
    } else {
      (at + 1).min(v.len())
    }
  };
  let skip_spaces = |mut at: usize| {
    while v.get(at) == Some(&T_SPACE!()) {
      at += 1;
    }
    at
  };
  // (whether the mark at `at` leads a line — the block's first, one after `\\`, `\quad` or `\and`, a `\thanks{…}`'s or
  // an `\IEEEauthorblockA{…}`'s —, spaces and math shifts before it: the label of that line; 2602.11758's
  // `\textsuperscript{1}Tsinghua University\quad \textsuperscript{2}HKUST(Guangzhou)`)
  let leads_a_line = |at: usize| {
    let mut j = at;
    while j > 0 && matches!(v[j - 1].get_catcode(), Catcode::SPACE | Catcode::MATH) {
      j -= 1;
    }
    // (a line break, or the spacing and `\and` an affiliation list parts its entries by: `affil_splits`)
    if j == 0
      || [
        T_CS!("\\\\"),
        T_CS!("\\newline"),
        T_CS!("\\quad"),
        T_CS!("\\qquad"),
        T_CS!("\\and"),
        T_CS!("\\And"),
        T_CS!("\\AND"),
      ]
      .contains(&v[j - 1])
    {
      return true;
    }
    let mut k = j - 1;
    if v[k].get_catcode() != Catcode::BEGIN {
      return false;
    }
    while k > 0 && v[k - 1] == T_SPACE!() {
      k -= 1;
    }
    k > 0
      && [
        T_CS!("\\thanks"),
        T_CS!("\\footnote"),
        T_CS!("\\IEEEauthorblockA"),
        T_CS!("\\authorblockA"),
      ]
      .contains(&v[k - 1])
  };
  // (each reference mark's number and the tokens it spans; the numbers of the other marks that label a line)
  let mut marks: Vec<(usize, usize, u32)> = Vec::new();
  let mut line_numbers: Vec<u32> = Vec::new();
  let mut i = 0;
  while i < v.len() {
    let t = v[i];
    if refmark(&t) {
      let at = skip_spaces(i + 1);
      let end = operand_end(at);
      let digits: String = v[at..end]
        .iter()
        .filter(|t| {
          !matches!(
            t.get_catcode(),
            Catcode::BEGIN | Catcode::END | Catcode::SPACE
          )
        })
        .map(ToString::to_string)
        .collect();
      if let Ok(n) = digits.parse::<u32>() {
        marks.push((i, end, n));
      }
      i = end.max(i + 1);
    } else if t.get_catcode() == Catcode::SUPER || t == T_CS!("\\textsuperscript") {
      let end = operand_end(skip_spaces(i + 1));
      if leads_a_line(i) {
        // (its numbers: `$^{1,2}$`'s 1 and 2)
        let operand: String = v[i + 1..end].iter().map(ToString::to_string).collect();
        line_numbers.extend(
          operand
            .split(|c: char| !c.is_ascii_digit())
            .filter_map(|n| n.parse::<u32>().ok()),
        );
      }
      i = end.max(i + 1);
    } else {
      i += 1;
    }
  }
  if !marks.iter().any(|(_, _, n)| line_numbers.contains(n)) {
    return body;
  }
  let symbol = |n: u32| -> Vec<Token> {
    let one = |k: u32| match k {
      1 => vec![T_OTHER!("*")],
      2 => vec![T_CS!("\\dagger")],
      3 => vec![T_CS!("\\ddagger")],
      4 => vec![T_CS!("\\S")],
      5 => vec![T_CS!("\\P")],
      _ => vec![T_CS!("\\|")],
    };
    match n {
      1..=6 => one(n),
      7..=9 => [one(n - 6), one(n - 6)].concat(),
      _ => Vec::new(),
    }
  };
  let mut out: Vec<Token> = Vec::with_capacity(v.len());
  let mut from = 0;
  let mut in_math = false;
  for (start, end, n) in marks {
    for t in &v[from..start] {
      if t.get_catcode() == Catcode::MATH {
        in_math = !in_math;
      }
      out.push(*t);
    }
    from = end;
    let sym = symbol(n);
    // (a number of 10 or more prints its roman numeral, a lettered mark; 0 prints nothing)
    if sym.is_empty() {
      if n >= 10 {
        out.extend([T_CS!("\\textsuperscript"), T_BEGIN!()]);
        out.extend(roman_numeral(n).chars().map(|c| T_LETTER!(&c.to_string())));
        out.push(T_END!());
      }
      continue;
    }
    if !in_math {
      out.push(T_MATH!());
    }
    out.extend([T_SUPER!(), T_BEGIN!()]);
    out.extend(sym);
    out.push(T_END!());
    if !in_math {
      out.push(T_MATH!());
    }
  }
  out.extend_from_slice(&v[from..]);
  Tokens::new(out)
}

/// `n` in lower-case roman numerals (TeX's `\romannumeral`).
fn roman_numeral(mut n: u32) -> String {
  const PARTS: &[(u32, &str)] = &[
    (1000, "m"),
    (900, "cm"),
    (500, "d"),
    (400, "cd"),
    (100, "c"),
    (90, "xc"),
    (50, "l"),
    (40, "xl"),
    (10, "x"),
    (9, "ix"),
    (5, "v"),
    (4, "iv"),
    (1, "i"),
  ];
  let mut out = String::new();
  for (value, part) in PARTS {
    while n >= *value {
      out.push_str(part);
      n -= value;
    }
  }
  out
}

/// An author list with each `\IEEEmembership{…}` of its name lines — before the first `\\` of each `\and` group —
/// moved in front of the comma before it and made `\lx@IEEE@membership`, so the grade stays with the name it follows
/// when the list is split at its commas (`Yong Man Ro,~\IEEEmembership{Senior Member,~IEEE}`). A grade ending in a
/// comma carries the list's separator inside its braces (`Yun-Chih~Chen,~\IEEEmembership{Member,~IEEE,}
/// Yuan-Hao~Chang`, 2408.00327): that comma becomes the separator after it. An empty grade (`\IEEEmembership{}`,
/// 2408.01702) is left out. The names go on past a `\\` that the article parse reads as continuing them
/// ([`names_continue`]: `Wenming~Li, \\ Xiaochun~Ye, …`, 2408.01902); grades past the names keep `\IEEEmembership`,
/// which drops them.
fn name_memberships(tokens: Vec<Token>) -> Vec<Token> {
  let tokens = without_stop_after_grade(tokens);
  let spacing = |t: &Token| *t == T_SPACE!() || *t == T_ACTIVE!('~');
  // the end of the line opening at `from`: the next `\\` or `\and` outside groups
  let line_end = |from: usize| {
    let mut level = 0usize;
    tokens[from..]
      .iter()
      .position(|t| {
        match t.code {
          Catcode::BEGIN => level += 1,
          Catcode::END => level = level.saturating_sub(1),
          _ => {},
        }
        level == 0 && (*t == T_CS!("\\\\") || *t == T_CS!("\\and"))
      })
      .map_or(tokens.len(), |at| from + at)
  };
  let mut out: Vec<Token> = Vec::new();
  let mut depth = 0usize;
  let mut in_names = true;
  let mut names_start = 0;
  let mut i = 0;
  while i < tokens.len() {
    let t = tokens[i];
    if depth == 0 {
      if t == T_CS!("\\and") {
        in_names = true;
        names_start = i + 1;
      } else if t == T_CS!("\\\\") && in_names && grade_only_line(&tokens[i + 1..line_end(i + 1)]) {
        // (a line holding only the last name's grade, its notes aside, is that name's: `and~Jussi
        // Haapola,\\~\IEEEmembership{Member,~IEEE,}\thanks{…}`, 2202.10869 — the break no part of the names, the grade
        // read as any other, and lost had the line been read as the block's)
        i += 1;
        continue;
      } else if t == T_CS!("\\\\") {
        in_names = in_names
          && names_continue(
            &Tokens::new(tokens[names_start..i].to_vec()),
            &Tokens::new(tokens[i + 1..line_end(i + 1)].to_vec()),
          );
      }
    }
    // (a note's text is no name line: `\thanks{… \IEEEmembership{…}}` keeps the grade it names, dropped)
    if [
      T_CS!("\\thanks"),
      T_CS!("\\footnote"),
      T_CS!("\\IEEEcompsocitemizethanks"),
    ]
    .contains(&t)
    {
      let mut j = i + 1;
      while j < tokens.len() && tokens[j] == T_SPACE!() {
        j += 1;
      }
      if j < tokens.len() && tokens[j].code == Catcode::BEGIN {
        let mut level = 0usize;
        while j < tokens.len() {
          match tokens[j].code {
            Catcode::BEGIN => level += 1,
            Catcode::END => level = level.saturating_sub(1),
            _ => {},
          }
          j += 1;
          if level == 0 {
            break;
          }
        }
        out.extend_from_slice(&tokens[i..j]);
        i = j;
        continue;
      }
    }
    // (inside a wrapper of the names too: `{{Ann Able},~\IEEEmembership{Senior Member, IEEE}, …}`, 2112.09769;
    // `\uppercase{K. Liu\authorrefmark{2}, J. Chen\authorrefmark{2}, \IEEEmembership{Senior, IEEE} R.C. de Lamare…}`,
    // ieeeaccess 2209.12464)
    if in_names && t == T_CS!("\\IEEEmembership") {
      let mut j = i + 1;
      while j < tokens.len() && tokens[j] == T_SPACE!() {
        j += 1;
      }
      if j < tokens.len() && tokens[j].code == Catcode::BEGIN {
        let start = j;
        let mut level = 0usize;
        while j < tokens.len() {
          match tokens[j].code {
            Catcode::BEGIN => level += 1,
            Catcode::END => {
              level -= 1;
              if level == 0 {
                break;
              }
            },
            _ => {},
          }
          j += 1;
        }
        if level == 0 && j < tokens.len() {
          let mut grade: Vec<Token> = tokens[start + 1..j].to_vec();
          while grade.last().is_some_and(spacing) {
            grade.pop();
          }
          let mut separated = grade.last() == Some(&T_OTHER!(","));
          if separated {
            grade.pop();
            while grade.last().is_some_and(spacing) {
              grade.pop();
            }
          }
          let kept = out
            .iter()
            .rposition(|prev| !spacing(prev))
            .map_or(0, |at| at + 1);
          if kept > 0 && out[kept - 1] == T_OTHER!(",") {
            out.truncate(kept - 1);
            separated = true;
          }
          if grade.iter().any(|t| !spacing(t)) {
            out.push(T_CS!("\\lx@IEEE@membership"));
            out.push(T_BEGIN!());
            out.extend(grade);
            out.push(T_END!());
          }
          // (unless the author's own comma follows: `Zeng,~\IEEEmembership{Senior Member, IEEE},`, 2408.01956)
          let next = tokens[j + 1..].iter().find(|t| !spacing(t));
          if separated && next != Some(&T_OTHER!(",")) {
            out.push(T_OTHER!(","));
          }
          i = j + 1;
          continue;
        }
      }
    }
    match t.code {
      Catcode::BEGIN => depth += 1,
      Catcode::END => depth = depth.saturating_sub(1),
      _ => {},
    }
    out.push(t);
    i += 1;
  }
  out
}

/// Whether `line` holds one `\IEEEmembership{…}` and else only spacing or authors' notes alone ([`notes_only_line`]:
/// `\thanks{…}`, `\footnote{…}`): a grade on the line after its name ([`name_memberships`]).
fn grade_only_line(line: &[Token]) -> bool {
  let spacing = |t: &Token| *t == T_SPACE!() || *t == T_ACTIVE!('~') || *t == T_CS!("\\par");
  let mut grades = 0;
  let mut rest: Vec<Token> = Vec::with_capacity(line.len());
  let mut depth = 0usize;
  let mut i = 0;
  while i < line.len() {
    let t = line[i];
    if depth == 0 && t == T_CS!("\\IEEEmembership") {
      let open = i
        + 1
        + line[i + 1..]
          .iter()
          .take_while(|t| **t == T_SPACE!())
          .count();
      if line.get(open).is_none_or(|t| t.code != Catcode::BEGIN) {
        return false;
      }
      let mut level = 0usize;
      let Some(close) = line[open..].iter().position(|t| {
        match t.code {
          Catcode::BEGIN => level += 1,
          Catcode::END => level -= 1,
          _ => {},
        }
        level == 0
      }) else {
        return false;
      };
      grades += 1;
      i = open + close + 1;
      continue;
    }
    match t.code {
      Catcode::BEGIN => depth += 1,
      Catcode::END => depth = depth.saturating_sub(1),
      _ => {},
    }
    rest.push(t);
    i += 1;
  }
  grades == 1 && (rest.iter().all(spacing) || notes_only_line(&rest))
}

/// `tokens` without the full stop or semicolon that closes them right after a grade: one at the top level that only
/// what prints no text follows — spaces, `\par`, `\\`, `\vspace{…}` and authors' notes (`\thanks{…}`, `\footnote{…}`)
/// — and only such material parts from an `\IEEEmembership{…}` before it, or from a grade written as italic text
/// ([`grade_text`]: `{\em Senior Member, IEEE}.`, `\emph{…}.`, `\textit{…}.`, 1810.08035): `…,\\ and Farshid Alambeigi$^{1}$
/// \IEEEmembership{Member, IEEE}\thanks{…}.}` (2405.17600), `…, Jiangchuan Liu, \IEEEmembership{Fellow, IEEE}. }` in an
/// `\IEEEauthorblockN` (2405.10825), `…~\IEEEmembership{Fellow,~IEEE}.` + a blank line, notes and a `\\` (2510.15773),
/// `…~\IEEEmembership{Fellow,~IEEE}` + a blank line, `\vspace{-10mm}` and notes, then `.` (2112.14209). The stop
/// closes the block's sentence and is no part of the last author's names or grade: every reading of the names after it
/// — whether the line after a `\\` continues them ([`names_continue`]), the block's split, a grade rejoining its name —
/// took the lone "." for a word, so the last line's grades were dropped and its authors became every author's
/// affiliation (2112.14209, 2409.11755, 2510.15773). A stop after any other text (an affiliation's) stays.
///
/// The stop parts the notes after it from the names, and the shared reader reads them by that: the stop goes only where
/// it is sure no note changes hands, by the reader's own tests of a mark — any mark on the names, `author_mark_position`
/// on the block with its notes taken out (which sees a `\let` alias of `\textsuperscript`, a mark in a group or an
/// `\mbox`); a mark on the names that reads as a label, `leads_with_mark` on the names from that mark (`$^{1}$`,
/// `\IEEEauthorrefmark{2}`, `$^{1,*}$`; not `$^{*}$`, `$^{\dag}$`); a note after the grade led by a label,
/// `leads_with_mark` (`$^{1}$…`, `\up{1}…`; a symbol, `$^{*}$Corresponding author`, is a note's own mark, no
/// affiliation's). Three branches. With neither the names nor a note marked, the stop goes and the notes after the
/// grade move onto a line of their own, after a `\\`, which reads as no names and yields the document's notes
/// (2510.15773's own `\\`-lines): left after the grade, they would follow the last name and go to its author by
/// position alone (all six of 2112.14209's, 2409.11755's one covering all five authors, 2605.22853's "The authors are
/// with"). With a label on the names and a note after the grade led by one, the stop goes and the notes stay, read by
/// their marks as the base reads them with the stop (2405.17600, whose first marks, `$^{*1}$`, lead with a
/// symbol; Sarah Go's `$^{1}$` is the label; the 86d r2 re-review's m0-m3, m5). Every other block is left exactly
/// as the base reads it: marks on the names alone (`Ann Able$^{*}$` over `\thanks{$^{*}$Corresponding author.}`,
/// the r2 re-review's m4, n4b), on the notes alone, or names
/// marked only by symbols over a note led by a number (the same block with `\thanks{$^{1}$All authors are with X.}`,
/// the r3 re-review's a9, a12: read as both-marked, the correspondence note went to the last author and the numbered
/// note became a creator with no name). There, without the stop, the notes went to the last author (n4b's
/// correspondence note, whose `*` marks the first: the shared reader ties no such note to a non-last author's mark,
/// the observer's S2c item). A note after a `\\` that already follows the grade is on such a line. Notes after a grade with no stop
/// keep their reading, Perl's too (the last author's; KNOWN_PERL_ERRORS #534).
fn without_stop_after_grade(mut tokens: Vec<Token>) -> Vec<Token> {
  let notes = [
    T_CS!("\\thanks"),
    T_CS!("\\footnote"),
    T_CS!("\\IEEEcompsocitemizethanks"),
  ];
  let spacing = |t: &Token| {
    *t == T_SPACE!() || *t == T_ACTIVE!('~') || *t == T_CS!("\\par") || *t == T_CS!("\\\\")
  };
  // the start of the group ending just before `end` that prints no text there (a `\thanks{…}`, `\footnote{…}`,
  // `\IEEEcompsocitemizethanks{…}`, `\vspace{…}`, `\vspace*{…}`), at its command
  let note_before = |tokens: &[Token], end: usize| -> Option<usize> {
    if end == 0 || tokens[end - 1].code != Catcode::END {
      return None;
    }
    let mut level = 0usize;
    let open = (0..end).rev().find(|&j| {
      match tokens[j].code {
        Catcode::END => level += 1,
        Catcode::BEGIN => level = level.saturating_sub(1),
        _ => {},
      }
      level == 0
    })?;
    let mut before = open;
    while before > 0 && tokens[before - 1] == T_SPACE!() {
      before -= 1;
    }
    if before > 1 && tokens[before - 1] == T_OTHER!("*") && tokens[before - 2] == T_CS!("\\vspace")
    {
      return Some(before - 2);
    }
    (before > 0 && (notes.contains(&tokens[before - 1]) || tokens[before - 1] == T_CS!("\\vspace")))
      .then(|| before - 1)
  };
  // back over spaces and notes from `end`
  let back = |tokens: &[Token], mut end: usize| -> usize {
    loop {
      if end > 0 && spacing(&tokens[end - 1]) {
        end -= 1;
      } else if let Some(start) = note_before(tokens, end) {
        end = start;
      } else {
        return end;
      }
    }
  };
  let end = back(&tokens, tokens.len());
  if end == 0 || ![T_OTHER!("."), T_OTHER!(";")].contains(&tokens[end - 1]) {
    return tokens;
  }
  let stop = end - 1;
  let grade_end = back(&tokens, stop);
  // the grade's group, and `\IEEEmembership` before it
  if grade_end == 0 || tokens[grade_end - 1].code != Catcode::END {
    return tokens;
  }
  let mut level = 0usize;
  let Some(open) = (0..grade_end).rev().find(|&j| {
    match tokens[j].code {
      Catcode::END => level += 1,
      Catcode::BEGIN => level = level.saturating_sub(1),
      _ => {},
    }
    level == 0
  }) else {
    return tokens;
  };
  let mut before = open;
  while before > 0 && tokens[before - 1] == T_SPACE!() {
    before -= 1;
  }
  // (or a grade written as italic text: `{\em Member, IEEE}.`, `\emph{…}.`, `\textit{…}.`, 1810.08035)
  let membership = before > 0 && tokens[before - 1] == T_CS!("\\IEEEmembership");
  let italic = before > 0 && [T_CS!("\\emph"), T_CS!("\\textit")].contains(&tokens[before - 1])
    || tokens[open + 1..grade_end - 1]
      .iter()
      .find(|t| **t != T_SPACE!())
      .is_some_and(|t| [T_CS!("\\em"), T_CS!("\\it"), T_CS!("\\itshape")].contains(t));
  if !membership && !(italic && grade_text(&tokens[open + 1..grade_end - 1])) {
    return tokens;
  }
  // (each note's argument at the top level from `from`, and `tokens` with every note taken out, at any depth)
  let note_args = |tokens: &[Token], from: usize| -> Vec<Tokens> {
    let mut out = Vec::new();
    let mut level = 0usize;
    let mut i = from;
    while i < tokens.len() {
      if level == 0 && notes.contains(&tokens[i]) {
        let (arg, next) = note_argument(tokens, i + 1);
        out.push(Tokens::new(arg));
        i = next;
        continue;
      }
      match tokens[i].code {
        Catcode::BEGIN => level += 1,
        Catcode::END => level = level.saturating_sub(1),
        _ => {},
      }
      i += 1;
    }
    out
  };
  let without_notes = |tokens: &[Token]| -> Tokens {
    let mut out = Vec::new();
    let mut i = 0;
    while i < tokens.len() {
      if notes.contains(&tokens[i]) {
        i = note_argument(tokens, i + 1).1;
      } else {
        out.push(tokens[i]);
        i += 1;
      }
    }
    Tokens::new(out)
  };
  // marks by the shared reader's own tests: any mark on the names (`author_mark_position`, which sees a `\let` alias,
  // a grouped mark, `\mbox{$^{1}$}`); a mark on the names that reads as a label, by the test a note's is read by
  // (`leads_with_mark` on the names from that mark: `$^{1}$`, `\IEEEauthorrefmark{2}`, `$^{1,*}$`, not `$^{*}$`,
  // `$^{\dag}$`); and a note leading with a label (`leads_with_mark`)
  let names = without_notes(&tokens).unlist();
  let names_any = author_mark_position(&Tokens::new(names.clone())).is_some();
  let mut names_labelled = false;
  let mut from = 0;
  while let Some(at) = author_mark_position(&Tokens::new(names[from..].to_vec())) {
    let at = from + at - 1;
    if leads_with_mark(&Tokens::new(names[at..].to_vec())) {
      names_labelled = true;
      break;
    }
    from = at + 1;
  }
  let notes_marked = note_args(&tokens, grade_end).iter().any(leads_with_mark);
  // (neither the names nor a note marked; or both marked with labels; else the block as the base reads it)
  let unmarked = !names_any && !notes_marked;
  if !unmarked && !(names_labelled && notes_marked) {
    return tokens;
  }
  tokens.remove(stop);
  if unmarked {
    // a note after the grade before any `\\` (the tail holds only spacing, `\vspace` and notes, at the top level)
    let mut level = 0usize;
    for t in &tokens[grade_end..] {
      if level == 0 && *t == T_CS!("\\\\") {
        break;
      }
      if level == 0 && notes.contains(t) {
        tokens.insert(grade_end, T_CS!("\\\\"));
        break;
      }
      match t.code {
        Catcode::BEGIN => level += 1,
        Catcode::END => level = level.saturating_sub(1),
        _ => {},
      }
    }
  }
  tokens
}

/// Whether the text `tokens` print, commands aside, is a membership grade: the names reader's own test
/// (`grade_words(…, true)`, classify/name_text.rs: its words, cut at spaces and commas, all a grade's rank or a
/// society, one of them `Member` or `Fellow`, `Senior Member, IEEE`) of the letters and spaces they print.
fn grade_text(tokens: &[Token]) -> bool {
  let text: String = tokens
    .iter()
    .filter_map(|t| match t.code {
      Catcode::LETTER | Catcode::OTHER => Some(t.to_string()),
      // (a tie as a space: `Senior~Member,~IEEE`)
      Catcode::SPACE | Catcode::ACTIVE => Some(" ".to_string()),
      _ => None,
    })
    .collect();
  grade_words(&text, true)
}

/// The argument of the note whose command ends just before `at` (`\thanks[…]{…}`): its tokens, braces dropped, and
/// the position after it; past spaces and an optional `[…]`, a group or else one token.
fn note_argument(tokens: &[Token], mut at: usize) -> (Vec<Token>, usize) {
  let skip_spaces = |at: &mut usize| {
    while *at < tokens.len() && tokens[*at] == T_SPACE!() {
      *at += 1;
    }
  };
  skip_spaces(&mut at);
  if at < tokens.len() && tokens[at] == T_OTHER!("[") {
    while at < tokens.len() && tokens[at] != T_OTHER!("]") {
      at += 1;
    }
    at += 1;
    skip_spaces(&mut at);
  }
  if at >= tokens.len() {
    return (Vec::new(), tokens.len());
  }
  if tokens[at].code != Catcode::BEGIN {
    return (vec![tokens[at]], at + 1);
  }
  let mut level = 0usize;
  for j in at..tokens.len() {
    match tokens[j].code {
      Catcode::BEGIN => level += 1,
      Catcode::END => {
        level -= 1;
        if level == 0 {
          return (tokens[at + 1..j].to_vec(), j + 1);
        }
      },
      _ => {},
    }
  }
  (tokens[at + 1..].to_vec(), tokens.len())
}

/// An explicit author-block body with the grades of each `\IEEEauthorblockN{…}` name line made its author's
/// (`name_memberships`); a grade outside the name blocks stays `\IEEEmembership`, dropped.
fn block_name_memberships(body: Tokens) -> Tokens {
  let tokens = body.unlist();
  let mut out: Vec<Token> = Vec::new();
  let mut i = 0;
  while i < tokens.len() {
    let t = tokens[i];
    out.push(t);
    i += 1;
    // (its old name too, read as this one by `with_block_names`: 1511.03982's grades)
    if t != T_CS!("\\IEEEauthorblockN") {
      continue;
    }
    while i < tokens.len() && tokens[i] == T_SPACE!() {
      out.push(tokens[i]);
      i += 1;
    }
    if i >= tokens.len() || tokens[i].code != Catcode::BEGIN {
      continue;
    }
    let start = i;
    let mut level = 0usize;
    while i < tokens.len() {
      match tokens[i].code {
        Catcode::BEGIN => level += 1,
        Catcode::END => {
          level -= 1;
          if level == 0 {
            break;
          }
        },
        _ => {},
      }
      i += 1;
    }
    if level != 0 || i >= tokens.len() {
      out.extend(tokens[start..].iter().copied());
      return Tokens::new(out);
    }
    out.push(T_BEGIN!());
    out.extend(name_memberships(tokens[start + 1..i].to_vec()));
    out.push(T_END!());
    i += 1;
  }
  Tokens::new(out)
}

/// Is `\IEEEmembership` still this binding's (`\lx@IEEE@membership@drop`)? A document's own definition keeps its
/// meaning and its grades their place.
fn class_membership() -> Result<bool> {
  Ok(
    lookup_definition(&T_CS!("\\IEEEmembership"))?.is_some_and(|def| {
      matches!(def.get_expansion(), Some(ExpansionBody::Tokens(body))
        if body.unlist_ref().first() == Some(&T_CS!("\\lx@IEEE@membership@drop")))
    }),
  )
}
