//! mn2e_support.sty — MNRAS (Monthly Notices of the Royal Astronomical Society) support
//! Perl: mn2e_support.sty.ltxml — 252 lines
//!
//! ## Def*-kind divergence from Perl (audit-flagged, intentional)
//!
//! 9 DP audit entries across a mixed kind distribution — most share
//! structural patterns documented elsewhere:
//! - 4 DefPrimitiveI↔DefMacro for literal-text astronomy symbols (\sun ☉, \degr °, \arcmin ′,
//!   \arcsec ″) — same Rust-idiom pattern as babel_support_sty.rs.
//! - 3 DefMath↔DefMacro for math shortcuts (\la, \ga, \getsto) — Rust uses DefMacro aliases to
//!   existing LaTeX math CSes, matching the LaTeX-shim approach in amsppt_sty.rs (WISDOM #42).
//! - 2 outliers (\mn@boldsymbol DefMacro↔DefConstructor, \bsp DefConstructor↔DefMacro) for local
//!   formatting — structural adaptations, no parity bug.
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

  // Keywords — Perl mn2e_support.sty.ltxml L48-54 registers the env with
  //   afterDigest => push ['ltx:classification',{scheme=>'keywords'},body]
  //   onto LookupValue('frontmatter')
  // so the classification surfaces in the document's frontmatter rather
  // than inline at the `\begin{keywords}` position. Rust's prior port
  // emitted it inline as env body (missed the frontmatter push).
  // Port via DefEnvironment expanding to `\@add@frontmatter` — same
  // idiom used by companion `\keywords{}` macro. Perl's `\@add@frontmatter`
  // schedules the attribute subtree for later frontmatter placement.
  DefEnvironment!("{keywords}",
    "\\@add@frontmatter{ltx:classification}[scheme=keywords]{#body}");
  DefMacro!("\\keywords{}",
    "\\@add@frontmatter{ltx:classification}[scheme=keywords]{#1}");
  DefMacro!("\\nokeywords", "");

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

  // Acknowledgements — Perl L186 defines `\bsp` as DefMacro('' )
  // (silent expand-to-nothing). Rust had DefConstructor with empty
  // template, which would emit a whatsit in the digest tree — a
  // visible no-op, but needlessly heavyweight. DefMacro matches
  // Perl's kind + leaves no trace in the digest stream.
  DefMacro!("\\bsp", "");
  Let!("\\ackn", "\\acknowledgments");
  DefMacro!("\\acknowledgments", "\\section*{Acknowledgments}");

  // Math shortcuts — Perl mn2e_support.sty.ltxml L118-131,145.
  // The Perl bindings are DefPrimitiveI for text glyphs and DefMath for
  // math RELOPs/ARROWs — each carries the correct Unicode + role/meaning
  // directly. A prior Rust port aliased these via DefMacro to the nearest
  // existing LaTeX command (\la→\lesssim etc.); that aliasing was close
  // enough for the text glyphs but wrong for \getsto (\rightleftharpoons
  // = U+21CC, two harpoons) whereas Perl \getsto = U+21C6 (two arrows).
  // Port to match Perl kind + exact Unicode.
  DefPrimitive!("\\sun", "\u{2609}");
  DefPrimitive!("\\degr", "\u{00B0}");
  DefPrimitive!("\\arcmin", "\u{2032}");
  DefPrimitive!("\\arcsec", "\u{2033}");
  DefMath!("\\la", "\u{2272}", role => "RELOP", meaning => "less-than-or-similar-to");
  DefMath!("\\ga", "\u{2273}", role => "RELOP", meaning => "greater-than-or-similar-to");
  DefMath!("\\getsto", "\u{21C6}", role => "ARROW");
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

  // Bold Greek — Perl mn2e_support.sty.ltxml L66-97.
  // Perl L66: `DefConstructor('\mn@boldsymbol{}', '#1', bounded => 1,
  //   requireMath => 1, font => { forcebold => 1 })` — a bounded
  // constructor that forces bold font on its argument.
  // Rust short-circuits to `\boldsymbol{#1}` (amsmath) which already
  // does the same `bounded + forcebold` work via its own
  // DefConstructor. The trampoline is kind-wise a DefMacro rather
  // than the font-forcing DefConstructor, but the resolved emit is
  // observationally identical (same XMApp+XMTok role='NUMBER' bold
  // + nested XMArg wrapping). Intentional DefConstructor → DefMacro
  // kind divergence (WISDOM #44) — delegating to the existing
  // `\boldsymbol` is simpler than re-implementing the bold-font glue.
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
  // Perl L177-181 ships {proof} with afterConstruct =>
  // maybeCloseElement('ltx:proof'). Rust doesn't expose
  // maybe_close_element from a DefEnvironment after_construct hook
  // ergonomically, so we use the equivalent full-template form
  // (matches elsart_support_sty.rs proof env), which already
  // includes the </ltx:proof> close so no after_construct hook is
  // needed. Title comes from Digest(\proofname) in Perl; we resolve
  // it through stored_map directly since \proofname always expands
  // to "Proof" in this package.
  DefEnvironment!("{proof}",
    "<ltx:proof><ltx:title font='italic' _force_font='true' class='ltx_runin'>#title</ltx:title>#body</ltx:proof>",
    properties => { stored_map!("title" => Stored::from("Proof")) }
  );
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
});
