//! Stub for LIPIcs class (Dagstuhl Leibniz International Proceedings).
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  RequirePackage!("amsthm");
  RequirePackage!("amssymb");
  RequirePackage!("xcolor");
  RequirePackage!("hyperref");

  // LIPIcs frontmatter — gobble cleanly.
  DefMacro!("\\Copyright{}", "");
  DefMacro!("\\CopyrightDetails", "");
  DefMacro!("\\authorrunning{}", "");
  DefMacro!("\\titlerunning{}", "");
  DefMacro!("\\funding{}", "");
  DefMacro!("\\fundingAgency{}", "");
  DefMacro!("\\authorcredit{}", "");
  DefMacro!("\\nolinenumbers", "");
  DefMacro!("\\category{}", "");
  DefMacro!("\\related{}", "");
  DefMacro!("\\relatedversion{}", "");
  DefMacro!("\\supplement{}", "");
  DefMacro!("\\supplementdetails[]{}{}", "");
  DefMacro!("\\acknowledgements{}", "");
  DefMacro!("\\ccsdesc[]{}", "");
  DefMacro!("\\subjclass[]{}", "");
  DefMacro!("\\keywords{}", "");
  DefMacro!("\\event{}", "");
  DefMacro!("\\EventEditors{}", "");
  DefMacro!("\\EventLongTitle{}", "");
  DefMacro!("\\EventShortTitle{}", "");
  DefMacro!("\\EventAcronym{}", "");
  DefMacro!("\\EventYear{}", "");
  DefMacro!("\\EventDate{}", "");
  DefMacro!("\\EventLocation{}", "");
  DefMacro!("\\EventLogo{}", "");
  DefMacro!("\\SeriesVolume{}", "");
  DefMacro!("\\ArticleNo{}", "");
  // LIPIcs L739: \EventNoEds{N} sets editor count.
  DefMacro!("\\EventNoEds{}", "");

  // LIPIcs L1158-1234: theorem-like environments.
  RawTeX!(
    r"\newtheorem{theorem}{Theorem}
\newtheorem{lemma}[theorem]{Lemma}
\newtheorem{corollary}[theorem]{Corollary}
\newtheorem{proposition}[theorem]{Proposition}
\newtheorem{definition}[theorem]{Definition}
\newtheorem{observation}[theorem]{Observation}
\newtheorem{remark}[theorem]{Remark}
\newtheorem{example}[theorem]{Example}
\newtheorem{claim}[theorem]{Claim}
\newtheorem{conjecture}[theorem]{Conjecture}"
  );
});
