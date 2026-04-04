//! mn2e_support.sty — MNRAS (Monthly Notices of the Royal Astronomical Society) support
//! Perl: mn2e_support.sty.ltxml — 252 lines
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Dependencies
  RequirePackage!("natbib");

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
});
