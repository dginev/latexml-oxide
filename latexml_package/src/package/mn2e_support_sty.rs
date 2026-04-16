//! mn2e_support.sty — MNRAS (Monthly Notices of the Royal Astronomical Society) support
//! Perl: mn2e_support.sty.ltxml — 252 lines
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Dependencies
  RequirePackage!("natbib");
  // mn2e.cls internal: base line skip (used in raw TeX class)
  DefRegister!("\\@bls" => Dimension!("12pt"));

  // Perl: mn2e_support.sty.ltxml L19-20 — load graphicx if option was set
  if state::lookup_int("@usegraphicx") != 0 {
    RequirePackage!("graphicx");
  }
  // mn2e.cls raw TeX: \if@useAMS\RequirePackage{amsmath,amssymb}\fi
  // Since we don't load the raw class, check the flag and load AMS packages
  if state::lookup_int("@useAMS") != 0 {
    RequirePackage!("amsmath");
    RequirePackage!("amssymb");
  }

  // Frontmatter — Perl L28-46
  DefMacro!("\\title[]{}", "\\@add@frontmatter{ltx:title}{#2}");
  DefMacro!("\\newauthor", "");
  DefMacro!("\\journal{}", "\\@add@frontmatter{ltx:note}[role=journal]{#1}");
  DefMacro!("\\volume{}", "\\@add@frontmatter{ltx:note}[role=volume]{#1}");
  DefMacro!("\\pubyear{}", "\\@add@frontmatter{ltx:note}[role=pubyear]{#1}");
  DefMacro!("\\microfiche{}", "\\@add@frontmatter{ltx:note}[role=microfiche]{#1}");
  DefMacro!("\\pagerange{}", "\\@add@frontmatter{ltx:note}[role=pagerange]{#1}");

  // Editorial queries — Perl L42-46
  DefConstructor!("\\BSLquery{}", "<ltx:note role='query'>#1</ltx:note>");
  DefConstructor!("\\aquery{}", "<ltx:note role='query'>#1</ltx:note>");
  DefConstructor!("\\tquery{}", "<ltx:note role='query'>#1</ltx:note>");
  DefEnvironment!("{query}", "<ltx:note role='query'>#body</ltx:note>");
  DefConstructor!("\\authorquery{}{}", "<ltx:note role='query'>#1: #2</ltx:note>");

  // Keywords — Perl L48-55
  DefMacro!("\\keywords{}", "\\@add@frontmatter{ltx:keywords}{#1}");

  // Dates — Perl L61-66
  DefMacro!("\\date[]{}", "\\@add@frontmatter{ltx:date}{#2}");
  DefMacro!("\\received{}", "\\@add@frontmatter{ltx:date}[role=received]{#1}");
  DefMacro!("\\accepted{}", "\\@add@frontmatter{ltx:date}[role=accepted]{#1}");

  // Affiliations — Perl L70-85
  DefMacro!("\\@affil[]{}", "\\@add@to@frontmatter{ltx:creator}{\\@@@affiliation{#2}}");
  DefConstructor!("\\@@@affiliation{}", "^ <ltx:contact role='affiliation'>#1</ltx:contact>");

  // Email
  DefMacro!("\\email Semiverbatim", "\\@add@to@frontmatter{ltx:creator}{\\@@@email{#1}}");
  DefConstructor!("\\@@@email{}", "^ <ltx:contact role='email'>#1</ltx:contact>");

  // Acknowledgements — Perl L95
  DefConstructor!("\\bsp", "");
  Let!("\\ackn", "\\acknowledgments");
  DefMacro!("\\acknowledgments", "\\section*{Acknowledgments}");

  // Math shortcuts — Perl L120-175
  DefMacro!("\\la", "\\lesssim");
  DefMacro!("\\ga", "\\gtrsim");
  DefMacro!("\\getsto", "\\rightleftharpoons");
  DefMacro!("\\sun", "\u{2609}");
  DefMacro!("\\degr", "\u{00B0}");
  DefMacro!("\\arcmin", "\u{2032}");
  DefMacro!("\\arcsec", "\u{2033}");
  DefMacro!("\\fd", ".\\!^{\\mathrm{d}}");
  DefMacro!("\\fh", ".\\!^{\\mathrm{h}}");
  DefMacro!("\\fm", ".\\!^{\\mathrm{m}}");
  DefMacro!("\\fs", ".\\!^{\\mathrm{s}}");
  DefMacro!("\\fp", ".\\!^{\\mathrm{p}}");
  // Perl: mn2e_support.sty.ltxml — degree/arcmin/arcsec using \aas@fstack
  DefMacro!("\\fdg", "\\aas@fstack{\\circ}");
  DefMacro!("\\farcm", "\\aas@fstack{\\prime}");
  DefMacro!("\\farcs", "\\aas@fstack{\\prime\\prime}");
  DefMacro!("\\ion{}{}", "#1\\,{\\sc #2}");

  // Journal abbreviations — Perl L180-252
  DefMacro!("\\mnras", "MNRAS");
  DefMacro!("\\nat", "Nature");
  DefMacro!("\\apj", "ApJ");
  DefMacro!("\\apjl", "ApJ");
  DefMacro!("\\apjs", "ApJS");
  DefMacro!("\\aj", "AJ");
  DefMacro!("\\aap", "A\\&A");
  DefMacro!("\\aapr", "A\\&A~Rev.");
  DefMacro!("\\aaps", "A\\&AS");
  DefMacro!("\\araa", "ARA\\&A");
  DefMacro!("\\pasp", "PASP");
  DefMacro!("\\pasa", "PASA");
  DefMacro!("\\pasj", "PASJ");
  DefMacro!("\\prd", "Phys. Rev. D");
  DefMacro!("\\prl", "Phys. Rev. Lett.");
  DefMacro!("\\physrep", "Phys. Rep.");
  DefMacro!("\\ssr", "Space Sci. Rev.");
  DefMacro!("\\jcap", "J. Cosmology Astropart. Phys.");
  DefMacro!("\\solphys", "Sol. Phys.");
  DefMacro!("\\lrr", "Living Rev. Relativity");
  DefMacro!("\\na", "New A");
  DefMacro!("\\nar", "New A Rev.");

  // Bold Greek — Perl L66-97
  DefMacro!("\\mn@boldsymbol{}", "\\boldsymbol{#1}");
  DefMacro!("\\balpha", "\\mn@boldsymbol{\\alpha}");
  DefMacro!("\\bbeta", "\\mn@boldsymbol{\\beta}");
  DefMacro!("\\bgamma", "\\mn@boldsymbol{\\gamma}");
  DefMacro!("\\bdelta", "\\mn@boldsymbol{\\delta}");
  DefMacro!("\\bepsilon", "\\mn@boldsymbol{\\epsilon}");
  DefMacro!("\\bzeta", "\\mn@boldsymbol{\\zeta}");
  DefMacro!("\\boldeta", "\\mn@boldsymbol{\\eta}");
  DefMacro!("\\btheta", "\\mn@boldsymbol{\\theta}");
  DefMacro!("\\biota", "\\mn@boldsymbol{\\iota}");
  DefMacro!("\\bkappa", "\\mn@boldsymbol{\\kappa}");
  DefMacro!("\\blambda", "\\mn@boldsymbol{\\lambda}");
  DefMacro!("\\bmu", "\\mn@boldsymbol{\\mu}");
  DefMacro!("\\bnu", "\\mn@boldsymbol{\\nu}");
  DefMacro!("\\bxi", "\\mn@boldsymbol{\\xi}");
  DefMacro!("\\bpi", "\\mn@boldsymbol{\\pi}");
  DefMacro!("\\brho", "\\mn@boldsymbol{\\rho}");
  DefMacro!("\\bsigma", "\\mn@boldsymbol{\\sigma}");
  DefMacro!("\\btau", "\\mn@boldsymbol{\\tau}");
  DefMacro!("\\bupsilon", "\\mn@boldsymbol{\\upsilon}");
  DefMacro!("\\bphi", "\\mn@boldsymbol{\\phi}");
  DefMacro!("\\bchi", "\\mn@boldsymbol{\\chi}");
  DefMacro!("\\bpsi", "\\mn@boldsymbol{\\psi}");
  DefMacro!("\\bomega", "\\mn@boldsymbol{\\omega}");

  // Degree fractions — Perl L109-117
  DefMacro!("\\aas@fstack{}", "\\ensuremath{.\\!^{\\mathrm{#1}}}");

  // Math relations — Perl L131-149
  DefMath!("\\sol", "\u{2A9D}", role => "RELOP", meaning => "similar-to-or-less-than");
  DefMath!("\\sog", "\u{2A9E}", role => "RELOP", meaning => "similar-to-or-greater-than");
  DefMath!("\\lse", "\u{2A8D}", role => "RELOP", meaning => "less-than-or-similar-to-or-equal");
  DefMath!("\\gse", "\u{2A8E}", role => "RELOP", meaning => "greater-than-or-similar-to-or-equal");
  DefMath!("\\leogr", "\u{2276}", role => "RELOP", meaning => "less-than-or-greater-than");
  DefMath!("\\grole", "\u{2277}", role => "RELOP", meaning => "greater-than-or-less-than");
  DefMath!("\\loa", "\u{2A85}", role => "RELOP", meaning => "less-than-or-approximately-equals");
  DefMath!("\\goa", "\u{2A86}", role => "RELOP", meaning => "greater-than-or-approximately-equals");
  DefMath!("\\lid", "\u{2266}", role => "RELOP", meaning => "less-than-or-equals");
  DefMath!("\\gid", "\u{2267}", role => "RELOP", meaning => "greater-than-or-equals");
  DefMath!("\\leqslant", "\u{2A7D}", role => "RELOP", meaning => "less-than-or-equals");
  DefMath!("\\geqslant", "\u{2A7E}", role => "RELOP", meaning => "greater-than-or-equals");
  DefPrimitive!("\\micron", "\u{00B5}m");

  // Font macros — Perl L153-161
  DefMacro!("\\rmn{}", "\\mathrm{#1}");
  DefMacro!("\\romn{}", "\\mathrm{#1}");
  DefMacro!("\\itl{}", "\\mathit{#1}");
  DefMacro!("\\bld{}", "\\mathbf{#1}");
  DefMacro!("\\textbfit{}", "\\textbf{\\textit{#1}}");
  DefMacro!("\\textbfss{}", "\\textbf{\\textsf{#1}}");
  DefMacro!("\\bmath{}", "\\mn@boldsymbol{#1}");

  Let!("\\upi", "\\pi");
  Let!("\\umu", "\\mu");
  Let!("\\upartial", "\\partial");

  // Table/proof — Perl L174-192
  DefMacro!("\\contcaption", "\\caption{continued}");
  DefMacro!("\\proofname", "Proof");
  DefEnvironment!("{lquote}", "<ltx:quote>#body</ltx:quote>");

  DefMacro!("\\loadboldmathitalic", "");
  DefMacro!("\\loadboldgreek", "");
  DefMacro!("\\fixfootnotes", "");
  DefMacro!("\\bibtitle", "References");
  DefMacro!("\\makeRLlabel{}", "#1");
  DefMacro!("\\makeRRlabel{}", "#1");
  DefMacro!("\\makenewlabel{}", "#1");
  DefMacro!("\\boxit{}", "#1");
  DefRegister!("\\smallindent" => Glue!("1.5em"));
  Let!("\\fullhline", "\\hline");
  DefMacro!("\\sevensize", "\\small");
  DefMacro!("\\plate", "");

  Let!("\\@internalcite", "\\cite");
  DefMacro!("\\shortcite", "\\cite");
  DefMacro!("\\citename{}", "#1");
});
