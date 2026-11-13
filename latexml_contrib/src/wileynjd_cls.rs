//! The Wiley NJD family of classes: WileyNJD-v1, WileyNJD-v2, WileyNJDv5 (and `wileyNJDv5`), and WileyASNA-v1
//! ([`crate::wileyasna_cls`]), one binding for every version (`wiley_njd_family`).
//!
//! These Wiley journal classes share a common set of frontmatter macros
//! (\corres, \authormark, \jnlcitation, \cname, \cyear, \vol, \DOI,
//! \papertype, ...). Route to OmniBus and gobble the frontmatter so
//! downstream content renders cleanly.
use latexml_package::prelude::*;

use crate::shipped_class::{
  class_code, class_code_contains, class_option_given, class_text_loads,
  require_shipped_algorithm_packages, shipped_class_text,
};

LoadDefinitions!({
  let class = Expand!(Tokens!(T_CS!("\\@currname"))).to_string();
  wiley_njd_family(&class)?;
});

/// The definitions every Wiley NJD-family binding shares; `class` is the class the paper asked for (`\@currname`:
/// `WileyNJD-v2`, `WileyNJDv5`, `WileyASNA-v1`, …). The versions agree but for the box environments and the
/// theorem set, where the paper's copy decides (the class as its name says without one).
pub(crate) fn wiley_njd_family(class: &str) -> Result<()> {
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  // amssymb pulls in \gtrsim/\lesssim and other relation symbols Wiley
  // journal papers commonly use without an explicit \usepackage{amssymb}.
  // Witness 2406.06228 (WileyASNA-v1 paper).
  RequirePackage!("amssymb");
  RequirePackage!("amsthm");
  RequirePackage!("xcolor");
  // Wiley journals frequently load hyperref; mirror so cross-refs work.
  RequirePackage!("hyperref");
  // Wiley templates assume booktabs / graphicx are available —
  // eager-load so docs that use \toprule/\midrule/\bottomrule and
  // \includegraphics without explicit \usepackage don't error.
  // Witness 2504.02281. (The algorithm packages are the paper's copy's, below: a fixed algorithm/algorithmic
  // conflicts with algpseudocode.)
  RequirePackage!("booktabs");
  RequirePackage!("graphicx");
  // wrapfig: WileyNJD-v2 authors routinely use \begin{wrapfigure} for
  // figure-with-side-text layouts. The Wiley cls itself doesn't load
  // wrapfig; mirror by eager-loading. Witness 2203.16535.
  RequirePackage!("wrapfig");
  // WileyNJDv5.cls:357-376, 413, 620-621 (the WileyNJD-v2 and WileyASNA-v1 copies this binding also stands in for load
  // the same: 2203.16535, 2406.06228): the tabular, float and caption packages the class loads, in its order
  // (`\multirow` and `\captionsetup` were undefined, 2609.06025), lettersp (:377) where the copy loads it, below. Left
  // out: ulem (without `normalem` it underlines `\emph`), enumerate, soul, `babel[english]`, `footmisc[bottom]`, and the
  // page-layout packages.
  RequirePackage!("multicol");
  RequirePackage!("multirow");
  RequirePackage!("float");
  RequirePackage!("rotating", options => vec![s!("figuresright")]);
  RequirePackage!("caption");
  // WileyNJDv5.cls:377 `\usepackage{lettersp}`, after caption and long before the algorithms (:6820): LETTERSP.STY:3-12
  // defines its `\Comment … \EndComment` block only while `\Comment` is undefined, keeping an existing one as its own;
  // loaded after algpseudocode it took algorithmicx's `\Comment` for it and ran its self-documentation as text
  // (2408.07575, 2501.12471, 2602.03829, 2609.08425, 2609.19994). lettersp is no TeX Live package: only the copy the
  // paper ships, where the paper's copy of the class loads it.
  let (_, class_text) = wiley_version(class)?;
  let lettersp = || {
    find_file(
      "lettersp",
      Some(FindFileOptions {
        ext_type: Some(Cow::Borrowed("sty")),
        forbid_ltxml: true,
        ..Default::default()
      }),
    )
    .is_some()
  };
  if class_text
    .as_deref()
    .is_some_and(|text| class_text_loads(text, "lettersp"))
    && lettersp()
  {
    RequirePackage!("lettersp");
  }
  RequirePackage!("tabularx");
  RequirePackage!("varwidth");
  RequirePackage!("dcolumn");
  // WileyNJD-v2.cls:3331, WileyNJDv5.cls:6820 and WileyASNA-v1.cls:2555 `\usepackage{algorithm, algorithmicx,
  // algpseudocode}`, a copy `algorithm2e` (2002.02812:3338): `{algorithm}` and `\State` were undefined in papers that
  // rely on the class for them (1810.03033, 1810.12033, 2106.06046, 2106.14215, 2210.02694, 2402.11361, 2402.13183,
  // 2506.07013). The paper's copy decides; without one, none (Perl's OmniBus has no copy to scan either).
  require_shipped_algorithm_packages("cls")?;
  // WileyNJD-v2.cls:63 `\DeclareOption{APA}{\global\@APArefstyletrue}` and :2722-2723 `\if@APArefstyle
  // \usepackage[natbibapa]{NJDapacite}`: the APA reference style's bibliography is apacite's (`\APACrefYearMonthDay`,
  // `{APACrefauthors}`, `\BBOQ`; 22 papers of run 353 with them undefined: 1811.00964, 1902.09230, 1904.10792).
  // NJDapacite.sty, shipped with the class, is apacite.sty of 2013 under another name (its :46 `\ProvidesPackage
  // {apacite}`), so the apacite binding stands for it, as for sn-jnl's `sn-apa` (sn_jnl_cls.rs). The class's
  // re-styling of the reference parts (:2725-2760, fonts and punctuation) is not followed. The load follows the
  // version (`wiley_njd_class_body`).
  DeclareOption!("APA", {
    AssignValue!("wileynjd@APA" => true);
  });
  ProcessOptions!();

  // WileyNJD-v2.cls:1217-1221 (WileyNJDv5.cls:3636-3640, WileyASNA-v1 through WileyNJD-v1): the address parts of an
  // affiliation, each printing its argument; OmniBus has the others, but leaves `\state` out (omnibus_cls.rs: a
  // `\newcount\state` in some classes) — here it is only `\orgaddress{\state{NY}, \country{USA}}`, as in mrm_cls.rs.
  // Witnesses 1705.06379, 1910.04517, 2210.01555, 2605.24284.
  def_macro_identity("\\state{}")?;

  // Wiley frontmatter — preserve author content as ltx:note.
  DefMacro!("\\authormark{}", "\\textsuperscript{#1}");
  DefMacro!(
    "\\corres{}",
    "\\@add@frontmatter{ltx:note}[role=corresponding]{#1}"
  );
  DefMacro!(
    "\\jnlcitation OptionalMatch:* []{}{}",
    "\\@add@frontmatter{ltx:note}[role=citation]{#3 #4}"
  );
  DefMacro!(
    "\\presentadd[]{}",
    "\\@add@frontmatter{ltx:note}[role=present-address]{#2}"
  );
  DefMacro!(
    "\\fundingInfo{}",
    "\\@add@frontmatter{ltx:note}[role=funding]{#1}"
  );
  DefMacro!(
    "\\papertype{}",
    "\\@add@frontmatter{ltx:note}[role=papertype]{#1}"
  );
  DefMacro!(
    "\\paperfield{}",
    "\\@add@frontmatter{ltx:note}[role=paperfield]{#1}"
  );
  DefMacro!(
    "\\jname{}",
    "\\@add@frontmatter{ltx:note}[role=journal]{#1}"
  );
  DefMacro!("\\jvol{}", "\\@add@frontmatter{ltx:note}[role=volume]{#1}");
  DefMacro!("\\jnum{}", "\\@add@frontmatter{ltx:note}[role=issue]{#1}");
  DefMacro!(
    "\\cname{}{}",
    "\\@add@frontmatter{ltx:note}[role=copyright]{#1 #2}"
  );
  DefMacro!("\\cyear{}", "\\@add@frontmatter{ltx:note}[role=year]{#1}");
  // \cjournal{name} / \cvol{N} / \ctitle{text} — citation-line fields
  // also in WileyNJDv5 frontmatter (alongside \cname / \cyear). Witness
  // 2504.02281 — preserve as ltx:note.
  DefMacro!(
    "\\cjournal{}",
    "\\@add@frontmatter{ltx:note}[role=cjournal]{#1}"
  );
  DefMacro!("\\cvol{}", "\\@add@frontmatter{ltx:note}[role=cvolume]{#1}");
  DefMacro!(
    "\\ctitle{}",
    "\\@add@frontmatter{ltx:note}[role=ctitle]{#1}"
  );
  DefMacro!(
    "\\Copyrightline{}",
    "\\@add@frontmatter{ltx:note}[role=copyright]{#1}"
  );
  DefMacro!(
    "\\artmonth{}",
    "\\@add@frontmatter{ltx:note}[role=month]{#1}"
  );
  DefMacro!("\\DOI{}", "\\@add@frontmatter{ltx:note}[role=doi]{#1}");
  DefMacro!("\\doiline{}", "\\@add@frontmatter{ltx:note}[role=doi]{#1}");
  DefMacro!(
    "\\runningheads{}{}",
    "\\@add@frontmatter{ltx:note}[role=runningheads]{#1 / #2}"
  );
  DefMacro!(
    "\\receiveddate{}",
    "\\@add@frontmatter{ltx:note}[role=received]{#1}"
  );
  DefMacro!(
    "\\reviseddate{}",
    "\\@add@frontmatter{ltx:note}[role=revised]{#1}"
  );
  DefMacro!(
    "\\accepteddate{}",
    "\\@add@frontmatter{ltx:note}[role=accepted]{#1}"
  );
  // wileyNJDv5.cls (newer template) adds these. Witness 2407.00139.
  DefMacro!(
    "\\copyyear{}",
    "\\@add@frontmatter{ltx:note}[role=copyright-year]{#1}"
  );
  DefMacro!(
    "\\titlemark{}",
    "\\@add@frontmatter{ltx:note}[role=titlemark]{#1}"
  );
  DefMacro!(
    "\\startpage{}",
    "\\@add@frontmatter{ltx:note}[role=startpage]{#1}"
  );
  wiley_njd_class_body(class)
}

