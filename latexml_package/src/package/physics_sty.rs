//! physics.sty — semantic physics macros
//! Perl: physics.sty.ltxml (729 lines)
//!
//! Pragmatic port: uses simple TeX-level definitions rather than I_dual infrastructure.
//! Produces correct visual output; semantic markup (XMDual) not yet implemented.
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  RequirePackage!("amsmath");

  //======================================================================
  // Automatic bracing

  DefMacro!("\\quantity{}", r"\left(#1\right)");
  Let!("\\qty", "\\quantity");

  DefMacro!("\\pqty{}", r"\left(#1\right)");
  DefMacro!("\\bqty{}", r"\left[#1\right]");
  DefMacro!("\\vqty{}", r"\left\vert #1\right\vert ");
  DefMacro!("\\Bqty{}", r"\left\{#1\right\}");

  DefMacro!("\\absolutevalue{}", r"\left\vert #1\right\vert ");
  DefMacro!("\\norm{}", r"\left\Vert #1\right\Vert ");
  Let!("\\abs", "\\absolutevalue");

  DefMacro!("\\evaluated{}", r"\left.#1\right\vert ");
  Let!("\\eval", "\\evaluated");

  DefMacro!("\\order{}", r"\mathcal{O}\left(#1\right)");
  DefMacro!("\\ordersymbol", r"\mathcal{O}");

  DefMacro!("\\commutator{}{}", r"\left[#1,#2\right]");
  DefMacro!("\\anticommutator{}{}", r"\left\{#1,#2\right\}");
  DefMacro!("\\poissonbracket{}{}", r"\left\{#1,#2\right\}");
  Let!("\\comm", "\\commutator");
  Let!("\\acomm", "\\anticommutator");
  Let!("\\pb", "\\poissonbracket");

  //======================================================================
  // Vector Notation
  DefMacro!("\\vectorbold{}", r"\mathbf{#1}");
  DefMacro!("\\vectorarrow{}", r"\overrightarrow{\mathbf{#1}}");
  DefMacro!("\\vectorunit{}", r"\hat{\mathbf{#1}}");
  Let!("\\vb", "\\vectorbold");
  Let!("\\va", "\\vectorarrow");
  Let!("\\vu", "\\vectorunit");

  DefMath!("\\dotproduct", None, "\u{22C5}", role => "MULOP", meaning => "dot-product");
  DefMath!("\\crossproduct", None, "\u{00D7}", role => "MULOP", meaning => "cross-product");
  Let!("\\vdot", "\\dotproduct");
  Let!("\\cross", "\\crossproduct");
  Let!("\\cp", "\\crossproduct");

  DefMath!("\\gradient", None, "\u{2207}", role => "OPERATOR", meaning => "gradient");
  DefMath!("\\divergence", None, "\u{2207}\u{22C5}", role => "OPERATOR", meaning => "divergence");
  DefMath!("\\curl", None, "\u{2207}\u{00D7}", role => "OPERATOR", meaning => "curl");
  DefMacro!("\\laplacian", r"\nabla^{2}");
  Let!("\\grad", "\\gradient");
  Let!("\\divisionsymbol", "\\div");
  Let!("\\div", "\\divergence");

  //======================================================================
  // Operators
  Let!("\\sine", "\\sin");
  Let!("\\cosine", "\\cos");
  Let!("\\tangent", "\\tan");
  Let!("\\cosecant", "\\csc");
  Let!("\\secant", "\\sec");
  Let!("\\cotangent", "\\cot");
  Let!("\\hypsine", "\\sinh");
  Let!("\\hypcosine", "\\cosh");
  Let!("\\hyptangent", "\\tanh");
  Let!("\\arcsine", "\\arcsin");
  Let!("\\arccosine", "\\arccos");
  Let!("\\arctangent", "\\arctan");
  Let!("\\asin", "\\arcsin");
  Let!("\\acos", "\\arccos");
  Let!("\\atan", "\\arctan");
  Let!("\\exponential", "\\exp");
  Let!("\\logarithm", "\\log");
  Let!("\\naturallogarithm", "\\ln");
  Let!("\\determinant", "\\det");
  Let!("\\Probability", "\\Pr");
  Let!("\\asine", "\\arcsin");
  Let!("\\acosine", "\\arccos");
  Let!("\\atangent", "\\arctan");

  DefMacro!("\\arccsc", r"\operatorname{arccsc}");
  DefMacro!("\\arcsec", r"\operatorname{arcsec}");
  DefMacro!("\\arccot", r"\operatorname{arccot}");
  DefMacro!("\\csch", r"\operatorname{csch}");
  DefMacro!("\\sech", r"\operatorname{sech}");
  Let!("\\hypcosecant", "\\csch");
  Let!("\\hypsecant", "\\sech");
  DefMacro!("\\hypcotangent", r"\operatorname{coth}");
  Let!("\\acsc", "\\arccsc");
  Let!("\\asec", "\\arcsec");
  Let!("\\acot", "\\arccot");
  Let!("\\arccosecant", "\\arccsc");
  Let!("\\arcsecant", "\\arcsec");
  Let!("\\arccotangent", "\\arccot");
  Let!("\\acosecant", "\\arccsc");
  Let!("\\asecant", "\\arcsec");
  Let!("\\acotangent", "\\arccot");

  DefMacro!("\\trace", r"\operatorname{tr}");
  DefMacro!("\\Trace", r"\operatorname{Tr}");
  DefMacro!("\\rank", r"\operatorname{rank}");
  DefMacro!("\\erf", r"\operatorname{erf}");
  DefMacro!("\\Res", r"\operatorname{Res}");
  DefMacro!("\\principalvalue", r"\mathcal{P}");
  DefMacro!("\\PV", r"\operatorname{P.V.}");
  Let!("\\tr", "\\trace");
  Let!("\\Tr", "\\Trace");
  Let!("\\pv", "\\principalvalue");
  Let!("\\real", "\\Re");
  Let!("\\imaginary", "\\Im");

  //======================================================================
  // Quick quad text
  DefMacro!("\\qqtext{}", r"\quad\text{#1}\quad");
  DefMacro!("\\qcomma", r",\quad");
  DefMacro!("\\qcc", r"\quad\text{c.c.}\quad");
  Let!("\\qq", "\\qqtext");
  Let!("\\qc", "\\qcomma");
  DefMacro!("\\qif", r"\quad\text{if}\quad");
  DefMacro!("\\qthen", r"\quad\text{then}\quad");
  DefMacro!("\\qelse", r"\quad\text{else}\quad");
  DefMacro!("\\qotherwise", r"\quad\text{otherwise}\quad");
  DefMacro!("\\qunless", r"\quad\text{unless}\quad");
  DefMacro!("\\qgiven", r"\quad\text{given}\quad");
  DefMacro!("\\qusing", r"\quad\text{using}\quad");
  DefMacro!("\\qassume", r"\quad\text{assume}\quad");
  DefMacro!("\\qsince", r"\quad\text{since}\quad");
  DefMacro!("\\qlet", r"\quad\text{let}\quad");
  DefMacro!("\\qfor", r"\quad\text{for}\quad");
  DefMacro!("\\qall", r"\quad\text{all}\quad");
  DefMacro!("\\qeven", r"\quad\text{even}\quad");
  DefMacro!("\\qodd", r"\quad\text{odd}\quad");
  DefMacro!("\\qinteger", r"\quad\text{integer}\quad");
  DefMacro!("\\qand", r"\quad\text{and}\quad");
  DefMacro!("\\qor", r"\quad\text{or}\quad");
  DefMacro!("\\qas", r"\quad\text{as}\quad");
  DefMacro!("\\qin", r"\quad\text{in}\quad");

  //======================================================================
  // Derivatives
  Let!("\\flatfrac", "\\ifrac");

  DefMacro!("\\differential{}", r"\mathrm{d}#1");
  DefMacro!("\\variation{}", r"\delta #1");
  Let!("\\dd", "\\differential");
  Let!("\\var", "\\variation");

  DefMacro!("\\derivative{}{}", r"\frac{\mathrm{d}#1}{\mathrm{d}#2}");
  DefMacro!("\\partialderivative{}{}", r"\frac{\partial #1}{\partial #2}");
  DefMacro!("\\functionalderivative{}{}", r"\frac{\delta #1}{\delta #2}");
  Let!("\\dv", "\\derivative");
  Let!("\\pdv", "\\partialderivative");
  Let!("\\pderivative", "\\partialderivative");
  Let!("\\fdv", "\\functionalderivative");

  //======================================================================
  // Dirac bra-ket notation
  DefMacro!("\\ket{}", r"\left\vert #1\right\rangle ");
  DefMacro!("\\bra{}", r"\left\langle #1\right\vert ");
  DefMacro!("\\innerproduct{}{}", r"\left\langle #1\middle\vert #2\right\rangle ");
  DefMacro!("\\outerproduct{}{}", r"\left\vert #1\right\rangle\left\langle #2\right\vert ");
  DefMacro!("\\expectationvalue{}", r"\left\langle #1\right\rangle ");
  DefMacro!("\\matrixelement{}{}{}", r"\left\langle #1\middle\vert #2\middle\vert #3\right\rangle ");

  Let!("\\braket", "\\innerproduct");
  Let!("\\ip", "\\innerproduct");
  Let!("\\dyad", "\\outerproduct");
  Let!("\\ketbra", "\\outerproduct");
  Let!("\\op", "\\outerproduct");
  Let!("\\expval", "\\expectationvalue");
  Let!("\\ev", "\\expectationvalue");
  Let!("\\matrixel", "\\matrixelement");
  Let!("\\mel", "\\matrixelement");

  //======================================================================
  // Matrix macros
  DefMacro!("\\matrixquantity{}", r"\begin{pmatrix}#1\end{pmatrix}");
  DefMacro!("\\pmqty{}", r"\begin{pmatrix}#1\end{pmatrix}");
  DefMacro!("\\Pmqty{}", r"\begin{pmatrix}#1\end{pmatrix}");
  DefMacro!("\\bmqty{}", r"\begin{bmatrix}#1\end{bmatrix}");
  DefMacro!("\\vmqty{}", r"\begin{vmatrix}#1\end{vmatrix}");
  DefMacro!("\\smallmatrixquantity{}", r"\begin{pmatrix}#1\end{pmatrix}");
  DefMacro!("\\spmqty{}", r"\begin{pmatrix}#1\end{pmatrix}");
  DefMacro!("\\sPmqty{}", r"\begin{pmatrix}#1\end{pmatrix}");
  DefMacro!("\\sbmqty{}", r"\begin{bmatrix}#1\end{bmatrix}");
  DefMacro!("\\svmqty{}", r"\begin{vmatrix}#1\end{vmatrix}");
  DefMacro!("\\matrixdeterminant{}", r"\begin{vmatrix}#1\end{vmatrix}");
  DefMacro!("\\smallmatrixdeterminant{}", r"\begin{vmatrix}#1\end{vmatrix}");

  Let!("\\mqty", "\\matrixquantity");
  Let!("\\smqty", "\\smallmatrixquantity");
  Let!("\\mdet", "\\matrixdeterminant");
  Let!("\\smdet", "\\smallmatrixdeterminant");

  DefMacro!("\\identitymatrix{}", "");
  DefMacro!("\\zeromatrix{}{}", "");
  DefMacro!("\\paulimatrix{}", "");
  DefMacro!("\\diagonalmatrix[]{}", "");
  DefMacro!("\\antidiagonalmatrix[]{}", "");
  DefMacro!("\\xmatrix{}{}{}", "");

  Let!("\\imat", "\\identitymatrix");
  Let!("\\xmat", "\\xmatrix");
  Let!("\\zmat", "\\zeromatrix");
  Let!("\\pmat", "\\paulimatrix");
  Let!("\\dmat", "\\diagonalmatrix");
  Let!("\\admat", "\\antidiagonalmatrix");
});
