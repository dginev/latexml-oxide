use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: eulervm.sty.ltxml
  // There's really nothing to do for LaTeXML.
  Let!("\\upDelta", "\\Delta");
  Let!("\\upOmega", "\\Omega");
  // eulervm.sty:58 `\DeclareMathAlphabet{\mathbold}{U}{zeur}{b}{n}`: the bold upright Euler letters, the package's own
  // bold alphabet (27 papers; 0710.3183, 1111.7285, 1203.0791, 1203.1733, 1405.0292). Perl's binding defines none
  // (KNOWN_PERL_ERRORS #820). An alphabet of its argument, as the kernel's `\mathbf`: the kernel's
  // `\DeclareMathAlphabet` makes a font switch that runs on past the argument (`\mathbold{x}+y` would set y bold
  // too; KNOWN_PERL_ERRORS #823). Guard: `perfect_kernel_batch68j::eulervm_mathbold_is_bold_upright`.
  DefConstructor!("\\mathbold{}", "#1", bounded => true, require_math => true,
    font => { family => "serif", series => "bold", shape => "upright" });
});