/// The version-sensitive definitions are chosen by the copy the paper ships, else by the class name.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WileyVersion {
  /// WileyNJD-v1.cls (2015-2017): its 20 theorems, `{boxtext}` an mdframed box.
  V1,
  /// WileyNJD-v2.cls and WileyASNA-v1.cls: the 47 theorems, `{boxtext}` without arguments.
  V2,
  /// WileyNJDv5.cls: v2's theorems, `{boxtext}[1]` and `{boxwithhead}[2]` (boites' `{breakbox}`).
  V5,
}

/// The version of the class the paper asked for, and the text of the copy it ships (`None` without one).
fn wiley_version(class: &str) -> Result<(WileyVersion, Option<String>)> {
  let name = class
    .rsplit(['/', '\\'])
    .next()
    .unwrap_or_default()
    .to_ascii_lowercase();
  let by_name = if name.contains("v5") {
    WileyVersion::V5
  } else if name.starts_with("wileynjd-v1") {
    WileyVersion::V1
  } else {
    WileyVersion::V2
  };
  // `shipped_class_text` reads the copy of the class the load resolved to (`\@currname`, or the request a fallback
  // answered with this binding: `WileyNJDv5_preprint`, `WileyNJD-v2a`)
  let binding = if name.starts_with("wileyasna") {
    "WileyASNA-v1"
  } else {
    class
  };
  let text = shipped_class_text(binding)?;
  let version = text
    .as_deref()
    .and_then(version_from_text)
    .unwrap_or(by_name);
  Ok((version, text))
}

