use crate::prelude::*;
LoadDefinitions!({
  RequirePackage!("amsgen");

  // \DeclareMathOperator*{cs}{text}
  //
  // Use `.untex()` instead of `.to_string()` — the latter concatenates
  // token texts with no separator, so `{\rm Aut}` (where the space after
  // `\rm` was swallowed by control-word tokenization) becomes `{\rmAut}`,
  // which tokenizes back as the single undefined CS `\rmAut`. `untex()`
  // inserts a space at CS→letter boundaries (tokens.rs L392-405), so the
  // round-trip through `def_math`'s internal `mouth::tokenize_internal`
  // preserves the correct token structure.
  //
  // Perl avoids this entirely by passing Tokens directly to DefMathI via
  // `Invocation(T_CS('\operatorname'), $star, $text)` — no stringify
  // round-trip. Rust's `def_math` takes String, so we use the TeX-safe
  // stringifier.
  //
  // Fixes sandbox papers 0806.2705 (`\rmTr`) and 0808.0535 (`\rmAut`/
  // `\rmSpan`) whose `\DeclareMathOperator{\X}{{\rm X}}` patterns would
  // otherwise produce undefined `\rmX` errors.
  //
  // The text is EXPANDED before stringifying. Perl's Tokens reach DefMathI
  // unexpanded and expand at digest time inside the defining catcode regime;
  // the stringify round-trip instead re-tokenizes under `tokenize_internal`'s
  // sty-state catcodes (dialect.rs:478, mouth.rs:1292), so an expl3 name like
  // `\cs_to_str:N \asinh` (numerica.sty:50-51, manual numerica.tex:148)
  // shatters into `\cs_to_str` `_` `:N` … at every use — 100 malformed:ltx +
  // a Stomach:Recursion Fatal. Expanding first yields the plain letters the
  // author meant; unexpandable font switches (`\rm`, `\mathrm`) survive intact.
  DefPrimitive!("\\DeclareMathOperator OptionalMatch:* {Token} {}", sub[(star, cs, text)] {
    let text_str = do_expand(text)?.untex();
    let has_star = star.is_some();
    // Perl L26-29: scriptpos => ($star ? \&doScriptpos : 'post') — starred form
    // gets dynamic mid/post from current display style; bare form is always 'post'.
    // revert_as => 'context' so source-export emits the user-facing CS name
    // rather than the operatorname expansion. Both were previously dropped.
    let opts = MathPrimitiveOptions {
      role: Some(if has_star { "OPERATOR" } else { "OPFUNCTION" }.to_string()),
      font: Some(fontmap!(family => "serif", series => "medium", shape => "upright").into()),
      scriptpos: if has_star { None } else { Some("post".to_string()) },
      dynamic_scriptpos: has_star,
      revert_as: Some(Cow::Borrowed("context")),
      ..Default::default()};
    def_math(cs, None, text_str, opts)?;
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
