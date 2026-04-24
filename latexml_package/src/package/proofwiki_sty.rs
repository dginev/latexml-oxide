//! proofwiki.sty — ProofWiki mathematical notation package
//! Perl: proofwiki.sty.ltxml — 326 lines
//! Pure notation/abbreviation package: Greek letters, bold symbols,
//! number sets, bracketing constructs, operators, distributions.
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  RequirePackage!("texvc");

  // Greek letters and symbol preferences — Perl L23-63
  DefMacro!("\\empty", "\\varnothing");
  DefMacro!("\\P", "\\unicode{xb6}");
  DefMacro!("\\Alpha", "\\unicode{x391}");
  DefMacro!("\\Beta", "\\unicode{x392}");
  DefMacro!("\\Epsilon", "\\unicode{x395}");
  DefMacro!("\\Zeta", "\\unicode{x396}");
  DefMacro!("\\Eta", "\\unicode{x397}");
  DefMacro!("\\Iota", "\\unicode{x399}");
  DefMacro!("\\Kappa", "\\unicode{x39a}");
  DefMacro!("\\Mu", "\\unicode{x39c}");
  DefMacro!("\\Nu", "\\unicode{x39d}");
  // Perl L34-38: also override \Pi/\Sigma/\Rho/\Tau/\Chi to explicit unicode
  DefMacro!("\\Pi", "\\unicode{x3a0}");
  DefMacro!("\\Rho", "\\unicode{x3a1}");
  DefMacro!("\\Sigma", "\\unicode{x3a3}");
  DefMacro!("\\Tau", "\\unicode{x3a4}");
  DefMacro!("\\Chi", "\\unicode{x3a7}");

  // Bold symbol variants — Perl L39-62
  DefMacro!("\\bsalpha", "\\boldsymbol \\alpha");
  DefMacro!("\\bsbeta", "\\boldsymbol \\beta");
  DefMacro!("\\bsgamma", "\\boldsymbol \\gamma");
  DefMacro!("\\bsdelta", "\\boldsymbol \\delta");
  DefMacro!("\\bsepsilon", "\\boldsymbol \\epsilon");
  DefMacro!("\\bszeta", "\\boldsymbol \\zeta");
  DefMacro!("\\bseta", "\\boldsymbol \\eta");
  DefMacro!("\\bstheta", "\\boldsymbol \\theta");
  DefMacro!("\\bsiota", "\\boldsymbol \\iota");
  DefMacro!("\\bskappa", "\\boldsymbol \\kappa");
  DefMacro!("\\bslambda", "\\boldsymbol \\lambda");
  DefMacro!("\\bsmu", "\\boldsymbol \\mu");
  DefMacro!("\\bsnu", "\\boldsymbol \\nu");
  DefMacro!("\\bsxi", "\\boldsymbol \\xi");
  DefMacro!("\\bsomicron", "\\boldsymbol \\omicron");
  DefMacro!("\\bspi", "\\boldsymbol \\pi");
  DefMacro!("\\bsrho", "\\boldsymbol \\rho");
  DefMacro!("\\bssigma", "\\boldsymbol \\sigma");
  DefMacro!("\\bstau", "\\boldsymbol \\tau");
  DefMacro!("\\bsupsilon", "\\boldsymbol \\upsilon");
  DefMacro!("\\bsphi", "\\boldsymbol \\phi");
  DefMacro!("\\bschi", "\\boldsymbol \\chi");
  DefMacro!("\\bspsi", "\\boldsymbol \\psi");
  DefMacro!("\\bsomega", "\\boldsymbol \\omega");

  // Number sets and constants — Perl L64-115
  DefMacro!("\\pounds", "{\\it\\unicode{xA3}}");
  DefMacro!("\\C", "\\mathbb C");
  DefMacro!("\\N", "\\mathbb N");
  DefMacro!("\\Q", "\\mathbb Q");
  DefMacro!("\\R", "\\mathbb R");
  DefMacro!("\\Z", "\\mathbb Z");
  DefMacro!("\\O", "\\varnothing");
  DefMacro!("\\T", "\\mathrm T");
  DefMacro!("\\F", "\\mathrm F");
  DefMacro!("\\GF", "\\mathbb F");
  DefMacro!("\\H", "\\mathbb H");
  DefMacro!("\\bszero", "\\boldsymbol 0");
  DefMacro!("\\bsone", "\\boldsymbol 1");
  DefMacro!("\\bst", "\\boldsymbol t");
  DefMacro!("\\bsv", "\\boldsymbol v");
  DefMacro!("\\bsw", "\\boldsymbol w");
  DefMacro!("\\bsx", "\\boldsymbol x");
  DefMacro!("\\bsy", "\\boldsymbol y");
  DefMacro!("\\bsz", "\\boldsymbol z");
  DefMacro!("\\bsDelta", "\\boldsymbol \\Delta");
  DefMacro!("\\E", "\\mathrm e");
  DefMacro!("\\rd", "\\,\\mathrm d");
  DefMacro!("\\d", "\\mathrm d");
  DefMacro!("\\rdelta", "\\,\\delta");
  DefMacro!("\\rD", "\\mathrm D");
  DefMacro!("\\bold{}", "{\\bf #1}");

  // Calligraphic letters — Perl L120-145
  DefMacro!("\\AA", "\\mathcal A");
  DefMacro!("\\BB", "\\mathcal B");
  DefMacro!("\\CC", "\\mathcal C");
  DefMacro!("\\DD", "\\mathcal D");
  DefMacro!("\\EE", "\\mathcal E");
  DefMacro!("\\FF", "\\mathcal F");
  DefMacro!("\\GG", "\\mathcal G");
  DefMacro!("\\HH", "\\mathcal H");
  DefMacro!("\\II", "\\mathcal I");
  DefMacro!("\\JJ", "\\mathcal J");
  DefMacro!("\\KK", "\\mathcal K");
  DefMacro!("\\LL", "\\mathcal L");
  DefMacro!("\\MM", "\\mathcal M");
  DefMacro!("\\NN", "\\mathcal N");
  DefMacro!("\\OO", "\\mathcal O");
  DefMacro!("\\PP", "\\mathcal P");
  DefMacro!("\\QQ", "\\mathcal Q");
  DefMacro!("\\RR", "\\mathcal R");
  DefMacro!("\\SS", "\\mathcal S");
  DefMacro!("\\TT", "\\mathcal T");
  DefMacro!("\\UU", "\\mathcal U");
  DefMacro!("\\VV", "\\mathcal V");
  DefMacro!("\\WW", "\\mathcal W");
  DefMacro!("\\XX", "\\mathcal X");
  DefMacro!("\\YY", "\\mathcal Y");
  DefMacro!("\\ZZ", "\\mathcal Z");

  // Operators — Perl L147-183
  DefMacro!("\\lcm", "\\operatorname{lcm}");
  DefMacro!("\\cosec", "\\operatorname{cosec}");
  DefMacro!("\\sech", "\\operatorname{sech}");
  DefMacro!("\\csch", "\\operatorname{csch}");
  DefMacro!("\\arccot", "\\operatorname{arccot}");
  DefMacro!("\\arccsc", "\\operatorname{arccsc}");
  DefMacro!("\\arcsec", "\\operatorname{arcsec}");
  DefMacro!("\\hav", "\\operatorname{hav}");
  DefMacro!("\\vers", "\\operatorname{vers}");
  DefMacro!("\\cis", "\\operatorname{cis}");
  DefMacro!("\\Ci", "\\operatorname{Ci}");
  DefMacro!("\\Si", "\\operatorname{Si}");
  DefMacro!("\\Li", "\\operatorname{Li}");
  DefMacro!("\\Ei", "\\operatorname{Ei}");
  DefMacro!("\\arsinh", "\\operatorname{arsinh}");
  DefMacro!("\\arcosh", "\\operatorname{arcosh}");
  DefMacro!("\\artanh", "\\operatorname{artanh}");
  DefMacro!("\\arcoth", "\\operatorname{arcoth}");
  DefMacro!("\\arsech", "\\operatorname{arsech}");
  DefMacro!("\\arcsch", "\\operatorname{arcsch}");
  // Perl L164-169: capitalized hyperbolic functions
  DefMacro!("\\Sinh", "\\operatorname{Sinh}");
  DefMacro!("\\Cosh", "\\operatorname{Cosh}");
  DefMacro!("\\Sech", "\\operatorname{Sech}");
  DefMacro!("\\Csch", "\\operatorname{Csch}");
  DefMacro!("\\Tanh", "\\operatorname{Tanh}");
  DefMacro!("\\Coth", "\\operatorname{Coth}");
  // Perl L176-181: capitalized inverse hyperbolic
  DefMacro!("\\Arsinh", "\\operatorname{Arsinh}");
  DefMacro!("\\Arcosh", "\\operatorname{Arcosh}");
  DefMacro!("\\Artanh", "\\operatorname{Artanh}");
  DefMacro!("\\Arcoth", "\\operatorname{Arcoth}");
  DefMacro!("\\Arsech", "\\operatorname{Arsech}");
  DefMacro!("\\Arcsch", "\\operatorname{Arcsch}");
  DefMacro!("\\erf", "\\operatorname{erf}");
  DefMacro!("\\erfc", "\\operatorname{erfc}");

  // Bracketing constructs — Perl L187-226
  DefMacro!("\\paren{}", "\\left( #1 \\right)");
  DefMacro!("\\sqbrk{}", "\\left[ #1 \\right]");
  DefMacro!("\\set{}", "\\left\\lbrace #1 \\right\\rbrace");
  DefMacro!("\\cmod{}", "\\left\\lvert #1 \\right\\rvert");
  DefMacro!("\\polar{}", "\\left\\langle #1 \\right\\rangle");
  DefMacro!("\\norm{}", "\\left\\lVert #1 \\right\\rVert");
  DefMacro!("\\floor{}", "\\left\\lfloor #1 \\right\\rfloor");
  DefMacro!("\\ceiling{}", "\\left\\lceil #1 \\right\\rceil");
  DefMacro!("\\closedint{}{}", "\\left[ #1 \\,.\\.\\, #2 \\right]");
  DefMacro!("\\openint{}{}", "\\left( #1 \\,.\\.\\, #2 \\right)");
  DefMacro!("\\tuple{}", "\\left( #1 \\right)");
  DefMacro!("\\struct{}", "\\left( #1 \\right)");
  DefMacro!("\\sequence{}", "\\left\\langle #1 \\right\\rangle");
  DefMacro!("\\family{}", "\\left\\langle #1 \\right\\rangle");
  DefMacro!("\\innerprod{}{}", "\\left\\langle #1, #2 \\right\\rangle");
  DefMacro!("\\gen{}", "{\\left\\langle #1 \\right\\rangle}");
  DefMacro!("\\order{}", "\\left\\lvert #1 \\right\\rvert");
  DefMacro!("\\size{}", "\\left\\lvert #1 \\right\\rvert");
  DefMacro!("\\card{}", "\\left\\lvert #1 \\right\\rvert");
  DefMacro!("\\map{}{}", "#1 \\left( #2 \\right)");
  DefMacro!("\\braket{}{}", "{\\left\\langle #1 \\, \\middle \\vert \\, #2 \\right\\rangle{}}");
  // Perl L188-208: one-sided bracketing + extended interval + multiset
  DefMacro!("\\leftparen{}", "\\left( #1 \\right.");
  DefMacro!("\\rightparen{}", "\\left. #1 \\right)");
  DefMacro!("\\leftset{}", "\\left\\lbrace #1 \\right.");
  DefMacro!("\\rightset{}", "\\left. #1 \\right\\rbrace");
  DefMacro!("\\nint{}", "\\left\\lfloor #1 \\right\\rceil");
  DefMacro!("\\hointl{}{}", "\\left( #1 \\,.\\,.\\, #2 \\right]");
  DefMacro!("\\hointr{}{}", "\\left[ #1 \\,.\\,.\\, #2 \\right)");
  DefMacro!("\\horectl{}{}", "\\left(\\left( #1 \\,.\\,.\\, #2 \\right]\\right]");
  DefMacro!("\\horectr{}{}", "\\left[\\left[ #1 \\,.\\,.\\, #2 \\right)\\right)");
  DefMacro!("\\multiset{}", "\\left\\lbrace\\!\\left\\lbrace #1 \\right\\rbrace\\!\\right\\rbrace");
  DefMacro!("\\ideal{}", "\\left( #1 \\right)");
  DefMacro!("\\eqclass{}{}", "\\left[\\!\\left[ #1 \\right]\\!\\right]_{#2}");
  DefMacro!("\\bigsize{}", "\\bigl\\lvert #1 \\bigr\\rvert");
  DefMacro!("\\fractpart{}", "\\left\\lbrace #1 \\right\\rbrace");

  // Perl L227-235: set/algebra constructors
  DefMacro!("\\Syl{}{}", "\\mathrm{Syl}_{#1} \\left( #2 \\right)");
  DefMacro!("\\relcomp{}{}", "\\complement_{#1} \\left( #2 \\right)");
  DefMacro!("\\laptrans{}", "\\LL \\left\\lbrace #1 \\right\\rbrace");
  DefMacro!("\\invlaptrans{}", "\\LL^{-1} \\left\\lbrace #1 \\right\\rbrace");
  DefMacro!("\\intlimits{}{}{}", "\\left[ #1 \\right]_{#2}^{#3}");
  DefMacro!("\\bigintlimits{}{}{}", "\\Bigl[ #1 \\Bigr]_{#2}^{#3}");
  DefMacro!("\\valueat{}{}", "\\left. #1 \\right\\rvert_{#2}");
  DefMacro!("\\bigvalueat{}{}", "\\Bigl. #1 \\Bigr\\rvert_{#2}");

  // Perl L237-242: mathematical logic
  DefMacro!("\\Add", "\\operatorname{Add}");
  DefMacro!("\\Mult", "\\operatorname{Mult}");
  DefMacro!("\\Succ", "\\operatorname{Succ}");
  DefMacro!("\\Zero", "\\operatorname{Zero}");

  // Group/algebra notation — Perl L227-308
  DefMacro!("\\powerset{}", "\\PP \\left( #1 \\right)");
  DefMacro!("\\GL{}", "\\mathrm{GL} \\left( #1 \\right)");
  DefMacro!("\\SL{}", "\\mathrm{SL} \\left( #1 \\right)");
  DefMacro!("\\Aut{}", "\\mathrm{Aut} \\left( #1 \\right)");
  DefMacro!("\\Gal{}", "\\mathrm{Gal} \\left( #1 \\right)");
  DefMacro!("\\Dom{}", "\\mathrm{Dom} \\left( #1 \\right)");
  DefMacro!("\\Rng{}", "\\mathrm{Rng} \\left( #1 \\right)");
  DefMacro!("\\Img{}", "\\mathrm{Img} \\left( #1 \\right)");
  DefMacro!("\\Orb{}", "\\mathrm{Orb} \\left( #1 \\right)");
  DefMacro!("\\Stab{}", "\\mathrm{Stab} \\left( #1 \\right)");
  DefMacro!("\\Char{}", "\\mathrm{Char} \\left( #1 \\right)");
  DefMacro!("\\Spec{}", "\\mathrm{Spec} \\left( #1 \\right)");

  // Miscellaneous — Perl L244-325
  DefMacro!("\\hcf", "\\operatorname{hcf}");
  DefMacro!("\\inj", "\\operatorname{inj}");
  DefMacro!("\\rem", "\\operatorname{rem}");
  DefMacro!("\\pr", "\\operatorname{pr}");
  DefMacro!("\\proj", "\\operatorname{proj}");
  DefMacro!("\\sgn", "\\operatorname{sgn}");
  DefMacro!("\\len", "\\operatorname{len}");
  DefMacro!("\\grad", "\\operatorname{grad}");
  DefMacro!("\\curl", "\\operatorname{curl}");
  DefMacro!("\\tr", "\\operatorname{tr}");
  DefMacro!("\\dr{}", "\\mathrm{dr} \\left( #1 \\right)");
  DefMacro!("\\cl", "\\mathrm{cl}");
  DefMacro!("\\var{}", "\\mathsf{var} \\left( #1 \\right)");
  DefMacro!("\\cov{}", "\\mathsf{cov} \\left( #1 \\right)");
  DefMacro!("\\expect{}", "\\mathsf E \\left( #1 \\right)");
  DefMacro!("\\conjclass{}", "\\mathrm C_{#1}");
  DefMacro!("\\Log", "\\operatorname{Log}");
  DefMacro!("\\Ln", "\\operatorname{Ln}");
  DefMacro!("\\On", "\\operatorname{On}");
  DefMacro!("\\Area", "\\operatorname{Area}");
  DefMacro!("\\Card", "\\operatorname{Card}");
  DefMacro!("\\Frob", "\\operatorname{Frob}");
  DefMacro!("\\Bernoulli{}", "\\mathrm{Bern} \\left( #1 \\right)");
  DefMacro!("\\BetaDist{}{}", "\\mathrm{Beta} \\left( #1, #2 \\right)");
  DefMacro!("\\Binomial{}{}", "\\mathrm B \\left( #1, #2 \\right)");
  DefMacro!("\\Cauchy{}{}", "\\mathrm{Cauchy} \\left( #1, #2 \\right)");
  DefMacro!("\\NegativeBinomial{}{}", "\\mathrm{NB} \\left( #1, #2 \\right)");
  DefMacro!("\\Exponential{}", "\\mathrm{Exp} \\left( #1 \\right)");
  DefMacro!("\\Gaussian{}{}", "N \\left( #1, #2 \\right)");
  DefMacro!("\\Geometric{}", "\\mathrm G_0 \\left( #1 \\right)");
  DefMacro!("\\ShiftedGeometric{}", "\\mathrm G_1 \\left( #1 \\right)");
  DefMacro!("\\Poisson{}", "\\mathrm{Poisson} \\left( #1 \\right)");
  DefMacro!("\\StudentT{}", "t_{#1}");
  DefMacro!("\\ContinuousUniform{}{}", "\\mathrm U \\left[ #1 \\,.\\,.\\, #2 \\right]");
  DefMacro!("\\DiscreteUniform{}", "\\mathrm U \\left( #1 \\right)");
  DefMacro!("\\condprob{}{}", "\\Pr \\left( #1 \\, \\middle \\vert \\, #2 \\right)");
  DefMacro!("\\Arg{}", "\\mathrm{Arg} \\left( #1 \\right)");
  DefMacro!("\\domain{}", "\\mathrm{Dom} \\left( #1 \\right)");
  DefMacro!("\\cont{}", "\\mathrm{cont} \\left( #1 \\right)");
  DefMacro!("\\Cdm{}", "\\mathrm{Cdm} \\left( #1 \\right)");
  DefMacro!("\\adj{}", "\\mathrm{adj} \\left( #1 \\right)");
  DefMacro!("\\Preimg{}", "\\mathrm{Img}^{-1} \\left( #1 \\right)");
  DefMacro!("\\image{}", "\\Img #1 ");
  DefMacro!("\\Dic{}", "\\operatorname{Dic}_{#1}");
  DefMacro!("\\SU{}", "\\operatorname{SU_{#1} }");
  DefMacro!("\\Out{}", "\\mathrm{Out} \\left( #1 \\right)");
  DefMacro!("\\Inn{}", "\\mathrm{Inn} \\left( #1 \\right)");
  DefMacro!("\\Fix{}", "\\mathrm{Fix} \\left( #1 \\right)");
  DefMacro!("\\Nil{}", "\\mathrm{Nil} \\left( #1 \\right)");
  DefMacro!("\\Ord{}", "\\map{\\mathrm{Ord} } #1");
  DefMacro!("\\Res{}{}", "\\mathrm{Res} \\left( #1, #2 \\right)");
  DefMacro!("\\Int{}", "\\mathrm{Int} \\left( #1 \\right)");
  DefMacro!("\\Ext{}", "\\mathrm{Ext} \\left( #1 \\right)");
  DefMacro!("\\Ber", "\\mathrm{Ber}");
  DefMacro!("\\Bei", "\\mathrm{Bei}");
  DefMacro!("\\Rad", "\\mathrm{Rad}");
  DefMacro!("\\diam", "\\mathrm{diam}");
  DefMacro!("\\stratgame{}{}{}", "\\left\\langle #1, \\sequence #2, \\sequence #3 \\right\\rangle");
  DefMacro!("\\divides", "\\mathrel \\backslash");
  DefMacro!("\\PV", "\\mathrm{PV} \\displaystyle \\int");
  DefMacro!("\\leadstoandfrom", "\\mathrel \\leftrightsquigarrow");
  DefMacro!("\\degrees", "^\\circ");
  DefMacro!("\\radians", "\\, \\mathrm{rad}");
  DefMacro!("\\cels", "\\, \\degrees \\mathrm C");
  DefMacro!("\\fahr", "\\, \\degrees \\mathrm F");
  DefMacro!("\\Re", "\\mathfrak{Re}");
  DefMacro!("\\Im", "\\mathfrak{Im}");
  DefMacro!("\\ds", "\\displaystyle");

  // Perl L218: \index{G}{H} — subgroup index notation [G : H].
  // ProofWiki redefines the LaTeX \index (normally an indexing command) as a
  // math macro. Both the LaTeX-index and math-index senses take the `{}{}`
  // shape here; this package is math-notation-only, so the override stands.
  // Previously unported.
  DefMacro!("\\index{}{}", "\\left[ #1 : #2 \\right]");
});
