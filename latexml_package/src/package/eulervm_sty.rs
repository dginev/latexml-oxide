use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: eulervm.sty.ltxml
  // There's really nothing to do for LaTeXML.
  // eulervm.sty:59-69 `\DeclareMathSymbol\Gamma{\mathalpha}{letters}{"00}`…, its `letters` the upright Euler font
  // (:53 `{U}{zeur}{m}{n}`): the capital Greek letters upright, over a class's italic ones (llncs.cls:379-389;
  // KNOWN_PERL_ERRORS #1147), and :97-98 `\DeclareMathSymbol\upDelta{\mathord}{letters}{"01}`, `\upOmega`: the same
  // glyphs, their own names in `tex=` (review rev79e finding 2; witness 2606.09074).
  declare_capital_greek("", None)?;
  declare_capital_greek_named("up", None, Some(&["Delta", "Omega"]))?;
  // eulervm.sty:58 `\DeclareMathAlphabet{\mathbold}{U}{zeur}{b}{n}`: the bold upright Euler letters, the package's own
  // bold alphabet (27 papers; 0710.3183, 1111.7285, 1203.0791, 1203.1733, 1405.0292). Perl's binding defines none
  // (KNOWN_PERL_ERRORS #820). An alphabet of its argument, as the kernel's `\mathbf`: the kernel's
  // `\DeclareMathAlphabet` makes a font switch that runs on past the argument (`\mathbold{x}+y` would set y bold
  // too; KNOWN_PERL_ERRORS #823). Guard: `perfect_kernel_batch68j::eulervm_mathbold_is_bold_upright`.
  DefConstructor!("\\mathbold{}", "#1", bounded => true, require_math => true,
    font => { family => "serif", series => "bold", shape => "upright" });
});
