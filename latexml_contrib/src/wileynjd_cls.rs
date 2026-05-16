//! Stub for Wiley NJD family of classes (WileyNJD-v1, WileyASNA-v1, ...).
//!
//! These Wiley journal classes share a common set of frontmatter macros
//! (\corres, \authormark, \jnlcitation, \cname, \cyear, \vol, \DOI,
//! \papertype, ...). Route to OmniBus and gobble the frontmatter so
//! downstream content renders cleanly.
use latexml_package::prelude::*;

LoadDefinitions!({
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

  // Wiley frontmatter — preserve author content as ltx:note.
  DefMacro!("\\authormark{}", "\\textsuperscript{#1}");
  DefMacro!("\\corres{}",
    "\\@add@frontmatter{ltx:note}[role=corresponding]{#1}");
  DefMacro!("\\jnlcitation OptionalMatch:* []{}{}",
    "\\@add@frontmatter{ltx:note}[role=citation]{#3 #4}");
  DefMacro!("\\presentadd[]{}",
    "\\@add@frontmatter{ltx:note}[role=present-address]{#2}");
  DefMacro!("\\fundingInfo{}",
    "\\@add@frontmatter{ltx:note}[role=funding]{#1}");
  DefMacro!("\\papertype{}",
    "\\@add@frontmatter{ltx:note}[role=papertype]{#1}");
  DefMacro!("\\paperfield{}",
    "\\@add@frontmatter{ltx:note}[role=paperfield]{#1}");
  DefMacro!("\\jname{}",
    "\\@add@frontmatter{ltx:note}[role=journal]{#1}");
  DefMacro!("\\jvol{}",
    "\\@add@frontmatter{ltx:note}[role=volume]{#1}");
  DefMacro!("\\jnum{}",
    "\\@add@frontmatter{ltx:note}[role=issue]{#1}");
  DefMacro!("\\cname{}{}",
    "\\@add@frontmatter{ltx:note}[role=copyright]{#1 #2}");
  DefMacro!("\\cyear{}",
    "\\@add@frontmatter{ltx:note}[role=year]{#1}");
  DefMacro!("\\Copyrightline{}",
    "\\@add@frontmatter{ltx:note}[role=copyright]{#1}");
  DefMacro!("\\artmonth{}",
    "\\@add@frontmatter{ltx:note}[role=month]{#1}");
  DefMacro!("\\DOI{}",
    "\\@add@frontmatter{ltx:note}[role=doi]{#1}");
  DefMacro!("\\doiline{}",
    "\\@add@frontmatter{ltx:note}[role=doi]{#1}");
  DefMacro!("\\runningheads{}{}",
    "\\@add@frontmatter{ltx:note}[role=runningheads]{#1 / #2}");
  DefMacro!("\\receiveddate{}",
    "\\@add@frontmatter{ltx:note}[role=received]{#1}");
  DefMacro!("\\reviseddate{}",
    "\\@add@frontmatter{ltx:note}[role=revised]{#1}");
  DefMacro!("\\accepteddate{}",
    "\\@add@frontmatter{ltx:note}[role=accepted]{#1}");
  // wileyNJDv5.cls (newer template) adds these. Witness 2407.00139.
  DefMacro!("\\copyyear{}",
    "\\@add@frontmatter{ltx:note}[role=copyright-year]{#1}");
  DefMacro!("\\titlemark{}",
    "\\@add@frontmatter{ltx:note}[role=titlemark]{#1}");
  DefMacro!("\\startpage{}",
    "\\@add@frontmatter{ltx:note}[role=startpage]{#1}");
  DefMacro!("\\bmsection{}", "\\section{#1}");
  DefMacro!("\\bmsubsection{}", "\\subsection{#1}");
  // 'corres' (without trailing 's' from real wileynjd) — alternative
  // \corres macro signature in v5 templates.
});
