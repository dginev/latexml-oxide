//! mathastext.sty — math font substitution.
//!
//! Like fdsymbol.sty, mathastext L1075 runs `\let\lbrace\undefined`
//! before `\DeclareMathDelimiter{\lbrace}`. Our `:locked` guard
//! blocks the redef, leaving `\lbrace` (and `\rbrace`, etc.) UNDEFINED
//! and cascading 500+ math-mode errors per paper.
//!
//! Perl LaTeXML has no mathastext binding and skips it via
//! INCLUDE_STYLES=false. Match with a no-op binding — math-font
//! substitution is moot for XML/HTML output.
//!
//! Witness 2410.03274 (\lbrace cascade with 505 errors).
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Don't run mathastext's math-symbol slot reassignments: the kernel \lbrace / \rbrace / \langle / etc. remain
  // authoritative. Its user commands choose fonts and math spacing, presentation the XML does not carry, so they
  // take their arguments and do nothing (mathastext.sty 1.4e): `\MTsetmathskips{<char>}{<skip>}{<skip>}` (:964, 54
  // papers; 1712.02869, 1804.04170), `\MTunsetmathskips` (:968); the font choices `\MTencoding`, `\MTfamily`,
  // `\MTseries`, `\MTshape`, `\MTboldvariant`, `\MTlettershape` (:837-842, with their `\Mathastext…` names
  // :843-848), `\MTgreekfont` (:857-858); the switches `\MTitgreek`, `\MTupgreek` (:849-850), `\MTmathactiveletters`,
  // `\MTmathstandardletters`, `\MTicinmath`, `\MTnoicinmath` (:934-958), `\MT(easy)nonletters(donot)obeymathxx`
  // (:490-549), `\MTexplicitbraces(donot)obeymathxx` (:1473-1477); `\MTWillUse[…]{…}{…}{…}{…}` (:2677). The
  // versions are math versions: `\MTDeclareVersion[…]{<name>}{…}{…}{…}{…}` declares `<name>` (:2411-2420) and
  // `\Mathastext[<name>]` through it (:2688-2697), and `\MTversion{<name>}` (`[<text version>]{<name>}`, or
  // `*{<name>}`) selects it with `\mathversion` first (:2585-2588; `\MTversion{bold}` is bold math), its text fonts
  // not modelled. Its `\Mathsf`/`\Mathtt` alphabets (:701-702, the text fonts' sans and typewriter) are
  // `\mathsf`/`\mathtt`.
  def_macro_noop("\\MTsetmathskips{}{}{}")?;
  def_macro_noop("\\MTunsetmathskips{}")?;
  for cs in ["\\MTencoding", "\\MTfamily", "\\MTseries", "\\MTshape", "\\MTboldvariant", "\\MTlettershape",
    "\\MTgreekfont"] {
    def_macro_noop(&s!("{cs}{{}}"))?;
  }
  for (alias, cs) in [("\\Mathastextencoding", "\\MTencoding"), ("\\Mathastextfamily", "\\MTfamily"),
    ("\\Mathastextseries", "\\MTseries"), ("\\Mathastextshape", "\\MTshape"),
    ("\\Mathastextboldvariant", "\\MTboldvariant"), ("\\Mathastextlettershape", "\\MTlettershape"),
    ("\\Mathastextgreekfont", "\\MTgreekfont")] {
    Let!(&T_CS!(alias), &T_CS!(cs));
  }
  for cs in ["\\MTitgreek", "\\MTupgreek", "\\MTmathactiveletters", "\\MTmathstandardletters", "\\MTicinmath",
    "\\MTnoicinmath", "\\MTnonlettersobeymathxx", "\\MTnonlettersdonotobeymathxx", "\\MTeasynonlettersobeymathxx",
    "\\MTeasynonlettersdonotobeymathxx", "\\MTexplicitbracesobeymathxx", "\\MTexplicitbracesdonotobeymathxx"] {
    def_macro_noop(cs)?;
  }
  DefMacro!("\\MTDeclareVersion[]{}{}{}{}{}", "\\DeclareMathVersion{#2}");
  Let!("\\MathastextDeclareVersion", "\\MTDeclareVersion");
  DefMacro!("\\Mathastext[]", "\\if\\relax\\detokenize{#1}\\relax\\else\\DeclareMathVersion{#1}\\fi");
  def_macro_noop("\\MTWillUse[]{}{}{}{}")?;
  Let!("\\MathastextWillUse", "\\MTWillUse");
  Let!("\\Mathastextwilluse", "\\MTWillUse");
  DefMacro!("\\MTversion OptionalMatch:* []{}", "\\mathversion{#3}");
  for alias in ["\\MathastextVersion", "\\Mathastextversion", "\\MTVersion", "\\mathastextversion"] {
    Let!(&T_CS!(alias), &T_CS!("\\MTversion"));
  }
  Let!("\\Mathsf", "\\mathsf");
  Let!("\\Mathtt", "\\mathtt");
});
