use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  DefMath!("\\bra{}", "\\langle#1|",            meaning => "bra");
  DefMath!("\\Bra{}", "\\left\\langle#1\\right|", meaning => "bra");
  DefMath!("\\ket{}", "|#1\\rangle",           meaning => "ket");
  DefMath!("\\Ket{}", "\\left|#1\\right\\rangle", meaning => "ket");
  DefMath!("\\lx@braket@{}", "\\langle#1\\rangle", meaning => "expectation");
  DefMath!("\\lx@Braket@{}", "\\left\\langle#1\\right\\rangle", meaning => "expectation");
  DefMath!("\\lx@braket@V{}{}", "\\langle#1\\,|\\,#2\\rangle", meaning => "inner-product");
  DefMath!("\\lx@braket@D{}{}", "\\langle#1\\,\\|\\,#2\\rangle", meaning => "inner-product");
  DefMath!("\\lx@Braket@V{}{}", "\\left\\langle#1\\,\\middle|\\,#2\\right\\rangle", meaning => "inner-product");
  DefMath!("\\lx@Braket@D{}{}", "\\left\\langle#1\\,\\middle\\|\\,#2\\right\\rangle", meaning => "inner-product");
  // All braket variants (Perl L90-114)
  DefMath!("\\lx@braket@VV{}{}{}", "\\langle#1\\,|#2\\,|\\,#3\\rangle", meaning => "quantum-operator-product");
  DefMath!("\\lx@braket@VD{}{}{}", "\\langle#1\\,|\\,#2\\,\\|\\,#3\\rangle", meaning => "quantum-operator-product");
  DefMath!("\\lx@braket@DV{}{}{}", "\\langle#1\\,\\|\\,#2\\,|\\,#3\\rangle", meaning => "quantum-operator-product");
  DefMath!("\\lx@braket@DD{}{}{}", "\\langle#1\\,\\|\\,#2\\,\\|\\,#3\\rangle", meaning => "quantum-operator-product");
  DefMath!("\\lx@Braket@VV{}{}{}", "\\left\\langle#1\\,\\middle|\\,#2\\,\\middle|\\,#3\\right\\rangle", meaning => "quantum-operator-product");
  DefMath!("\\lx@Braket@VD{}{}{}", "\\left\\langle#1\\,\\middle|\\,#2\\,\\middle\\|\\,#3\\right\\rangle", meaning => "quantum-operator-product");
  DefMath!("\\lx@Braket@DV{}{}{}", "\\left\\langle#1\\,\\middle\\|\\,#2\\,\\middle|\\,#3\\right\\rangle", meaning => "quantum-operator-product");
  DefMath!("\\lx@Braket@DD{}{}{}", "\\left\\langle#1\\,\\middle\\|\\,#2\\,\\middle\\|\\,#3\\right\\rangle", meaning => "quantum-operator-product");

  // TODO: \braket and \Braket should split on | inside the argument
  // For now, pass through as single-arg (expectation value)
  DefMacro!("\\braket{}", "\\lx@braket@{#1}");
  DefMacro!("\\Braket{}", "\\lx@Braket@{#1}");

  // Set notation (Perl L117-146)
  DefMath!("\\lx@set@{}", "\\{#1\\}", meaning => "set");
  DefMath!("\\lx@Set@{}", "\\left\\{#1\\right\\}", meaning => "set");
  DefMath!("\\lx@set@V{}{}", "\\{#1\\;|\\;#2\\}", meaning => "set");
  DefMath!("\\lx@set@D{}{}", "\\{#1\\;\\|\\;#2\\}", meaning => "set");
  DefMath!("\\lx@Set@V{}{}", "\\left\\{#1\\;\\middle|\\;#2\\right\\}", meaning => "set");
  DefMath!("\\lx@Set@D{}{}", "\\left\\{#1\\;\\middle\\|\\;#2\\right\\}", meaning => "set");
  // TODO: \set and \Set should split on | inside the argument
  DefMacro!("\\set{}", "\\lx@set@{#1}");
  DefMacro!("\\Set{}", "\\lx@Set@{#1}");
});
