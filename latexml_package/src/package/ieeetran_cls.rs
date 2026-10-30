//! IEEEtran.cls — IEEE Transactions document class
//! Perl: IEEEtran.cls.ltxml — 458 lines
use crate::{
  engine::{
    base_utilities::names_continue,
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
/// the last author)
pub const FRONTMATTER_CONVENTIONS: FmConventions = FmConventions {
  name: "IEEEtran",
  binding: "IEEEtran.cls",
  unmarked_notes: true,
  cite: "IEEEtran.cls:4756 `\\def\\thefootnote{}`; OD #837 S2a.2 r6 (2001.02600, 1803.04996, 2504.08644, 2602.13176)",
  written: false,
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
  Let!("\\IEEEpeerreviewmaketitle", "\\maketitle");
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
  DefMacro!("\\lx@IEEE@author@blocks{}",
    "\\def\\@author{#1}\\begingroup\\def\\and{}#1\\endgroup");
  DefMacro!("\\lx@IEEE@author@plain{}",
    "\\def\\@shortauthor{}\\gdef\\shortauthor{}\\def\\@author{#1}\\lx@add@authors{#1}");
  // Override `\author`: dispatch to explicit-block handling when the body uses
  // `\IEEEauthorblockN`, else to the standard name-splitter. `\author` is locked
  // in the kernel; re-lock so a user `\renewcommand` can't shadow this.
  DefMacro!("\\author[]{}", sub[(_short, body)] {
    // (a document's own `\IEEEmembership` keeps its meaning: `\def\IEEEmembership#1{\textit{#1}}` in 2408.00647)
    let ours = class_membership()?;
    let (target, body) = if body.to_string().contains("authorblockN") {
      // Transpose a genuine `\and`×`\\` grid to row-major reading order first
      // (arXiv:2403.16405); a single-row `\and` list is returned unchanged. Then
      // wrap any trailing bare-text run (a loose email after `\IEEEauthorblockA`) in
      // `\IEEEauthorblockA{}` so it attaches to the creator instead of leaking into
      // the body (arXiv 1901.07768).
      (T_CS!("\\lx@IEEE@author@blocks"),
       {
         let body = wrap_bare_author_block_text(transpose_ieee_author_grid(body));
         if ours { block_name_memberships(body) } else { body }
       })
    } else {
      (T_CS!("\\lx@IEEE@author@plain"), if ours { Tokens::new(name_memberships(body.unlist())) } else { body })
    };
    Ok(Invocation!(target, vec![Some(body)]))
  }, locked => true);

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
  // witness 2404.07847). Take the argument and render it as a superscript
  // (beyond-Perl): "Lei Chen²", matching pdfTeX and preserving the marker.
  DefMacro!("\\IEEEauthorrefmark{}", "\\textsuperscript{#1}");
  def_macro_noop("\\IEEEtriggeratref{}")?;
  DefMacro!("\\IEEEpubid{}", "\\lx@add@pubnote[role=pubid]{pubid: #1}");
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

/// Split `tokens` at each TOP-LEVEL (brace-depth 0) occurrence of `delim`.
fn split_top_level(tokens: &[Token], delim: &Token) -> Vec<Vec<Token>> {
  let mut parts: Vec<Vec<Token>> = vec![Vec::new()];
  let mut depth: i32 = 0;
  for t in tokens {
    match t.get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END => depth -= 1,
      _ => {},
    }
    if depth == 0 && t == delim {
      parts.push(Vec::new());
    } else {
      parts.last_mut().unwrap().push(*t);
    }
  }
  parts
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
/// depth>0) are left exactly as declared. Surpass-Perl divergence: OXIDIZED_DESIGN
/// #128 (shared upstream bug KNOWN_PERL_ERRORS #94).
fn transpose_ieee_author_grid(body: Tokens) -> Tokens {
  let toks = body.unlist();
  let columns = split_top_level(&toks, &T_CS!("\\and"));
  if columns.len() < 2 {
    return Tokens::new(toks); // not a multi-column grid
  }
  let grid: Vec<Vec<Vec<Token>>> = columns
    .iter()
    .map(|c| split_top_level(c, &T_CS!("\\\\")))
    .collect();
  let rows = grid[0].len();
  // Only a REGULAR grid transposes: same row count in every column, and ≥2 rows.
  if rows < 2 || grid.iter().any(|c| c.len() != rows) {
    return Tokens::new(toks);
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
  Tokens::new(out)
}

/// Wrap each top-level bare-text run in an IEEE author BLOCK body in
/// `\IEEEauthorblockA{...}` so it attaches to the current creator as an affiliation
/// instead of digesting into the document body. A block macro (`\IEEEauthorblockN/A`,
/// `\authorblockN/A`) plus its balanced `{...}` argument is copied verbatim; `\and`
/// and whitespace-only gaps pass through untouched. Without this, the trailing bare
/// email in `\author{\IEEEauthorblockN{…}\IEEEauthorblockA{…} \{…\}@host}` (the `\{`/`\}`
/// are catcode-12 literal-brace control symbols, so the email sits at brace-depth 0 =
/// top level) is ordinary horizontal text and `\lx@IEEE@author@blocks` digests it into
/// the document body as the first paragraph. Real Perl LaTeXML bundles the WHOLE author
/// argument into `<personname>` (`\lx@add@authors`), so the tail never leaks —
/// GENUINE-RUST-ONLY. Witness arXiv 1901.07768 (`\{anuja, seth.gilbert, tankl\}@comp.nus.edu.sg`).
fn wrap_bare_author_block_text(body: Tokens) -> Tokens {
  let toks = body.unlist();
  let is_block = |t: &Token| {
    *t == T_CS!("\\IEEEauthorblockN")
      || *t == T_CS!("\\IEEEauthorblockA")
      || *t == T_CS!("\\authorblockN")
      || *t == T_CS!("\\authorblockA")
  };
  fn wrap(run: &mut Vec<Token>, out: &mut Vec<Token>) {
    // The line break between two blocks (`\IEEEauthorblockN{…}\\ \IEEEauthorblockA{…}`, 2410.19527) is layout:
    // wrapped, it was an empty affiliation of the block's authors.
    let gap = |t: &Token| *t == T_SPACE!() || *t == T_CS!("\\\\") || *t == T_CS!("\\newline");
    if run.iter().any(|t| !gap(t)) {
      // Non-empty bare run → route to the creator as an affiliation.
      out.push(T_CS!("\\IEEEauthorblockA"));
      out.push(T_BEGIN!());
      out.append(run);
      out.push(T_END!());
    } else {
      // whitespace-only gap: pass its spaces through
      out.extend(run.iter().filter(|t| **t == T_SPACE!()));
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
  fn flush(run: &mut Vec<Token>, out: &mut Vec<Token>, open_in_blocks: &mut i32) {
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
      wrap(run, out);
      return;
    }
    if whole && depth == 0 {
      wrap(run, out);
      return;
    }
    let mut piece: Vec<Token> = Vec::new();
    let mut tokens = run.drain(..).peekable();
    while let Some(t) = tokens.next() {
      let Some(kind) = lookup_conditional(&t) else {
        piece.push(t);
        continue;
      };
      wrap(&mut piece, out);
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
    wrap(&mut piece, out);
  }
  let mut out: Vec<Token> = Vec::new();
  let mut run: Vec<Token> = Vec::new();
  let mut open_in_blocks = 0i32;
  let mut i = 0;
  while i < toks.len() {
    let t = toks[i];
    if is_block(&t) {
      flush(&mut run, &mut out, &mut open_in_blocks);
      out.push(t);
      i += 1;
      // Copy the block's balanced `{...}` argument verbatim.
      if i < toks.len() && toks[i].get_catcode() == Catcode::BEGIN {
        let mut depth = 0;
        while i < toks.len() {
          let u = toks[i];
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
      flush(&mut run, &mut out, &mut open_in_blocks);
      out.push(t);
      i += 1;
    } else {
      run.push(t);
      i += 1;
    }
  }
  flush(&mut run, &mut out, &mut open_in_blocks);
  Tokens::new(out)
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

/// `tokens` without the full stop or semicolon that closes them right after a grade: one at the top level that only
/// what prints no text follows — spaces, `\par`, `\\`, `\vspace{…}` and authors' notes (`\thanks{…}`, `\footnote{…}`)
/// — and only such material parts from an `\IEEEmembership{…}` before it: `…,\\ and Farshid Alambeigi$^{1}$
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
  if before == 0 || tokens[before - 1] != T_CS!("\\IEEEmembership") {
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
