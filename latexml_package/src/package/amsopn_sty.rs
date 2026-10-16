use crate::prelude::*;
LoadDefinitions!({
  RequirePackage!("amsgen");

  // \DeclareMathOperator*{cs}{text} — Perl L23-29: `DefMathI($cs, undef,
  // Invocation(T_CS('\operatorname'), $star, $text), …)`, a token presentation, so the DefMath is
  // the wrapped kind (Package.pm:1656-1660, `def_math_wrapped`): `\cs` expands to
  // `\cs@wrapper{\cs@presentation}` and the body stays the tokens as written — unexpanded until use,
  // with their catcodes — inside `\operatorname`'s upright operator font. The name is the control
  // sequence's (it differs from the presentation, Package.pm:1628-1632), the wrapper's `XMWrap`
  // carries it with the role and scriptpos onto the parsed operator, and the formula's `tex=` reads
  // `\operatorname{…}` (the wrapper reverts as its argument). An earlier string round trip had to
  // untex `{\rm Aut}` (0806.2705, 0808.0535), expand expl3 names at definition (numerica.sty:50-51)
  // under `\protected@edef` (pm-isomath.sty:185, euclideangeometry-man) and keep a body naming an
  // undefined control sequence verbatim (iidef.sty:147, thucoursework ithw); the tokens need none
  // of it. Guards `perfect_kernel_batch56::{declaremathoperator_keeps_protected_macros,
  // declaremathoperator_body_stays_lazy, declaremathoperator_keeps_the_following_letters_italic}`;
  // golden tests/parse/declared_operators.tex.
  DefPrimitive!("\\DeclareMathOperator OptionalMatch:* {Token} {}", sub[(star, cs, text)] {
    let has_star = star.is_some();
    let mut presentation = vec![T_CS!("\\operatorname")];
    if has_star {
      presentation.push(T_OTHER!("*"));
    }
    presentation.push(T_BEGIN!());
    presentation.extend(text.unlist());
    presentation.push(T_END!());
    // Perl L26-29: scriptpos => ($star ? \&doScriptpos : 'post') — the starred form's is `mid` in
    // display, read with the wrapper's properties at digestion (dialect.rs); revert_as => 'context'
    // (Perl passes it; the wrapped DefMath does not read it).
    let opts = MathPrimitiveOptions {
      role: Some(if has_star { "OPERATOR" } else { "OPFUNCTION" }.to_string()),
      font: Some(fontmap!(family => "serif", series => "medium", shape => "upright").into()),
      scriptpos: if has_star { None } else { Some("post".to_string()) },
      dynamic_scriptpos: has_star,
      revert_as: Some(Cow::Borrowed("context")),
      ..Default::default()};
    def_math(cs, None, Tokens::new(presentation), opts)?;
  });

  // \operatorname*{text}
  DefConstructor!("\\operatorname OptionalMatch:* {}",
  "<ltx:XMWrap role='#role' scriptpos='#scriptpos'>#2</ltx:XMWrap>",
  bounded => true, require_math => true,
  font => { family => "serif", series => "medium", shape => "upright" },
  properties => sub[args] {
    let starred = args[0].is_some();
    let role = if starred { "OPERATOR" } else { "OPFUNCTION" };
    let scriptpos = if starred { "mid" } else { "post" };
    Ok(stored_map!("role" => role, "scriptpos" => scriptpos))
  });

  DefConstructor!("\\operatornamewithlimits {}",
    "<ltx:XMWrap role='OPERATOR' scriptpos='mid'>#1</ltx:XMWrap>",
    bounded => true, require_math => true,
    font => { family => "serif", series => "medium", shape => "upright" });

  // Real amsopn.sty L56-89 UNCONDITIONALLY re-asserts every classic log-like
  // operator (`\protected\def\arg{\qopname\relax o{arg}}` …). That matters:
  // documentation classes clobber some of them before amsmath loads
  // (amsldoc.cls L205 makes `\arg{1}` doc-markup for a macro parameter), and
  // real LaTeX gets the operator back here. Perl's amsopn.sty.ltxml SHARES
  // the omission (relies on the kernel defs) — witness amsmath/amsldoc:
  // `$\arg$` ate its closing `$` and cascaded to 101 errors; pdflatex is the
  // oracle. Table mirrors latexml_engine/src/math_common.rs L1188-1235
  // exactly (roles/meanings/scriptpos) — keep the two in sync.
  DefMath!("\\arccos", "arccos", role => "OPFUNCTION", meaning => "inverse-cosine");
  DefMath!("\\arcsin", "arcsin", role => "OPFUNCTION", meaning => "inverse-sine");
  DefMath!("\\arctan", "arctan", role => "OPFUNCTION", meaning => "inverse-tangent");
  DefMath!("\\arg", "arg", role => "OPFUNCTION", meaning => "argument");
  DefMath!("\\cos", "cos", role => "TRIGFUNCTION", meaning => "cosine");
  DefMath!("\\cosh", "cosh", role => "TRIGFUNCTION", meaning => "hyperbolic-cosine");
  DefMath!("\\cot", "cot", role => "TRIGFUNCTION", meaning => "cotangent");
  DefMath!("\\coth", "coth", role => "TRIGFUNCTION", meaning => "hyperbolic-cotangent");
  DefMath!("\\csc", "csc", role => "TRIGFUNCTION", meaning => "cosecant");
  DefMath!("\\deg", "deg", role => "OPFUNCTION", meaning => "degree");
  DefMath!("\\det", None, "det", role => "LIMITOP", meaning => "determinant",
    dynamic_scriptpos => true);
  DefMath!("\\dim", "dim", role => "LIMITOP", meaning => "dimension");
  DefMath!("\\exp", "exp", role => "OPFUNCTION", meaning => "exponential");
  DefMath!("\\gcd", None, "gcd", role => "OPFUNCTION", meaning => "gcd",
    dynamic_scriptpos => true);
  DefMath!("\\hom", "hom", role => "OPFUNCTION");
  DefMath!("\\inf", None, "inf", role => "LIMITOP", meaning => "infimum",
    dynamic_scriptpos => true);
  DefMath!("\\ker", "ker", role => "OPFUNCTION", meaning => "kernel");
  DefMath!("\\lg", "lg", role => "OPFUNCTION");
  DefMath!("\\lim", None, "lim", role => "LIMITOP", meaning => "limit",
    dynamic_scriptpos => true);
  DefMath!("\\liminf", None, "lim inf", role => "LIMITOP", meaning => "limit-infimum",
    dynamic_scriptpos => true);
  DefMath!("\\limsup", None, "lim sup", role => "LIMITOP", meaning => "limit-supremum",
    dynamic_scriptpos => true);
  DefMath!("\\ln", "ln", role => "OPFUNCTION", meaning => "natural-logarithm");
  DefMath!("\\log", "log", role => "OPFUNCTION", meaning => "logarithm");
  DefMath!("\\max", None, "max", role => "OPFUNCTION", meaning => "maximum",
    dynamic_scriptpos => true);
  DefMath!("\\min", None, "min", role => "OPFUNCTION", meaning => "minimum",
    dynamic_scriptpos => true);
  DefMath!("\\Pr", None, "Pr", role => "OPFUNCTION",
    dynamic_scriptpos => true);
  DefMath!("\\sec", "sec", role => "TRIGFUNCTION", meaning => "secant");
  DefMath!("\\sin", "sin", role => "TRIGFUNCTION", meaning => "sine");
  DefMath!("\\sinh", "sinh", role => "TRIGFUNCTION", meaning => "hyperbolic-sine");
  DefMath!("\\sup", None, "sup", role => "LIMITOP", meaning => "supremum",
    dynamic_scriptpos => true);
  DefMath!("\\tan", "tan", role => "TRIGFUNCTION", meaning => "tangent");
  DefMath!("\\tanh", "tanh", role => "TRIGFUNCTION", meaning => "hyperbolic-tangent");

  // Operator variants — Perl L33-38 ships scriptpos => \&doScriptpos so the
  // operators sit mid (under/over) in display style and post (sub/super) in
  // inline. Without it, Rust statically rendered everything as 'post', giving
  // wrong placement in display-mode formulas.
  DefMath!("\\injlim", "inj lim",
    role => "LIMITOP", meaning => "injective-limit", dynamic_scriptpos => true);
  DefMath!("\\projlim", "proj lim",
    role => "LIMITOP", meaning => "projective-limit", dynamic_scriptpos => true);

  // Perl: amsopn.sty.ltxml — var limit operators
  DefMath!("\\varlimsup", "\\overline{\\operatorname{lim}}",
    role => "LIMITOP", meaning => "limit-supremum");
  DefMath!("\\varliminf", "\\underline{\\operatorname{lim}}",
    role => "LIMITOP", meaning => "limit-infimum");
  DefMath!("\\varinjlim", "\\underrightarrow{\\operatorname{lim}}",
    role => "LIMITOP", meaning => "injective-limit");
  DefMath!("\\varprojlim", "\\underleftarrow{\\operatorname{lim}}",
    role => "LIMITOP", meaning => "projective-limit");

  // amsopn.sty:92-97 `\varlim@{<style>}{<fill>\<style>}`, the builder of `\varinjlim`/`\varprojlim` (an
  // `\operator@font lim` over an arrow fill), which papers call to define their own direct and inverse limits
  // (`\mathop{\mathpalette\varlim@{\leftarrowfill@\scriptstyle}}\nmlimits@`, 23 amsart papers in run 353 with it
  // undefined: 1608.08273, 1807.11932, 1909.11399). It renders as those two do, the arrow chosen from the fill macro as
  // amsmath's `\underarrow@` chooses it (amsmath_sty.rs). Perl's binding lacks it (KNOWN_PERL_ERRORS #696).
  RawTeX!(
    r"\def\varlim@#1#2{\lx@varlim@#2\relax}
\def\lx@varlim@#1#2\relax{\ifx#1\leftarrowfill@\underleftarrow{\operatorname{lim}}\else\underrightarrow{\operatorname{lim}}\fi}"
  );
  // amsopn.sty:105-108, `\varliminf@`/`\varlimsup@{<style>}` (an `\operator@font lim` under or over a
  // rule), the builders of `\varliminf`/`\varlimsup` that papers call to define their own (`\renewcommand{\limsup}{
  // \mathop{\mathpalette\varlimsup@{\vphantom{p}}}\nmlimits@}`, 2005.13511). They render as those two do, as `\varlim@`
  // does above. Perl's binding lacks them (KNOWN_PERL_ERRORS #801).
  RawTeX!(
    r"\def\varliminf@#1{\underline{\operatorname{lim}}}
\def\varlimsup@#1{\overline{\operatorname{lim}}}"
  );
  // amsopn.sty:28-42 `\newmcodes@`, the math codes an operator name is set with (`'`, `*`, `.`, `-`, `/`, `:` as
  // ordinary characters, the minus kept in `\std@minus`); the pdfTeX branch, as LaTeXML is pdfTeX (`\Umathcode`
  // undefined), its hexadecimal `"613A`/`"603A` written in decimal (the file reads them under `\catcode`\"=12`, :27).
  // Papers set variable names with it, `\begingroup\newmcodes@\mathit{#1}\endgroup` (11 papers: 1511.04387,
  // 1712.08220, 1802.09883). amsopn.sty:51-53 `\@declmathop{<m|o>}{<cs>}{<text>}`, `\DeclareMathOperator`'s worker
  // after its star test, which papers call to declare operators in bulk (`\expandafter\@declmathop\expandafter
  // #1\csname #2\endcsname{#2}`, 2101.09853): `\@ifdefinable` as there, then the declaration `\DeclareMathOperator`
  // (`m`: starred, limits) makes. Perl's binding lacks both (KNOWN_PERL_ERRORS #801).
  RawTeX!(
    r#"\gdef\newmcodes@{\mathcode`\'39\mathcode`\*42\mathcode`\.24890 %
  \ifnum\mathcode`\-=45 \else
    \mathchardef\std@minus\mathcode`\-\relax
  \fi
  \mathcode`\-45\mathcode`\/47\mathcode`\:24634\relax}
\long\def\@declmathop#1#2#3{\@ifdefinable{#2}{\csname lx@amsopn@declmathop@#1\endcsname#2{#3}}}
\def\lx@amsopn@declmathop@m{\DeclareMathOperator*}
\def\lx@amsopn@declmathop@o{\DeclareMathOperator}"#
  );
  DefMacro!("\\nolimits@", "\\nolimits");
  // amsopn.sty:90 `\def\operatorfont{\operator@font}` — the user-level name
  // (glosmathtools.sty:54 `\sbu` uses it ~54× per manual). Perl's binding
  // omits it too: KNOWN_PERL_ERRORS #146. Witness glosmathtools en/fr.
  DefMacro!("\\operatorfont", "\\operator@font");
  DefMacro!("\\nmlimits@", "\\displaylimits");
  DefMacro!(
    "\\qopname{}{}{}",
    "\\mathop{#3}\\csname n#2limits@\\endcsname"
  );
});
