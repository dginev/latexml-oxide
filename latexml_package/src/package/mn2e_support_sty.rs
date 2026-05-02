//! mn2e_support.sty — MNRAS (Monthly Notices of the Royal Astronomical Society) support
//! Perl: mn2e_support.sty.ltxml — 252 lines
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Dependencies — Perl mn2e_support.sty.ltxml L18-23 conditionally loads
  // only dcolumn/natbib/graphicx based on the matching option flags.
  // NOTE: Perl does NOT load amsmath even when @useAMS is set — the
  // raw-TeX `\if@useAMS\RequirePackage{amsmath,amssymb}\fi` from the .cls
  // is DELIBERATELY bypassed. Loading amsmath makes `\cases` route through
  // the amsmath `\lx@ams@cases@` constructor (DigestedBody — no explicit
  // `\lx@end@alignment` close), whereas the base `\cases` from
  // Base_XMath (`\lx@gen@plain@cases`) wraps the body with
  // `\lx@end@alignment` which provides the clean termination.
  // Regression path: paper 1112.6246 (`giersz_rv1.tex`, mn2e class)
  // cascades 10001 mode-leak errors if amsmath is loaded here.
  if state::lookup_int("@usedcolumn") != 0 {
    RequirePackage!("dcolumn");
  }
  if state::lookup_int("@usenatbib") != 0 {
    RequirePackage!("natbib");
  }
  if state::lookup_int("@usegraphicx") != 0 {
    RequirePackage!("graphicx");
  }

  // Frontmatter — Perl L28-46
  DefMacro!("\\title[]{}", "\\@add@frontmatter{ltx:title}{#2}");
  // Perl L31:
  //   DefMacro('\author[]{}', sub { andSplit(T_CS('\lx@author'), $_[2]); });
  // $_[2] is the mandatory body (author list); the optional `[short]` is
  // consumed and discarded.
  DefMacro!("\\author[]{}", sub[(_short, authors)] {
    and_split(T_CS!("\\lx@author"), authors)
  });
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

  // Keywords — Perl mn2e_support.sty.ltxml L48-54:
  //   DefEnvironment('{keywords}', '',
  //     afterDigest => sub { push 'ltx:classification'→frontmatter });
  //
  // As an environment, `\endkeywords` is auto-defined, so raw
  // `mn2e-breakabs.sty` redefinitions of `\endkeywords` (which reference
  // undefined `\SFB@keywordstrue`) never fire. Body digests as frontmatter
  // classification entry via `\@add@frontmatter`.
  DefEnvironment!("{keywords}",
    "<ltx:classification scheme='keywords'>#body</ltx:classification>");
  DefMacro!("\\keywords{}", "\\@add@frontmatter{ltx:keywords}{#1}");

  // Perl L186: `\bsp` is a no-op DefMacro (not DefConstructor).
  DefMacro!("\\bsp", "");

  // Math shortcuts — Perl mn2e_support.sty.ltxml L131-145.
  // Perl binds these directly via DefMath, NOT by aliasing to amssymb
  // CSes — mn2e_support is intentionally amssymb-free (see top-of-file
  // comment on the dropped amsmath/amssymb RequirePackage). Aliasing
  // \la → \lesssim leaves \la dangling whenever a paper doesn't load
  // amssymb separately (sandbox 0911.3798, ~21 papers).
  DefMath!("\\la", "\u{2272}", role => "RELOP",
    meaning => "less-than-or-similar-to");
  DefMath!("\\ga", "\u{2273}", role => "RELOP",
    meaning => "greater-than-or-similar-to");
  DefMath!("\\getsto", "\u{21C6}", role => "ARROW");
  DefMacro!("\\sun", "\u{2609}");
  DefMacro!("\\degr", "\u{00B0}");
  DefMacro!("\\arcmin", "\u{2032}");
  DefMacro!("\\arcsec", "\u{2033}");
  // Perl mn2e_support.sty.ltxml L106-109,113: \fd/\fh/\fm/\fs/\fp use \aas@fstack.
  // \aas@fstack wraps in \ensuremath so it works in both text and math contexts.
  DefMacro!("\\fd", "\\aas@fstack{d}");
  DefMacro!("\\fh", "\\aas@fstack{h}");
  DefMacro!("\\fm", "\\aas@fstack{m}");
  DefMacro!("\\fs", "\\aas@fstack{s}");
  DefMacro!("\\fp", "\\aas@fstack{p}");
  // Perl: mn2e_support.sty.ltxml — degree/arcmin/arcsec using \aas@fstack
  DefMacro!("\\fdg", "\\aas@fstack{\\circ}");
  DefMacro!("\\farcm", "\\aas@fstack{\\prime}");
  DefMacro!("\\farcs", "\\aas@fstack{\\prime\\prime}");
  DefMacro!("\\ion{}{}", "#1\\,{\\sc #2}");

  // Journal abbreviations (\mnras, \nat, \apj, \prd, ...) are NOT defined
  // in Perl mn2e_support.sty.ltxml. They live in aas_macros.sty.ltxml
  // (ported to aas_macros_sty.rs) where they wrap via \ref@jnl{...}.

  // Bold Greek — Perl L66-97. `\mn@boldsymbol` is a self-contained
  // DefConstructor in Perl (L66): `'#1', bounded => 1, requireMath => 1,
  // font => { forcebold => 1 }`. The earlier Rust port routed via
  // `\boldsymbol{#1}` (DefMacro alias), but mn2e_support is intentionally
  // amsmath/amsbsy-free (see top-of-file comment) so `\boldsymbol` is
  // undefined and the alias errors out the moment any `\b<greek>` shortcut
  // is used. Sandbox 100k stage 1 sample (round-18): astro-ph0001132
  // `\documentstyle{mn}` paper failed conversion with
  // `Error:undefined:\boldsymbol`. Round-18 fix: faithful DefConstructor
  // mirror of Perl L66.
  DefConstructor!("\\mn@boldsymbol{}", "#1",
    bounded => true, require_math => true,
    font => {forcebold => true});
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
  // Perl L90-95: bold variant-Greek
  DefMacro!("\\bvarepsilon", "\\mn@boldsymbol{\\varepsilon}");
  DefMacro!("\\bvartheta", "\\mn@boldsymbol{\\vartheta}");
  DefMacro!("\\bvarrho", "\\mn@boldsymbol{\\varrho}");
  DefMacro!("\\bvarsigma", "\\mn@boldsymbol{\\varsigma}");
  DefMacro!("\\bvarphi", "\\mn@boldsymbol{\\varphi}");
  DefMacro!("\\bvarpi", "\\mn@boldsymbol{\\varpi}");

  // Degree fractions — Perl L101-117: constructor + macro form (semantic POSTFIX XMApp)
  DefConstructor!("\\aas@@fstack{}",
    "<ltx:XMApp role='POSTFIX'><ltx:XMTok role='SUPERSCRIPTOP' scriptpos='#scriptpos'/><ltx:XMTok>.</ltx:XMTok><ltx:XMWrap>#1</ltx:XMWrap></ltx:XMApp>",
    mode => "math", bounded => true,
    properties => sub[_args] {
      let script_level = state::lookup_int("script_level");
      Ok(stored_map!("scriptpos" => s!("mid{}", script_level)))
    });
  DefMacro!("\\aas@fstack{}", "\\ensuremath{\\aas@@fstack{#1}}");

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
  DefMath!("\\cor", "\u{2258}", role => "RELOP", meaning => "corresonds-to");
  DefPrimitive!("\\micron", "\u{00B5}m");

  // Perl L122-125: quod erat demonstrandum marker
  DefConstructor!("\\squareforqed",
    "?#isMath(<ltx:XMTok role='PUNCT'>\u{220E}</ltx:XMTok>)(\u{220E})");
  Let!("\\sq", "\\squareforqed");
  Let!("\\proofbox", "\\squareforqed");

  // Perl L128-129: astronomical symbols
  DefPrimitive!("\\diameter", "\u{2300}");
  DefPrimitive!("\\earth", "\u{2295}");

  // Perl L162-165: pre-AMS aliases
  Let!("\\oldle", "\\le");
  Let!("\\oldleq", "\\leq");
  Let!("\\oldge", "\\ge");
  Let!("\\oldgeq", "\\geq");

  // Font macros — Perl L153-161
  DefMacro!("\\rmn{}", "\\mathrm{#1}");
  DefMacro!("\\romn{}", "\\mathrm{#1}");
  DefMacro!("\\itl{}", "\\mathit{#1}");
  DefMacro!("\\bld{}", "\\mathbf{#1}");
  DefMacro!("\\textbfit{}", "\\textbf{\\textit{#1}}");
  DefMacro!("\\textbfss{}", "\\textbf{\\textsf{#1}}");
  DefMacro!("\\mathbfit{}", "\\textbf{\\textit{#1}}");
  DefMacro!("\\mathbfss{}", "\\textbf{\\textsf{#1}}");
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
  DefMacro!("\\nokeywords", "");
  DefMacro!("\\bibtitle", "References");
  DefMacro!("\\bibheadtitle", "REFERENCES");
  DefMacro!("\\makeRLlabel{}", "#1");
  DefMacro!("\\makeRRlabel{}", "#1");
  DefMacro!("\\makenewlabel{}", "#1");
  DefMacro!("\\boxit{}", "#1");
  DefRegister!("\\smallindent" => Glue!("1.5em"));
  Let!("\\fullhline", "\\hline");
  DefMacro!("\\sevensize", "\\small");
  DefMacro!("\\plate", "");

  // Perl L57-62: equation numbering schemes
  DefMacro!("\\eqsecnum",
    "\\@addtoreset{equation}{section}\\def\\theequation{\\arabic{section}.\\arabic{equation}}");
  DefMacro!("\\eqsubsecnum",
    "\\@addtoreset{equation}{subsection}\\def\\theequation{\\arabic{subsection}.\\arabic{equation}}");

  // Perl L204-205: utility macros
  DefMacro!("\\hexnumber{}", sub[(n)] {
    let n = n.to_string().trim().parse::<i64>().unwrap_or(0);
    Ok(Tokens!(T_OTHER!(format!("{:x}", n))))
  });
  DefMacro!("\\mathch{}{}", "\\ensuremath{#2}");

  Let!("\\@internalcite", "\\cite");
  DefMacro!("\\shortcite", "\\cite");
  DefMacro!("\\citename{}", "#1");

  // Perl mn2e_support.sty.ltxml L212-245: "Redefine equations (bizarrely)
  // to allow $ within" — rebind T_MATH inside display math so a literal
  // `$` becomes a no-op, not an attempt to close math. MN (Monthly
  // Notices) papers routinely write idioms like
  //   T_0 =\, $HJD$\, 2453195.2859 \pm 0.0003
  // inside `\begin{equation} … \end{equation}` where `$HJD$` is a
  // text-escape to typeset "HJD" in roman. Without the override, Rust
  // raised Error:expected:$ Missing $ closing display math at every
  // occurrence. Matches the identical port in aa_support_sty.
  use crate::engine::latex_constructs::{
    after_equation, before_equation, prepare_equation_counter,
  };
  DefEnvironment!(
    "{equation}",
    "<ltx:equation xml:id='#id'>#tags<ltx:Math mode='display'><ltx:XMath>#body</ltx:XMath></ltx:Math></ltx:equation>",
    mode => "display_math",
    before_digest => {
      prepare_equation_counter(stored_map!("numbered" => true, "preset" => true));
      before_equation()?;
      Let!(T_MATH!(), "\\lx@dollar@in@mathmode");
    },
    after_digest_body => sub[whatsit] {
      after_equation(Some(whatsit))?;
    },
    locked => true);
  DefEnvironment!(
    "{equation*}",
    "<ltx:equation xml:id='#id'>#tags<ltx:Math mode='display'><ltx:XMath>#body</ltx:XMath></ltx:Math></ltx:equation>",
    mode => "display_math",
    before_digest => {
      prepare_equation_counter(stored_map!("preset" => true));
      before_equation()?;
      Let!(T_MATH!(), "\\lx@dollar@in@mathmode");
    },
    after_digest_body => sub[whatsit] {
      after_equation(Some(whatsit))?;
    },
    locked => true);

  // Perl mn2e_support.sty.ltxml L200-201 — declare two boolean ifs that
  // mn2e papers test against later. \ifCUPmtlplainloaded gates a CUP
  // plain-mode branch, \iffirstta gates the first-table-author flag.
  // Sandbox astro-ph0207632 + astro-ph9807011 + astro-ph9909211 +
  // astro-ph9907099 hit `\ifCUPmtlplainloaded` undefined.
  RawTeX!(r"\newif\ifCUPmtlplainloaded");
  RawTeX!(r"\newif\iffirstta");
});