/// The version a copy of the class is, by the definitions where the versions differ: WileyNJDv5.cls:592 `{boxtext}[1]`
/// (2302.02310), the v2 theorem set's `{eexample}` (WileyNJD-v2.cls:3012, 1712.08341; WileyASNA-v1.cls and the
/// WileyNJD-v1.cls of 2020, 2010.09463, have it too), the v1 set's `{example}` (WileyNJD-v1.cls:4564, 1511.08547).
/// `None` for a copy with none of them.
fn version_from_text(text: &str) -> Option<WileyVersion> {
  if class_code_contains(text, "\\newenvironment{boxtext}[1]") {
    Some(WileyVersion::V5)
  } else if class_code_contains(text, "\\newtheorem{eexample}") {
    Some(WileyVersion::V2)
  } else if class_code_contains(text, "\\newtheorem{example}{Example}") {
    Some(WileyVersion::V1)
  } else {
    None
  }
}

/// The commands, environments and theorems the Wiley NJD classes define that papers use and OmniBus does not have
/// (the class bodies are the same in WileyNJD-v2.cls of 2017-10-25 (1712.08341, 312 copies), WileyNJDv5.cls
/// (2302.02310, 89 copies), WileyASNA-v1.cls (1804.11014, 158 copies) and WileyNJD-v1.cls (1511.08547, 12 copies) where
/// not said otherwise; line numbers are these copies').
fn wiley_njd_class_body(class: &str) -> Result<()> {
  let (version, class_text) = wiley_version(class)?;
  let asna = class.to_ascii_lowercase().contains("wileyasna");
  // WileyNJD-v2.cls:1781 (WileyNJDv5.cls:5066, WileyASNA-v1.cls:1195, WileyNJD-v1.cls:956)
  // `\def\presentaddress#1{\gdef\@presentaddress{…\textbf{Present Address}\par #1}}`, printed under the
  // correspondence block of the title page (:2289): a frontmatter note, as sn-jnl's (sn_jnl_cls.rs). Undefined in 156
  // WileyNJD-v2, 68 WileyASNA-v1 and 38 WileyNJDv5 papers of run 353 (1701.02349, 1705.09846, 1712.08341, 1801.09541;
  // WileyNJD-v1 1709.02675).
  DefMacro!(
    "\\presentaddress{}",
    "\\@add@frontmatter{ltx:note}[role=present-address]{#1}"
  );
  // The title-page stores (WileyNJD-v2.cls:1812, 1890-1891, 1198; WileyNJDv5.cls:5099, 5182-5183; WileyASNA-v1.cls:1226,
  // 1319-1320, 624): the history line, the funding block's headings and the article id are page furniture
  // `\@maketitle` lays out (`\historydates` 1902.09230, 2605.19910; `\fundinginfohead` 1806.09518; `\artid` 2101.04467,
  // 2608.01016). The running head's DOI text (WileyNJD-v2.cls:1820, WileyNJDv5.cls:5107, WileyASNA-v1.cls:1234), a
  // paper's DOI where it is given (`\doiheadtext{DOI: 10.1002/fld.4504}`, 1803.06134; 2306.05470, 2505.23984,
  // 2512.05601), is a DOI note, as `\DOI`.
  // `\uns` (WileyNJD-v2.cls:1180 `\def\uns{\ifmmode\,\else$\,$\fi}`) is a thin space in or out of math (2208.08472,
  // 2211.16474), as the class defines it (until 78a `\thinspace` out of math: the formula holding only `\,` ended the
  // paragraph's `--` ligatures in the kernel, 2208.08472's "1989--2008"; fixed in document.rs, guard
  // `perfect_kernel_batch78a::text_after_a_space_only_formula_is_ligatured`). `\subtitlefont`
  // (WileyNJD-v2.cls:1436, 1506, WileyNJDv5.cls:3990, the size options' title fonts) the font of a subtitle in
  // `\title{…\\\subtitlefont …}` (1902.01961, 2108.10040), its margins left out.
  RawTeX!(
    r"\def\historydates#1{\gdef\@history@dates{#1}}%
\def\subtitlefont{\rmfamily\fontsize{16}{21}\bfseries\selectfont}%
\def\fundinginfohead#1{\gdef\@fundinginfo@head{#1}}%
\def\fundinginfoheadtext#1{\gdef\@fundinginfo@head@text{#1}}%
\def\artid#1{\gdef\@artid{#1}}%
\def\uns{\ifmmode\,\else$\,$\fi}%"
  );
  DefMacro!(
    "\\doiheadtext{}",
    "\\@add@frontmatter{ltx:note}[role=doi]{#1}"
  );
  // WileyNJD-v1.cls's reference styles (1511.08547's copy: APA :1104-1106, AMS :1850-1851, AMA :2498-2499, Vancouver
  // :3080-3081, Chemistry :3684-3685) each load amsrefs, whose `{bibdiv}`, `{biblist}`, `\bib` and `\ndash` the papers'
  // reference lists use (1511.08547, 1707.00398, 1709.02675, 1709.08526, 1806.01802: 9 papers of run 353), each load
  // inside its `\if@…refstyle`, which only the option sets (:33-43, no `\ExecuteOptions`): no amsrefs without one of
  // APA, AMS, ACS, AMA, VANCOUVER. Those copies load no apacite, so their APA style is amsrefs' author-year, not the
  // later versions' NJDapacite (WileyNJD-v2.cls:2722-2723, WileyNJDv5.cls, WileyASNA-v1.cls and the WileyNJD-v1.cls of
  // 2020, 2010.09463, 2208.12482, which load no amsrefs and are the v2 version here). The 2020 copies' title-page
  // stores (2010.09463's copy :1825-1842): the journal's name, logo and URL in the running head; `\openaccess` (:1871)
  // sets its licence text as an unmarked footnote of the title page, a licence pubnote here (as
  // dmtcs_episciences_cls.rs, acmart_cls.rs).
  let copy_loads = |package: &str, without_copy: bool| {
    class_text
      .as_deref()
      .map_or(without_copy, |text| class_text_loads(text, package))
  };
  let refstyle = ["APA", "AMS", "ACS", "AMA", "VANCOUVER"]
    .into_iter()
    .map(class_option_given)
    .collect::<Result<Vec<_>>>()?
    .contains(&true);
  if refstyle && copy_loads("amsrefs", version == WileyVersion::V1) {
    RequirePackage!("amsrefs");
  }
  if lookup_bool("wileynjd@APA")
    && (copy_loads("NJDapacite", version != WileyVersion::V1) || copy_loads("apacite", false))
  {
    RequirePackage!("apacite", options => vec![s!("natbibapa")]);
    // The copy's own `\usepackage[apaciteclassic]{NJDapacite}` (WileyNJDv5.cls:6067; v2's `[natbibapa]`) is a repeat
    // of this load, NJDapacite.sty being the apacite.sty the binding stands for: marked loaded on the class's behalf,
    // as amsart marks amsthm (ams_support_sty.rs). Read raw after the binding, its `apaciteclassic` skip of the
    // `\if@APAC@natbib@emu` branch (NJDapacite.sty:587-987) met the natbib binding's `\ifciteindex` at :931's
    // `\newif\ifciteindex` — a conditional to the skip, where TeX, with no natbib loaded, sees an undefined name — and
    // ran on to :1878-1882's `\let\ifBR@verbose\iffalse\let\fi\fi`: three "Attempt to close a group" Errors
    // (2606.10545, 2602.12288; pdflatex 0).
    AssignValue!("NJDapacite.sty_loaded" => true, Some(Scope::Global));
    RawTeX!(
      r"\expandafter\let\csname ver@NJDapacite.sty\expandafter\endcsname\csname ver@apacite.sty\endcsname"
    );
  }
  // Every version loads listings and alltt (WileyNJD-v2.cls:3346-3347, WileyNJDv5.cls:6822, WileyASNA-v1.cls:2557,
  // WileyNJD-v1.cls:4712; `\lstset` undefined: 1802.08557, 2206.13356). WileyASNA-v1.cls:250 and the older
  // WileyNJD-v1.cls:256 load `[labelformat=empty]{subfig}` (`\subfloat` undefined: 1511.08547, 1707.00398); WileyNJD-v2
  // (:287), WileyNJDv5 (:397) and the WileyNJD-v1 of 2020 (2010.09463:281) only `\IfFileExists{subfigure.sty}{}{…}`,
  // and TeX Live has subfigure.sty, so none.
  RequirePackage!("listings");
  RequirePackage!("alltt");
  let subfig = match class_text.as_deref() {
    Some(text) => {
      class_text_loads(text, "subfig")
        && !class_code_contains(text, "\\IfFileExists{subfigure.sty}")
    },
    None => version == WileyVersion::V1 || asna,
  };
  if subfig {
    RequirePackage!("subfig", options => vec![s!("labelformat=empty")]);
  }
  RawTeX!(
    r"\def\journalname#1{\gdef\@journalname{#1}}%
\def\journallogo#1#2{\gdef\@journallogo{\textsc{#1}\ \textcolor{blue}{#2}}}%
\def\jurl#1{\gdef\@jurl{#1}}"
  );
  DefMacro!("\\openaccess{}", "\\lx@add@pubnote[role=license]{#1}");
  // WileyNJD-v2.cls:1857-1864 (WileyNJDv5.cls:5149, WileyASNA-v1.cls:1271, WileyNJD-v1.cls:978, every copy)
  // `\renewcommand\abstract[2][]{…\absheadfont#1…#2…}`: the abstract and, optionally, its heading
  // (`\abstract[Summary]{…}`, the templates' form: 45 of 60 sampled papers of the family, 1701.02349, 1712.08341; the
  // kernel's `\abstract` read `[Summary]` as text of the abstract in 219 of 490 papers). The heading names the
  // abstract; without one it keeps the kernel's name.
  RawTeX!(
    r"\def\lx@wiley@abstract#1#2{\if\relax\detokenize{#1}\relax\lx@add@abstract{#2}\else%
\lx@add@abstract[name={#1}]{#2}\fi}"
  );
  DefMacro!("\\abstract[]{}", "\\lx@wiley@abstract{#1}{#2}");
  // WileyNJD-v2.cls:1877-1878 (WileyNJDv5.cls:5169-5170): the parts of a `\fundingInfo` entry, printed as
  // "Agency, Number."; WileyASNA-v1 sets them in running text (wileyasna_cls.rs).
  RawTeX!(r"\def\fundingNumber#1{#1.}\def\fundingAgency#1{#1,\space}");
  // WileyNJDv5.cls:5762-5764: the class's own sectioning commands, `\@startsection`s of the section levels, so with
  // their star and optional title (`\bmsection*{Acknowledgments}`, 2405.03603, 2511.08532: the former `\bmsection{}`
  // read the star as the title) and the section counter after `\appendix` (2511.08532's `\bmsection{Details …}`):
  // `\section` as it is when called. `\bmsubsubsection`: 2309.00837, 2311.07331, 2602.03894.
  RawTeX!(
    r"\def\bmsection{\section}\def\bmsubsection{\subsection}\def\bmsubsubsection{\subsubsection}"
  );

  // WileyNJD-v2.cls:2816-2828 (WileyNJDv5.cls:6188-6200, WileyASNA-v1.cls:2315-2327, WileyNJD-v1.cls:4321-4333): the
  // notes under a table, a list whose first item is the text before any `\item` (`\begin{tablenotes} a) Eq. 1, …`,
  // 1808.09739; `\item[$\dagger$] …`, 1808.05540, 1901.00057, 1903.09736), and `\tnote`, which prints its mark as it
  // is (1910.13464, 1911.09684). threeparttable, loaded after the class, replaces both. The class's leading
  // `\item\relax` opens an item only where text precedes the first `\item` (OXIDIZED_DESIGN_DIVERGENCES #601: the
  // empty first item the class sets before `\item[$\dagger$]` is a blank, no glyph, in the PDF, and would be an empty
  // `<ltx:item>` here).
  RawTeX!(
    r"\newenvironment{tablenotes}{\list{}{\setlength{\labelsep}{0pt}%
\setlength{\labelwidth}{0pt}%
\setlength{\leftmargin}{0pt}%
\setlength{\rightmargin}{0pt}%
\setlength{\topsep}{2pt}%
\setlength{\itemsep}{0pt}%
\setlength{\partopsep}{0pt}%
\setlength{\listparindent}{0em}%
\setlength{\parsep}{0pt}}%
\@ifnextchar\item{}{\item\relax}}{\endlist}%
\def\tnote#1{#1}"
  );

  // WileyNJD-v2.cls:3445-3465 (WileyNJDv5.cls:6924-6943 in a `{breakbox}`, WileyASNA-v1.cls:2656, WileyNJD-v1.cls:4811):
  // `\begin{biography}{photo}{text}`, the author's photo wrapped by the text, `\author` bold; shaped as IEEEtran's
  // biography (ieeetran_cls.rs, Perl IEEEtran.cls.ltxml:231-240), without the photo cell when the photo is empty
  // (`\begin{biography}{}{…}`, 1812.10266, 2102.03158). Witnesses 1705.00674, 1805.12509, 1810.07251, 1809.09035.
  DefEnvironment!(
    "{biography}{}{}",
    "<ltx:float class='biography'>\
      <ltx:tabular>\
        <ltx:tr>\
          ?#1(<ltx:td>#1</ltx:td>)()\
          <ltx:td><ltx:inline-block>#2 #body</ltx:inline-block></ltx:td>\
        </ltx:tr>\
      </ltx:tabular>\
    </ltx:float>",
    before_digest => {
      DefMacro!("\\author{}", "\\textbf{#1}");
    }
  );

  // WileyNJD-v2.cls:3393-3395 (WileyNJDv5.cls:6869, WileyASNA-v1.cls:2604, WileyNJD-v1.cls:4759)
  // `\newenvironment{ack}[1][]{\begingroup\if!#1!\else\section*{#1}\fi}{\endgroup}`: the acknowledgements, headed by
  // the optional title (`\begin{ack}[Acknowledgements]`, 2205.06045; `[Conflict of interest]`, 2412.02681; none,
  // 1807.03475).
  DefEnvironment!(
    "{ack}[]",
    "<ltx:acknowledgements ?#1(name='#1')()>#body</ltx:acknowledgements>",
    mode => "internal_vertical"
  );

  // The boxed text. WileyNJD-v2.cls:466-475 frames the body (`\BX`, an `\fbox`); WileyASNA-v1.cls:267-271 and
  // WileyNJD-v1.cls:281-285 put it in an `{mdframed}`; WileyNJDv5.cls:592-608 frames the argument (boites'
  // `{breakbox}`, :6244), `{boxwithhead}` a bold head above it, and the body follows the box. A framed block, as
  // framed.sty's `{framed}`, for each. Witnesses 1811.09505, 2012.13740, 2101.02028, 2107.01251 (`{boxtext}`),
  // 2309.00837, 2405.03603, 2604.17508 (`{boxwithhead}`).
  // A `\section*` opening the box (its title: 1811.09505, 2012.13740) is the box's head, a bold line in the frame,
  // which a section, ending only at the next one, cannot be (OXIDIZED_DESIGN_DIVERGENCES #603). Only `\section` is
  // rebound: the 390 witness papers put no other level in a box.
  // framed.sty is the implementation vehicle of the frame (its `{framed}` the binding's framed logical block), not a
  // load of any version: the classes frame with `\BX`/`\fbox`, mdframed (WileyNJD-v1.cls:281, WileyASNA-v1.cls:267)
  // and boites (WileyNJDv5.cls).
  RequirePackage!("framed");
  RawTeX!(
    r"\def\lx@wiley@boxhead{\@ifstar\lx@wiley@boxhead@\lx@wiley@boxhead@}%
\def\lx@wiley@boxhead@#1{\par\noindent{\bfseries #1}\par}"
  );
  if version == WileyVersion::V5 {
    RawTeX!(
      r"\newenvironment{boxtext}[1]{\begin{framed}\noindent{\normalsize #1\par}\end{framed}}{}%
\newenvironment{boxwithhead}[2]{\begin{framed}{\noindent\bfseries #1}\par\noindent{\normalsize #2\par}\end{framed}}{}"
    );
  } else {
    RawTeX!(r"\newenvironment{boxtext}{\begin{framed}\let\section\lx@wiley@boxhead}{\end{framed}}");
  }

  // The theorem set (WileyNJD-v2.cls:2923-3030, WileyNJDv5.cls and WileyASNA-v1.cls alike; WileyNJD-v1.cls:4502-4568
  // its first 20 in other styles): three amsthm styles and 47 theorems, each with its own counter. Undefined before
  // in the papers that use them without declaring them (`{assumption}`, 44 WileyNJD-v2 and 23 WileyNJDv5 papers of
  // run 353: 1803.11411, 1808.10367, 1901.09809; `{eexample}` 1806.09518, `{property}` 1904.09923, `{result}` 2106.06046,
  // `{LWrule}` 1911.03315). The class first redefines amsthm's `plain` (WileyNJD-v2.cls:2902-2910, every copy: an
  // italic head without punctuation, set off by `\newline`), then defines its three styles (:2923-2963), and leaves
  // the last, WBstylethree, current (v2:3005, WileyNJDv5.cls:6377, WileyASNA-v1.cls:2504, WileyNJD-v1.cls:4567): a
  // paper's own `\newtheorem{thm}` is in it, as a `\theoremstyle{plain}` one is in the class's `plain`. The versions'
  // body fonts differ (`theorem_styles`): pdflatex with 1712.08341's WileyNJD-v2.cls sets the paper's theorem, a
  // `plain` one and `{lemma}` upright (t1-stixgeneral), with 2302.02310's WileyNJDv5.cls the paper's upright, the
  // `plain` one and `{lemma}` italic (ptmr8t, ptmri8t, ptmri8t).
  RawTeX!(r"\newif\ifOldenunciation\newif\ifEnunciation");
  let styles = class_text
    .as_deref()
    .and_then(copy_theorem_styles)
    .unwrap_or_else(|| theorem_styles(version, asna).to_owned());
  RawTeX!(&styles);
  if version == WileyVersion::V1 {
    RawTeX!(
      r"\theoremstyle{WBstyleone}%
\newtheorem{theorem}{Theorem}\newtheorem{property}{Property}\newtheorem{problem}{Problem}\newtheorem{fact}{Fact}%
\newtheorem{construction}{Construction}\newtheorem{estimate}{Estimate}\newtheorem{case}{Case}\newtheorem{lemma}{Lemma}%
\newtheorem{corollary}{Corollary}\newtheorem{result}{Result}\newtheorem{algth}{Algorithm}%
\newtheorem{proposition}{Proposition}\newtheorem{condition}{Condition}\newtheorem{hypothesis}{Hypothesis}%
\newtheorem{model}{Model}\newtheorem{experiment}{Experiment}%
\theoremstyle{WBstyletwo}%
\newtheorem{remark}{Remark}\newtheorem{example}{Example}\newtheorem{assumption}{Assumption}%
\theoremstyle{WBstylethree}%
\newtheorem{definition}{Definition}"
    );
  } else {
    RawTeX!(
      r"\theoremstyle{WBstyleone}%
\newtheorem{theorem}{Theorem}\newtheorem{construction}{Construction}\newtheorem{estimate}{Estimate}%
\newtheorem{lemma}{Lemma}\newtheorem{corollary}{Corollary}\newtheorem{result}{Result}\newtheorem{algth}{Algorithm}%
\newtheorem{proposition}{Proposition}\newtheorem{hypothesis}{Hypothesis}\newtheorem{experiment}{Experiment}%
\newtheorem{LWrule}{Rule}\newtheorem{procedure}{Procedure}%
\newtheorem{assertion}{Assertion}\newtheorem{axiom}{Axiom}\newtheorem{conjecture}{Conjecture}%
\newtheorem{criterion}{Criterion}\newtheorem{reduction}{Reduction}\newtheorem{sublemma}{Sublemma}%
\theoremstyle{WBstyletwo}%
\newtheorem{case}{Case}\newtheorem{remark}{Remark}\newtheorem{notation}{Notation}%
\newtheorem{answer}{Answer}\newtheorem{base}{Base}\newtheorem{claim}{Claim}\newtheorem{ccomment}{Comment}%
\newtheorem{subproof}{Subproof}\newtheorem{conclusion}{Conclusion}\newtheorem{observation}{Observation}%
\newtheorem{subcase}{Subcase}\newtheorem{step}{Step}\newtheorem{summary}{Summary}%
\theoremstyle{WBstylethree}%
\newtheorem{definition}{Definition}\newtheorem{condition}{Condition}\newtheorem{property}{Property}%
\newtheorem{problem}{Problem}\newtheorem{fact}{Fact}\newtheorem{assumption}{Assumption}%
\newtheorem{eexample}{Example}\newtheorem{model}{Model}%
\newtheorem{affirmation}{Affirmation}\newtheorem{application}{Application}\newtheorem{convention}{Convention}%
\newtheorem{discussion}{Discussion}\newtheorem{exercise}{Exercise}\newtheorem{question}{Question}%
\newtheorem{scholium}{Scholium}\newtheorem{terminology}{Terminology}"
    );
  }
  Ok(())
}

/// The class's four theorem styles.
const THEOREM_STYLES: [&str; 4] = ["plain", "WBstyleone", "WBstyletwo", "WBstylethree"];

/// The settings of WileyNJD-v1.cls's (2015-2018) theorem bodies that set their math upright (1511.08547's copy
/// :4505, :4527, :4537 `{\itshape\everymath{\rm}\let\text\mathrm}`), left out: the formulas of a theorem would be
/// `\rm` formulas, whose letters the math parser reads as one word (`A_{ij}`'s subscript the identifier "ij", 1511.08547)
/// and whose `tex` starts with `\rm`; the body stays italic.
const UPRIGHT_MATH: [&str; 2] = ["\\everymath{\\rm}", "\\let\\text\\mathrm"];

/// The theorem styles as the paper's copy of the class defines them: each `\newtheoremstyle{<name>}` of
/// [`THEOREM_STYLES`] with its eight arguments, the last definition outside a comment; `None` where the copy lacks
/// one. (The 578 copies of the family's classes shipped by the 817 papers of the 67w inventory define them four ways,
/// [`theorem_styles`].)
fn copy_theorem_styles(text: &str) -> Option<String> {
  let code = class_code(text);
  let mut styles = String::new();
  for name in THEOREM_STYLES {
    let head = format!("\\newtheoremstyle{{{name}}}");
    let at = code.rfind(&head)?;
    let mut rest = &code[at + head.len()..];
    styles.push_str(&head);
    for _ in 0..8 {
      rest = rest.trim_start();
      let group = balanced_group(rest)?;
      styles.push_str(group);
      rest = &rest[group.len()..];
    }
    styles.push_str("%\n");
  }
  Some(
    UPRIGHT_MATH
      .iter()
      .fold(styles, |styles, setting| styles.replace(setting, "")),
  )
}

/// The brace group `text` starts with, its braces included.
fn balanced_group(text: &str) -> Option<&str> {
  if !text.starts_with('{') {
    return None;
  }
  let mut depth = 0usize;
  for (at, ch) in text.char_indices() {
    match ch {
      '{' => depth += 1,
      '}' => {
        depth -= 1;
        if depth == 0 {
          return Some(&text[..=at]);
        }
      },
      _ => {},
    }
  }
  None
}

/// The theorem styles of a version, for a paper that ships no copy of the class: the bodies of WileyNJD-v2.cls:2902-2963
/// (1712.08341, 294 of the 312 v2 copies; the WileyNJD-v1.cls of 2020, 2010.09463, alike), WileyASNA-v1.cls:2401-2463
/// (1804.11014, and 18 v2 copies: the bodies indented, WBstyleone's italic), WileyNJDv5.cls:6274-6335 (2302.02310: `plain`
/// and WBstyleone italic) and WileyNJD-v1.cls:4481-4543 (1511.08547: the WB styles italic with bold heads, `plain`
/// upright with an italic head; their upright math left out, [`UPRIGHT_MATH`]).
fn theorem_styles(version: WileyVersion, asna: bool) -> &'static str {
  match version {
    WileyVersion::V2 if asna => {
      r"\newtheoremstyle{plain}{8pt}{8pt}{\Enunciationtrue\ifOldenunciation\else\leftskip8pt\rightskip8pt\fi\normalfont}{}{\itshape}{}{\newline}{}%
\newtheoremstyle{WBstyleone}{6\p@}{6\p@}{\Enunciationtrue\leftskip8pt\rightskip8pt\normalfont\itshape}{\z@}{\bfseries}{.}{5\p@}{}%
\newtheoremstyle{WBstyletwo}{6\p@}{6\p@}{\Enunciationtrue\leftskip8pt\rightskip8pt\normalfont}{\z@}{\itshape}{.}{5\p@}{\thmname{#1}\thmnumber{ #2}\thmnote{. #3}}%
\newtheoremstyle{WBstylethree}{6\p@}{6\p@}{\Enunciationtrue\normalfont\leftskip8pt\rightskip8pt}{\z@}{\bfseries}{.}{5\p@}{}"
    },
    WileyVersion::V2 => {
      r"\newtheoremstyle{plain}{8pt}{8pt}{\normalfont}{}{\itshape}{}{\newline}{}%
\newtheoremstyle{WBstyleone}{6\p@}{6\p@}{\normalfont}{\z@}{\bfseries}{.}{5\p@}{}%
\newtheoremstyle{WBstyletwo}{6\p@}{6\p@}{\normalfont}{\z@}{\itshape}{.}{5\p@}{\thmname{#1}\thmnumber{ #2}\thmnote{. #3}}%
\newtheoremstyle{WBstylethree}{6\p@}{6\p@}{\normalfont}{\z@}{\bfseries}{.}{5\p@}{}"
    },
    WileyVersion::V5 => {
      r"\newtheoremstyle{plain}{8pt}{8pt}{\normalsize\it}{}{\itshape}{}{\newline}{}%
\newtheoremstyle{WBstyleone}{6\p@}{6\p@}{\normalfont\it}{\z@}{\bfseries}{.}{5\p@}{}%
\newtheoremstyle{WBstyletwo}{6\p@}{6\p@}{\normalfont}{\z@}{\itshape}{.}{5\p@}{\thmname{#1}\thmnumber{ #2}\thmnote{. #3}}%
\newtheoremstyle{WBstylethree}{6\p@}{6\p@}{\normalfont}{\z@}{\bfseries}{.}{5\p@}{}"
    },
    WileyVersion::V1 => {
      r"\newtheoremstyle{plain}{8pt}{8pt}{\normalfont}{}{\itshape}{}{\newline}{}%
\newtheoremstyle{WBstyleone}{6\p@}{6\p@}{\itshape}{\z@}{\bfseries}{.}{5\p@}{}%
\newtheoremstyle{WBstyletwo}{6\p@}{6\p@}{\normalfont\itshape}{\z@}{\bfseries}{.}{5\p@}{\thmname{#1}\thmnumber{ #2}\thmnote{. #3}}%
\newtheoremstyle{WBstylethree}{6\p@}{6\p@}{\normalfont\itshape}{\z@}{\bfseries}{.}{5\p@}{}"
    },
  }
}

#[cfg(test)]
mod tests {
  use super::{WileyVersion, copy_theorem_styles, version_from_text};

  /// The definitions `version_from_text` tells the copies apart by, as the copies have them, commented-out
  /// alternatives included (1511.08547's WileyNJD-v1.cls:4471 `%\newtheorem{eexample}{Example}`, 2302.02310's
  /// WileyNJDv5.cls:578 `%%\newenvironment{boxtext}{…`).
  #[test]
  fn wiley_version_from_text_batch67w() {
    let v5 = "%%\\newenvironment{boxtext}{\\begingroup\\parskip=0pt\\parindent=0pt%\n\
              \\newenvironment{boxtext}[1]{%\n\\begin{breakbox}#1\\end{breakbox}}{}\n\
              %\\newtheorem{eexample}{Example}\n\\theoremstyle{WBstylethree}\n\\newtheorem{eexample}{Example}\n";
    let v2 = "%\\newenvironment{boxtext}{\\begingroup\\begin{fcolorbox}{black}{white}%\n\
              \\newenvironment{boxtext}{\\begingroup\\parskip=0pt\\parindent=0pt%\n\
              %\\newtheorem{eexample}{Example}\n\\newtheorem{definition}{Definition}\n\\newtheorem{eexample}{Example}\n";
    let v1 = "\\newenvironment{boxtext}{\\begingroup%\n%\\newtheorem{eexample}{Example}\n\
              \\theoremstyle{WBstyletwo}\n\\newtheorem{remark}{Remark}\n\\newtheorem{example}{Example}\n";
    assert_eq!(version_from_text(v5), Some(WileyVersion::V5));
    assert_eq!(version_from_text(v2), Some(WileyVersion::V2));
    assert_eq!(version_from_text(v1), Some(WileyVersion::V1));
    assert_eq!(
      version_from_text("\\ProvidesClass{WileyNJD-v2}\n%\\newtheorem{eexample}{Example}\n"),
      None
    );
  }

  /// The copy's own theorem styles, read as the class text writes them (2302.02310's WileyNJDv5.cls:6274-6335, a
  /// comment after every argument, an earlier commented-out definition).
  #[test]
  fn wiley_copy_theorem_styles_batch67w() {
    let copy = "%\\newtheoremstyle{plain}{1pt}{1pt}{\\bfseries}{}{}{}{}{}\n\
                \\newtheoremstyle{plain}% Numbered\n{8pt}% Space above\n{8pt}% Space below\n{\\normalsize\\it}% Body font\n\
                {}% Indent amount\n{\\itshape}% Theorem head font\n{}% Punctuation\n{\\newline}% Space after\n{}% spec\n\n\
                \\newtheoremstyle{WBstyleone}\n  {6\\p@} % Space above\n  {6\\p@}\n  {\\normalfont\\it} % Body font\n  {\\z@}\n\
                  {\\bfseries}\n  {.}\n  {5\\p@}\n  {}\n\
                \\newtheoremstyle{WBstyletwo}{6\\p@}{6\\p@}{\\normalfont}{\\z@}{\\itshape}{.}{5\\p@}\n\
                  {\\thmname{#1}\\thmnumber{ #2}\\thmnote{. #3}} % Theorem head spec\n\
                \\newtheoremstyle{WBstylethree}{6\\p@}{6\\p@}{\\normalfont}{\\z@}{\\bfseries}{.}{5\\p@}{}\n";
    assert_eq!(
      copy_theorem_styles(copy).as_deref(),
      Some(
        "\\newtheoremstyle{plain}{8pt}{8pt}{\\normalsize\\it}{}{\\itshape}{}{\\newline}{}%\n\
         \\newtheoremstyle{WBstyleone}{6\\p@}{6\\p@}{\\normalfont\\it}{\\z@}{\\bfseries}{.}{5\\p@}{}%\n\
         \\newtheoremstyle{WBstyletwo}{6\\p@}{6\\p@}{\\normalfont}{\\z@}{\\itshape}{.}{5\\p@}{\\thmname{#1}\\thmnumber{ #2}\\thmnote{. #3}}%\n\
         \\newtheoremstyle{WBstylethree}{6\\p@}{6\\p@}{\\normalfont}{\\z@}{\\bfseries}{.}{5\\p@}{}%\n"
      )
    );
    // WileyNJD-v1.cls's (1511.08547) italic bodies, their upright math left out
    let v1 = copy.replace(
      "{\\normalfont\\it}",
      "{\\itshape\\everymath{\\rm}\\let\\text\\mathrm}",
    );
    assert!(
      copy_theorem_styles(&v1)
        .unwrap()
        .contains("\\newtheoremstyle{WBstyleone}{6\\p@}{6\\p@}{\\itshape}{\\z@}"),
      "{v1}"
    );
    // a copy without WBstylethree: the version's styles instead
    assert_eq!(
      copy_theorem_styles(&copy.replace("WBstylethree", "WBother")),
      None
    );
  }
}
